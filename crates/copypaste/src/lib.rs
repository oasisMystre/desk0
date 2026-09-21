use std::{
  borrow::Cow,
  fmt::Debug,
  fs::File,
  io::{Read, Seek, SeekFrom, Write},
  path::PathBuf,
  sync::{Arc, Mutex},
  time::{SystemTime, UNIX_EPOCH},
};

use clipboard_rs::{
  Clipboard, ClipboardContext, ClipboardHandler, RustImageData, common::RustImage,
};
use image::{ColorType, ImageFormat};
#[cfg(feature = "server")]
use ironrdp::server::{CliprdrServerFactory, ServerEvent, ServerEventSender};
use ironrdp::{
  cliprdr::{
    backend::{ClipboardMessage, CliprdrBackend, CliprdrBackendFactory},
    pdu::{
      ClipboardFileAttributes, ClipboardFormat, ClipboardFormatId, ClipboardFormatName,
      ClipboardGeneralCapabilityFlags, FileContentsFlags, FileContentsRequest,
      FileContentsResponse, FileDescriptor, FormatDataRequest, FormatDataResponse, LockDataId,
      OwnedFormatDataResponse, PackedFileList,
    },
  },
  core::impl_as_any,
};
#[cfg(not(feature = "server"))]
use ironrdp_client::rdp::RdpInputEvent;
use tokio::sync::mpsc::UnboundedSender;

use crate::types::{ClipboardFile, ClipboardState, DownloadState, DownloadTask};

mod types;

const CF_RTF: u32 = 0xC001;
const CF_HTML: u32 = 0xC002;
const MAX_CLIPBOARD_CHUNK: u32 = 1024 * 1024;

#[cfg(feature = "server")]
pub type Event = ServerEvent;
#[cfg(not(feature = "server"))]
pub type Event = RdpInputEvent;

#[derive(Clone)]
pub struct CliprdrHandler {
  sender: Option<UnboundedSender<Event>>,
}

struct CliprdrHandlerBackend {
  temp_dir: String,
  clipboard: ClipboardContext,
  download_state: DownloadState,
  state: Arc<Mutex<ClipboardState>>,
  sender: UnboundedSender<Event>,
  capabilities: ClipboardGeneralCapabilityFlags,
}

impl Debug for CliprdrHandlerBackend {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CliprdrHandlerBackend")
      .field("sender", &self.sender)
      .finish()
  }
}

#[cfg(feature = "server")]
impl ServerEventSender for CliprdrHandler {
  fn set_sender(&mut self, sender: UnboundedSender<ServerEvent>) {
    self.sender = Some(sender);
  }
}

impl CliprdrBackendFactory for CliprdrHandler {
  fn build_cliprdr_backend(&self) -> Box<dyn CliprdrBackend> {
    let sender = self
      .sender
      .clone()
      .expect("sender must be set before building backend");

    let temp_dir = std::env::temp_dir()
      .join("cliprdr")
      .to_string_lossy()
      .into_owned();

    Box::new(CliprdrHandlerBackend {
      sender,
      temp_dir,
      download_state: DownloadState::default(),
      capabilities: ClipboardGeneralCapabilityFlags::empty(),
      state: Arc::new(Mutex::new(ClipboardState::default())),
      clipboard: ClipboardContext::new().expect("creating clipboard instance failed"),
    })
  }
}

#[cfg(feature = "server")]
impl CliprdrServerFactory for CliprdrHandler {}

impl_as_any!(CliprdrHandlerBackend);

