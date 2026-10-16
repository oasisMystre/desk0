use std::{
  fs::File,
  io::Write,
  path::PathBuf,
  time::{SystemTime, UNIX_EPOCH},
};

use ironrdp::cliprdr::{
  backend::ClipboardMessage,
  pdu::{ClipboardFileAttributes, FileContentsFlags, FileContentsRequest, FileDescriptor},
};

use crate::cliprdr::{
  cliprdr_backend::MAX_CLIPBOARD_CHUNK,
  types::{
    Event,
    clipboard_event::{ClipboardEvent, FileDownloadStatus},
  },
};

pub struct DownloadTask {
  pub index: u32,
  pub path: PathBuf,
  pub size: u64,
  pub received: u64,
  pub file: Option<File>,
}

#[derive(Default)]
pub struct DownloadState {
  inflight: bool,
  pub queue: Vec<DownloadTask>,
  pub downloaded: Vec<PathBuf>,
  pub current: Option<DownloadTask>,
}

pub enum DownloadEvent {
  DownloadComplete(Vec<PathBuf>),
  SendFileContentsRequest(Event),
}

impl DownloadState {
  pub fn clear(&mut self) {
    self.inflight = false;
    self.queue.clear();
    self.current = None;
    self.downloaded.clear();
  }

  pub fn next_event(&mut self) -> Option<DownloadEvent> {
    if self.inflight {
      return None;
    }

    loop {
      if self.current.is_none() {
        if let Some(mut task) = self.queue.pop() {
          match File::create(&task.path) {
            Ok(_) if task.size == 0 => {
              self.downloaded.push(task.path);
              continue;
            }
            Ok(file) => {
              task.file = Some(file);
              self.current = Some(task);
            }
            Err(error) => {
              tracing::error!(?task.path, ?error, "Failed to create local file for download");
              continue;
            }
          }
        } else if !self.downloaded.is_empty() {
          return Some(DownloadEvent::DownloadComplete(std::mem::take(
            &mut self.downloaded,
          )));
        } else {
          return None;
        }
      }

      let task = self.current.as_ref()?;
      let remaining = task.size.saturating_sub(task.received);

      if remaining == 0 {
        let task = self.current.take()?;
        self.downloaded.push(task.path);
        continue;
      }

      let request = FileContentsRequest {
        data_id: None,
        stream_id: task.index,
        index: task.index as i32,
        position: task.received,
        flags: FileContentsFlags::RANGE,
        requested_size: remaining.min(MAX_CLIPBOARD_CHUNK as u64) as u32,
      };

      self.inflight = true;
      return Some(DownloadEvent::SendFileContentsRequest(Event::Clipboard(
        ClipboardMessage::SendFileContentsRequest(request),
      )));
    }
  }

  pub fn download_chunk(&mut self, stream_id: u32, data: &[u8]) -> std::io::Result<ClipboardEvent> {
    if !self.inflight {
      return Err(std::io::Error::other(
        "received file chunk without an inflight request",
      ));
    }

    let Some(task) = self.current.as_mut() else {
      return Err(std::io::Error::other(
        "received file chunk without an active file",
      ));
    };

    if task.index != stream_id {
      return Err(std::io::Error::other(format!(
        "file chunk stream mismatch: expected {}, got {}",
        task.index, stream_id
      )));
    }

    if data.is_empty() {
      return Err(std::io::Error::other("remote returned an empty chunk"));
    }

    let take = data.len().min((task.size - task.received) as usize);
    if let Some(file) = task.file.as_mut() {
      file.write_all(&data[..take])?;
    }

    task.received += take as u64;

    self.inflight = false;
    Ok(ClipboardEvent::File(FileDownloadStatus::Progress {
      id: task.index,
      total_size: task.size,
      received: task.received,
    }))
  }

  pub fn abort_current(&mut self, stream_id: u32) -> bool {
    if self.inflight
      && self
        .current
        .as_ref()
        .is_some_and(|task| task.index == stream_id)
    {
      self.current = None;
      self.inflight = false;
      return true;
    }

    false
  }

  pub fn request_file(
    &mut self,
    temp_dir: &str,
    files: &[FileDescriptor],
  ) -> Option<ClipboardEvent> {
    self.clear();

    let timestamp = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_millis();
    let base_dir = PathBuf::from(temp_dir).join(format!("rdp_drop_{}", timestamp));

    if let Err(error) = std::fs::create_dir_all(&base_dir) {
      tracing::error!(?error, "Failed to create clipboard temp dir");
      return None;
    }

    for (index, fd) in files.iter().enumerate().rev() {
      let is_dir = fd
        .attributes
        .map(|attributes| attributes.contains(ClipboardFileAttributes::DIRECTORY))
        .unwrap_or(false);

      let normalized_name = fd.name.replace('\\', "/");
      let raw_path = PathBuf::from(&normalized_name);

      let mut safe_rel_path = PathBuf::new();
      for component in raw_path.components() {
        if let std::path::Component::Normal(c) = component {
          safe_rel_path.push(c);
        }
      }

      if safe_rel_path.as_os_str().is_empty() {
        continue;
      }

      let path = base_dir.join(&safe_rel_path);
      if is_dir {
        if let Err(error) = std::fs::create_dir_all(&path) {
          tracing::error!(?error, "Failed to create directory");
        }

        self.downloaded.push(path);
        continue;
      } else {
        if let Some(parent) = path.parent() {
          if let Err(error) = std::fs::create_dir_all(parent) {
            tracing::error!(?error, "Failed to create directory");
          }
        }
      }

      let size = fd.file_size.unwrap_or(0);

      self.queue.push(DownloadTask {
        path,
        size,
        file: None,
        received: 0,
        index: index as u32,
      });

      return Some(ClipboardEvent::File(FileDownloadStatus::Download {
        id: index as u32,
        total_size: size,
      }));
    }

    None
  }

  pub fn is_idle(&self) -> bool {
    self.current.is_none() && self.queue.is_empty()
  }
}
