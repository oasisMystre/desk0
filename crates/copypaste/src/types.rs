use std::{
  fs::File,
  path::PathBuf,
  time::{SystemTime, UNIX_EPOCH},
};

use ironrdp::cliprdr::pdu::{ClipboardFileAttributes, FileDescriptor, LockDataId};

pub struct ClipboardFile {
  pub path: PathBuf,
  #[allow(dead_code)]
  pub name: Option<String>,
  pub size: Option<u64>,
  pub modified: Option<SystemTime>,
}

#[derive(Default)]
pub struct ClipboardState {
  pub text: Option<String>,
  pub html: Option<String>,
  pub rich_text: Option<String>,
  pub files: Vec<ClipboardFile>,
  pub image: Option<Vec<u8>>,

  pub active_locks: Vec<LockDataId>,
}

pub struct DownloadTask {
  pub index: u32,
  pub path: PathBuf,
  pub size: u64,
  pub received: u64,
  pub file: Option<File>,
}

#[derive(Default)]
pub struct DownloadState {
  pub queue: Vec<DownloadTask>,
  pub downloaded: Vec<PathBuf>,
  pub current: Option<DownloadTask>,
}

impl ClipboardState {
  pub const UNIX_TO_FILETIME: u64 = 116_444_736_000_000_000;

  pub fn clear(&mut self) {
    self.text = None;
    self.html = None;
    self.rich_text = None;
    self.files.clear();
  }

  pub fn file_descriptors(&self) -> anyhow::Result<Vec<FileDescriptor>> {
    let file_descriptors = self
      .files
      .iter()
      .map(|file| {
        let name = file
          .name
          .clone()
          .or_else(|| {
            file
              .path
              .file_name()
              .map(|name| name.to_string_lossy().into_owned())
          })
          .unwrap_or_else(|| "file".to_string());

        let mut fd = FileDescriptor::new(name);
        if let Some(file_size) = file.size {
          fd = fd.with_file_size(file_size);
        }

        if let Some(modified) = file.modified {
          if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
            let intervals =
              duration.as_secs() * 10_000_000 + (duration.subsec_nanos() as u64 / 100);
            fd = fd.with_last_write_time(intervals + Self::UNIX_TO_FILETIME);
          }
        }

        let mut attributes = ClipboardFileAttributes::empty();

        if let Ok(metadata) = file.path.metadata() {
          if metadata.is_dir() {
            attributes |= ClipboardFileAttributes::DIRECTORY;
          }

          if metadata.permissions().readonly() {
            attributes |= ClipboardFileAttributes::READONLY;
          }
        }

        if attributes.is_empty() {
          attributes |= ClipboardFileAttributes::NORMAL;
        }

        return fd.with_attributes(attributes);
      })
      .collect();

    Ok(file_descriptors)
  }
}

impl DownloadState {
  pub fn clear(&mut self) {
    self.queue.clear();
    self.current = None;
    self.downloaded.clear();
  }
}