impl ClipboardHandler for CliprdrHandlerBackend {
  fn on_clipboard_change(&mut self) {
    {
      let mut state = self.state.lock().unwrap();

      if !state.active_locks.is_empty() {
        return;
      }

      state.clear();

      if let Ok(files) = self.clipboard.get_files() {
        state.files.extend(files.into_iter().map(|file| {
          let path = std::path::Path::new(&file);
          let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned());
          let metadata = path.symlink_metadata().ok();
          let size = metadata.as_ref().map(|metadata| metadata.len());
          let modified = metadata
            .as_ref()
            .and_then(|metadata| metadata.modified().ok());

          ClipboardFile {
            name,
            size,
            modified,
            path: path.to_path_buf(),
          }
        }));
      }

      if let Ok(image) = self.clipboard.get_image() {
        match image.encode_image(ColorType::Rgb8, ImageFormat::Bmp) {
          Ok(bytes) => {
            let bytes = bytes.get_bytes();
            let data = &bytes[14..];
            state.image = Some(data.to_vec());
          }
          Err(error) => {
            tracing::error!(?error, "Failed to encode image");
          }
        };
      }

      if let Ok(text) = self.clipboard.get_text() {
        state.text = Some(text);
      }

      if let Ok(html) = self.clipboard.get_html() {
        state.html = Some(html);
      }

      if let Ok(rtf) = self.clipboard.get_rich_text() {
        state.rich_text = Some(rtf);
      }
    }

    self.send_format_list();
  }
}

impl CliprdrHandlerBackend {
  fn send_format_list(&self) {
    let state = self.state.lock().unwrap();

    let mut formats = Vec::new();

    if state.text.is_some() {
      formats.push(ClipboardFormat::new(ClipboardFormatId::CF_UNICODETEXT));
    }

    if state.rich_text.is_some() {
      let mut format = ClipboardFormat::new(ClipboardFormatId::new(CF_RTF));
      format.name = Some(ClipboardFormatName::new_static("Rich Text Format"));
      formats.push(format);
    }

    if state.html.is_some() {
      let mut format = ClipboardFormat::new(ClipboardFormatId::new(CF_HTML));
      format.name = Some(ClipboardFormatName::new_static("HTML Format"));
      formats.push(format);
    }

    if state.image.is_some() {
      formats.push(ClipboardFormat::new(ClipboardFormatId::CF_DIB));
    }

    if !state.files.is_empty() {
      formats.push(ClipboardFormat::new(ClipboardFormatId::CF_HDROP));
    }

    if !formats.is_empty() {
      if let Err(error) = self
        .sender
        .send(Event::Clipboard(ClipboardMessage::SendInitiateCopy(
          formats,
        )))
      {
        eprintln!("Failed to send initiate copy event {error}");
      };
    }
  }

  fn send_text(&self, state: &mut ClipboardState) {
    if let Some(text) = &state.text {
      if let Err(error) = self
        .sender
        .send(Event::Clipboard(ClipboardMessage::SendFormatData(
          OwnedFormatDataResponse::new_unicode_string(&text),
        )))
      {
        eprintln!("Failed to send text clipboard event {error}");
      }
    }
  }

  fn send_html(&self, state: &mut ClipboardState) {
    if let Some(html) = &state.html {
      if let Err(error) = self
        .sender
        .send(Event::Clipboard(ClipboardMessage::SendFormatData(
          OwnedFormatDataResponse::new_data(Cow::Owned(html.as_bytes().to_vec())),
        )))
      {
        eprintln!("Failed to send html clipboard event {error}");
      }
    }
  }

  fn send_rich_text(&self, state: &mut ClipboardState) {
    if let Some(rich_text) = &state.rich_text {
      if let Err(error) = self
        .sender
        .send(Event::Clipboard(ClipboardMessage::SendFormatData(
          OwnedFormatDataResponse::new_data(Cow::Owned(rich_text.as_bytes().to_vec())),
        )))
      {
        eprintln!("Failed to send rich_text clipboard event {error}");
      }
    }
  }

  fn send_file_list(&self, state: &mut ClipboardState) {
    if !state.files.is_empty() {
      let files = match state.file_descriptors() {
        Ok(files) => files,
        Err(error) => {
          eprintln!("reading clipboard files failed {error}");
          return;
        }
      };

      if let Err(error) = self
        .sender
        .send(Event::Clipboard(ClipboardMessage::SendFormatData(
          OwnedFormatDataResponse::new_file_list(&PackedFileList { files }).unwrap(),
        )))
      {
        eprintln!("failed to send file lists {error}");
      }
    }
  }

  fn send_image(&self, state: &mut ClipboardState) {
    if let Some(image) = &state.image {
      if let Err(error) = self
        .sender
        .send(Event::Clipboard(ClipboardMessage::SendFormatData(
          OwnedFormatDataResponse::new_data(Cow::Owned(image.clone())),
        )))
      {
        eprintln!("failed to send file lists {error}");
      }
    }
  }

  fn request_next_download_chunk(&mut self) {
    if self.download_state.current.is_none() {
      if let Some(mut task) = self.download_state.queue.pop() {
        if task.size == 0 {
          if let Err(error) = File::create(&task.path) {
            tracing::warn!(?error, "Failed to create local file for download");
          } else {
            self.download_state.downloaded.push(task.path);
          }

          self.request_next_download_chunk();
          return;
        }

        if let Ok(file) = File::create(&task.path) {
          task.file = Some(file);
          self.download_state.current = Some(task);
        } else {
          tracing::error!(?task.path,"Failed to create local file for download");
          self.request_next_download_chunk();
          return;
        }
      } else {
        if !self.download_state.downloaded.is_empty() {
          let paths: Vec<String> = self
            .download_state
            .downloaded
            .iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect();

          if let Err(error) = self.clipboard.set_files(paths) {
            tracing::error!(?error, "Failed to set downloaded files to clipboard");
          }

          self.download_state.downloaded.clear();
        }
      }
    }

    if let Some(task) = &self.download_state.current {
      let remaining = task.size.saturating_sub(task.received);
      if remaining == 0 {
        if let Some(task) = self.download_state.current.take() {
          self.download_state.downloaded.push(task.path);
        }

        self.request_next_download_chunk();
        return;
      }

      let chunk_size = remaining.min(MAX_CLIPBOARD_CHUNK as u64) as u32;
      if let Err(error) =
        self
          .sender
          .send(Event::Clipboard(ClipboardMessage::SendFileContentsRequest(
            FileContentsRequest {
              data_id: None,
              stream_id: task.index,
              index: task.index as i32,
              position: task.received,
              requested_size: chunk_size,
              flags: FileContentsFlags::RANGE,
            },
          )))
      {
        tracing::error!(?error, "Failed to send file contents request");
      }
    }
  }
}

impl CliprdrBackend for CliprdrHandlerBackend {
  fn temporary_directory(&self) -> &str {
    &self.temp_dir
  }

  fn client_capabilities(&self) -> ClipboardGeneralCapabilityFlags {
    ClipboardGeneralCapabilityFlags::USE_LONG_FORMAT_NAMES
      | ClipboardGeneralCapabilityFlags::STREAM_FILECLIP_ENABLED
      | ClipboardGeneralCapabilityFlags::FILECLIP_NO_FILE_PATHS
  }

  fn on_ready(&mut self) {
    self.on_clipboard_change();
  }

  fn on_request_format_list(&mut self) {
    self.send_format_list();
  }

  fn on_process_negotiated_capabilities(&mut self, capabilities: ClipboardGeneralCapabilityFlags) {
    self.capabilities = capabilities;
    tracing::debug!(?capabilities, "Negotiated cliprdr capabilities");
  }

  fn on_remote_copy(&mut self, available_formats: &[ClipboardFormat]) {
    tracing::debug!(?available_formats, "Remote copied formats");

    let mut format_to_request = None;
    for format in available_formats {
      match format.id {
        ClipboardFormatId::CF_HDROP => {
          format_to_request = Some(format.id);
          break;
        }
        ClipboardFormatId::CF_DIB => {
          format_to_request = Some(format.id);
          break;
        }
        ClipboardFormatId::CF_UNICODETEXT => {
          if format_to_request.is_none() {
            format_to_request = Some(format.id);
          }
        }
        ClipboardFormatId(CF_RTF) => {
          if format_to_request.is_none() {
            format_to_request = Some(format.id);
          }
        }
        ClipboardFormatId(CF_HTML) => {
          if format_to_request.is_none() {
            format_to_request = Some(format.id);
          }
        }
        _ => {}
      }
    }

    if let Some(format_id) = format_to_request {
      if let Err(error) = self
        .sender
        .send(Event::Clipboard(ClipboardMessage::SendInitiatePaste(
          format_id,
        )))
      {
        eprintln!("Failed to initiate paste from remote {error}");
      }
    }
  }

  fn on_format_data_request(&mut self, request: FormatDataRequest) {
    let mut state = self.state.lock().unwrap();

    match request.format {
      ClipboardFormatId::CF_UNICODETEXT => {
        self.send_text(&mut state);
      }
      ClipboardFormatId::CF_HDROP => {
        self.send_file_list(&mut state);
      }
      ClipboardFormatId::CF_DIB => {
        self.send_image(&mut state);
      }
      ClipboardFormatId(CF_RTF) => {
        self.send_rich_text(&mut state);
      }
      ClipboardFormatId(CF_HTML) => {
        self.send_html(&mut state);
      }
      _ => {}
    }
  }

  fn on_format_data_response(&mut self, response: FormatDataResponse<'_>) {
    if let Ok(text) = response.to_string() {
      if !text.is_empty() {
        if let Err(error) = self.clipboard.set_text(text) {
          eprintln!("Failed to paste unicode string {error}");
        }

        return;
      }
    }

    let raw_data = response.data();
    if raw_data.is_empty() {
      return;
    }

    if let Ok(str_data) = std::str::from_utf8(raw_data) {
      if str_data.starts_with("Version:") || str_data.contains("<html") {
        if let Err(error) = self.clipboard.set_html(str_data.to_string()) {
          eprintln!("Failed to paste HTML {error}");
        }

        return;
      }

      if str_data.starts_with("{\\rtf1") {
        if let Err(error) = self.clipboard.set_rich_text(str_data.to_string()) {
          eprintln!("Failed to paste rich text {error}");
        }

        return;
      }
    }

    if raw_data.len() >= 40 {
      let header_size = u32::from_le_bytes(raw_data[0..4].try_into().unwrap_or([0u8; 4])) as usize;
      if header_size == 40 || header_size == 108 || header_size == 124 {
        let bit_count = if raw_data.len() >= 16 {
          u16::from_le_bytes(raw_data[14..16].try_into().unwrap_or([0u8; 2]))
        } else {
          0
        };
        let clr_used = if raw_data.len() >= 36 {
          u32::from_le_bytes(raw_data[32..36].try_into().unwrap_or([0u8; 4]))
        } else {
          0
        };
        let num_colors = if clr_used > 0 {
          clr_used as usize
        } else if bit_count <= 8 && bit_count > 0 {
          1 << bit_count
        } else {
          0
        };
        let off_bits = (14 + header_size + num_colors * 4) as u32;

        let mut bmp = Vec::with_capacity(14 + raw_data.len());
        bmp.extend_from_slice(b"BM");
        let file_size = (14 + raw_data.len()) as u32;
        bmp.extend_from_slice(&file_size.to_le_bytes());
        bmp.extend_from_slice(&[0u8; 4]);
        bmp.extend_from_slice(&off_bits.to_le_bytes());
        bmp.extend_from_slice(raw_data);

        if let Ok(image) = image::load_from_memory_with_format(&bmp, image::ImageFormat::Bmp) {
          let rust_image = RustImageData::from_dynamic_image(image);
          if let Err(error) = self.clipboard.set_image(rust_image) {
            eprintln!("Failed to paste image {error}");
          }
        }
        return;
      }
    }
  }

  fn on_file_contents_request(&mut self, request: FileContentsRequest) {
    let path = {
      let state = self.state.lock().unwrap();
      let file_index = request.index;

      let Some(file_info) = state.files.get(file_index as usize) else {
        eprintln!("invalid clipboard file index={}", request.index);
        return;
      };

      file_info.path.clone()
    };

    let result = (|| -> std::io::Result<Vec<u8>> {
      let mut file = File::open(&path)?;
      file.seek(SeekFrom::Start(request.position))?;

      let mut buffer = vec![0; request.requested_size.min(MAX_CLIPBOARD_CHUNK) as usize];
      let bytes_read = file.read(&mut buffer)?;
      buffer.truncate(bytes_read);

      Ok(buffer)
    })();

    match result {
      Ok(buffer) => {
        if let Err(error) = self.sender.send(Event::Clipboard(
          ClipboardMessage::SendFileContentsResponse(FileContentsResponse::new_data_response(
            request.stream_id,
            Cow::Owned(buffer),
          )),
        )) {
          eprintln!("Failed to send file content response {error}");
        }
      }
      Err(error) => {
        tracing::error!(
          path = ?path.display(),
          ?error,
          "failed to read clipboard file"
        );
        if let Err(error) = self.sender.send(Event::Clipboard(
          ClipboardMessage::SendFileContentsResponse(FileContentsResponse::new_error(
            request.stream_id,
          )),
        )) {
          eprintln!("Failed to send file content response {error}");
        }
      }
    }
  }

  fn on_file_contents_response<'a>(&mut self, response: FileContentsResponse<'a>) {
    if let Some(task) = &mut self.download_state.current {
      if response.stream_id() != task.index {
        return;
      }

      let chunk_data = response.data();

      if chunk_data.is_empty() && task.size > 0 {
        tracing::error!("Received empty chunk for file download, aborting task");
        self.download_state.current = None;
      } else if let Some(ref mut file) = task.file {
        match file.write_all(chunk_data) {
          Ok(()) => {
            task.received += chunk_data.len() as u64;
          }
          Err(error) => {
            tracing::error!(?error, "file write error, aborting download task");
            self.download_state.current = None;
          }
        }
      }
    }

    self.request_next_download_chunk();
  }

  fn on_remote_file_list(&mut self, files: &[FileDescriptor], _: Option<u32>) {
    self.download_state.clear();
    let timestamp = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_millis();
    let base_dir =
      PathBuf::from(self.temporary_directory()).join(format!("rdp_drop_{}", timestamp));

    if let Err(error) = std::fs::create_dir_all(&base_dir) {
      tracing::error!(?error, "Failed to create clipboard temp dir");
      return;
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

        self.download_state.downloaded.push(path);
        continue;
      } else {
        if let Some(parent) = path.parent() {
          if let Err(error) = std::fs::create_dir_all(&parent) {
            tracing::error!(?error, "Failed to create directory");
          }
        }
      }

      self.download_state.queue.push(DownloadTask {
        path,
        file: None,
        received: 0,
        index: index as u32,
        size: fd.file_size.unwrap_or(0),
      });
    }

    self.request_next_download_chunk();
  }

  fn on_lock(&mut self, data_id: LockDataId) {
    tracing::debug!(?data_id, "Clipboard lock requested");
    let mut state = self.state.lock().unwrap();
    state.active_locks.push(data_id);
  }

  fn on_unlock(&mut self, data_id: LockDataId) {
    tracing::debug!(?data_id, "Clipboard unlock requested");
    let mut state = self.state.lock().unwrap();
    state.active_locks.retain(|id| id != &data_id);
  }

  fn on_outgoing_locks_cleared(&mut self, _: &[LockDataId]) {
    self.download_state.queue.clear();
    self.download_state.current = None;
  }
}

impl CliprdrHandler {
  pub fn new() -> Self {
    Self { sender: None }
  }
}
