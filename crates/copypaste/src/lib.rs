use std::{
  borrow::Cow,
  fmt::Debug,
  fs::File,
  io::{Read, Seek, SeekFrom, Write},
  path::PathBuf,
  sync::{Arc, Mutex},
  thread::JoinHandle,
  time::{SystemTime, UNIX_EPOCH},
};

use clipboard_rs::{
  Clipboard, ClipboardContext, ClipboardHandler, ClipboardWatcher, ClipboardWatcherContext,
  RustImageData, WatcherShutdown, common::RustImage,
};
use image::ImageFormat;
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
use ironrdp_client::rdp::{RdpInputEvent, RdpInputSender};
use ironrdp_cliprdr_format::{
  bitmap::{dib_to_png, dibv5_to_png, png_to_cf_dibv5},
  html::{cf_html_to_plain_html, plain_html_to_cf_html},
};
#[cfg(not(feature = "server"))]
use tokio::sync::mpsc::error::TrySendError;
#[cfg(feature = "server")]
use tokio::sync::mpsc::{UnboundedSender, error::SendError};

use crate::types::{ClipboardFile, ClipboardState, DownloadState, DownloadTask};

mod types;

const CF_RTF: u32 = 0xC001;
const CF_HTML: u32 = 0xC002;
const MAX_CLIPBOARD_CHUNK: u32 = 1024 * 1024;

#[cfg(feature = "server")]
pub type Event = ServerEvent;
#[cfg(not(feature = "server"))]
pub type Event = RdpInputEvent;

struct WatcherState {
  thread: Option<JoinHandle<()>>,
  shutdown: Option<WatcherShutdown>,
}

#[derive(Clone, Copy)]
pub enum CliprdrState {
  Copy,
  Ready,
  Paste,
  Initialize,
}

impl CliprdrState {
  pub fn allow_next_event(&self) -> bool {
    match self {
      CliprdrState::Paste | CliprdrState::Copy => false,
      CliprdrState::Ready | CliprdrState::Initialize => true,
    }
  }
}

#[derive(Clone)]
pub struct CliprdrHandler {
  state: Arc<Mutex<CliprdrState>>,
  clipboard: Arc<ClipboardContext>,
  watcher: Arc<Mutex<WatcherState>>,
  #[cfg(feature = "server")]
  sender: Option<UnboundedSender<Event>>,
  #[cfg(not(feature = "server"))]
  sender: Option<RdpInputSender>,
  download_state: Arc<Mutex<DownloadState>>,
  clipboard_state: Arc<Mutex<ClipboardState>>,
}

struct CliprdrHandlerBackend {
  temp_dir: String,
  handler: CliprdrHandler,
  clipboard: Arc<ClipboardContext>,
  capabilities: ClipboardGeneralCapabilityFlags,
}

impl Debug for CliprdrHandlerBackend {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CliprdrHandlerBackend")
      .field("temp_dir", &self.temp_dir)
      .field("capabilites", &self.capabilities)
      .finish()
  }
}

impl Drop for CliprdrHandlerBackend {
  fn drop(&mut self) {
    self.handler.set_state(CliprdrState::Initialize);
  }
}

#[cfg(feature = "server")]
impl ServerEventSender for CliprdrHandler {
  fn set_sender(&mut self, sender: UnboundedSender<Event>) {
    self.sender = Some(sender);
    self.start_watcher();
  }
}

#[cfg(not(feature = "server"))]
impl CliprdrHandler {
  pub fn set_sender(&mut self, sender: RdpInputSender) {
    self.sender = Some(sender);
    self.start_watcher();
  }
}

impl CliprdrBackendFactory for CliprdrHandler {
  fn build_cliprdr_backend(&self) -> Box<dyn CliprdrBackend> {
    Box::new(self.build_backend())
  }
}

impl CliprdrHandler {
  fn build_backend(&self) -> CliprdrHandlerBackend {
    self.set_state(CliprdrState::Initialize);

    let temp_dir = std::env::temp_dir()
      .join("cliprdr")
      .to_string_lossy()
      .into_owned();

    CliprdrHandlerBackend {
      temp_dir,
      handler: self.clone(),
      clipboard: self.clipboard.clone(),
      capabilities: ClipboardGeneralCapabilityFlags::empty(),
    }
  }
}

#[cfg(feature = "server")]
impl CliprdrServerFactory for CliprdrHandler {}

impl_as_any!(CliprdrHandlerBackend);

impl CliprdrBackend for CliprdrHandlerBackend {
  fn temporary_directory(&self) -> &str {
    &self.temp_dir
  }

  fn client_capabilities(&self) -> ClipboardGeneralCapabilityFlags {
    ClipboardGeneralCapabilityFlags::USE_LONG_FORMAT_NAMES
      | ClipboardGeneralCapabilityFlags::STREAM_FILECLIP_ENABLED
      | ClipboardGeneralCapabilityFlags::FILECLIP_NO_FILE_PATHS
      | ClipboardGeneralCapabilityFlags::CAN_LOCK_CLIPDATA
      | ClipboardGeneralCapabilityFlags::HUGE_FILE_SUPPORT_ENABLED
  }

  fn on_ready(&mut self) {
    self.handler.set_state(CliprdrState::Ready);
    self.handler.start_watcher();
  }

  fn on_request_format_list(&mut self) {
    self.handler.send_format_list();
  }

  fn on_process_negotiated_capabilities(&mut self, capabilities: ClipboardGeneralCapabilityFlags) {
    self.capabilities = capabilities;
  }

  fn on_remote_copy(&mut self, available_formats: &[ClipboardFormat]) {
    let mut format_to_request = None;
    for format in available_formats {
      if format
        .name
        .as_ref()
        .map(|name| name.value() == ClipboardFormatName::FILE_LIST.value())
        .unwrap_or(false)
      {
        format_to_request = Some(format.id);
        break;
      }

      match format.id {
        ClipboardFormatId::CF_HDROP => {
          format_to_request = Some(format.id);
          break;
        }
        ClipboardFormatId::CF_DIB => {
          format_to_request = Some(format.id);
          break;
        }
        ClipboardFormatId::CF_DIBV5 => {
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
        .handler
        .send(Event::Clipboard(ClipboardMessage::SendInitiatePaste(
          format_id,
        )))
      {
        tracing::error!(?error, "Failed to initiate paste from remote");
      }
    }
  }

  fn on_format_data_request(&mut self, request: FormatDataRequest) {
    let mut state = self.handler.clipboard_state.lock().unwrap();

    match request.format {
      ClipboardFormatId::CF_UNICODETEXT => {
        self.handler.send_text(&mut state);
      }
      ClipboardFormatId::CF_HDROP => {
        self.handler.send_file_list(&mut state);
      }
      ClipboardFormatId::CF_DIB => {
        self.handler.send_image(&mut state);
      }
      ClipboardFormatId::CF_DIBV5 => {
        self.handler.send_image(&mut state);
      }
      ClipboardFormatId(CF_RTF) => {
        self.handler.send_rich_text(&mut state);
      }
      ClipboardFormatId(CF_HTML) => {
        self.handler.send_html(&mut state);
      }
      _ => {}
    }

    self.handler.set_state(CliprdrState::Copy);
  }

  fn on_format_data_response(&mut self, response: FormatDataResponse<'_>) {
    self.handler.set_state(CliprdrState::Paste);

    if let Ok(text) = response.to_string() {
      if !text.is_empty() {
        if let Err(error) = self.clipboard.set_text(text) {
          tracing::error!(?error, "Failed to paste unicode string");
        }

        return;
      }
    }

    let data = response.data();
    if data.is_empty() {
      return;
    }

    if let Ok(data) = cf_html_to_plain_html(data) {
      if let Err(error) = self.clipboard.set_html(data.to_string()) {
        tracing::error!(?error, "Failed to paste HTML");
      }

      return;
    }

    if let Ok(data) = std::str::from_utf8(data) {
      if data.starts_with("{\\rtf1") {
        if let Err(error) = self.clipboard.set_rich_text(data.to_string()) {
          tracing::error!(?error, "Failed to paste rich text");
        }

        return;
      }
    }

    if let Ok(data) = dibv5_to_png(data) {
      if let Ok(image) = image::load_from_memory_with_format(&data, ImageFormat::Png) {
        let rust_image = RustImageData::from_dynamic_image(image);
        if let Err(error) = self.clipboard.set_image(rust_image) {
          tracing::error!(?error, "Failed to paste image");
        }
      }
      return;
    }

    if let Ok(data) = dib_to_png(data) {
      if let Ok(image) = image::load_from_memory_with_format(&data, ImageFormat::Png) {
        let rust_image = RustImageData::from_dynamic_image(image);
        if let Err(error) = self.clipboard.set_image(rust_image) {
          tracing::error!(?error, "Failed to paste image");
        }
      }
      return;
    }
  }

  fn on_file_contents_request(&mut self, request: FileContentsRequest) {
    let path = {
      let state = self.handler.clipboard_state.lock().unwrap();
      let file_index = request.index;

      let Some(file_info) = state.files.get(file_index as usize) else {
        tracing::error!(index=?request.index, "invalid clipboard file index");
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
        if let Err(error) = self.handler.send(Event::Clipboard(
          ClipboardMessage::SendFileContentsResponse(FileContentsResponse::new_data_response(
            request.stream_id,
            Cow::Owned(buffer),
          )),
        )) {
          tracing::error!(?error, "Failed to send file content response");
        }
      }
      Err(error) => {
        tracing::error!(
          path = ?path.display(),
          ?error,
          "failed to read clipboard file"
        );

        if let Err(error) = self.handler.send(Event::Clipboard(
          ClipboardMessage::SendFileContentsResponse(FileContentsResponse::new_error(
            request.stream_id,
          )),
        )) {
          tracing::error!(?error, "Failed to send file content response");
        }
      }
    }
  }

  fn on_file_contents_response<'a>(&mut self, response: FileContentsResponse<'a>) {
    let mut download_state = self.handler.download_state.lock().unwrap();

    if let Some(task) = &mut download_state.current {
      if response.stream_id() != task.index {
        return;
      }

      let chunk_data = response.data();

      if chunk_data.is_empty() && task.size > 0 {
        download_state.current = None;
        tracing::error!("Received empty chunk for file download, aborting task");
      } else if let Some(ref mut file) = task.file {
        match file.write_all(chunk_data) {
          Ok(()) => {
            task.received += chunk_data.len() as u64;
          }
          Err(error) => {
            tracing::error!(?error, "file write error, aborting download task");
            download_state.current = None;
          }
        }
      }
    }

    drop(download_state);
    self.handler.request_next_download_chunk();
  }

  fn on_remote_file_list(&mut self, files: &[FileDescriptor], _: Option<u32>) {
    self.handler.download_state.lock().unwrap().clear();
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

        self
          .handler
          .download_state
          .lock()
          .unwrap()
          .downloaded
          .push(path);

        continue;
      } else {
        if let Some(parent) = path.parent() {
          if let Err(error) = std::fs::create_dir_all(&parent) {
            tracing::error!(?error, "Failed to create directory");
          }
        }
      }

      self
        .handler
        .download_state
        .lock()
        .unwrap()
        .queue
        .push(DownloadTask {
          path,
          file: None,
          received: 0,
          index: index as u32,
          size: fd.file_size.unwrap_or(0),
        });
    }

    self.handler.request_next_download_chunk();
  }

  fn on_lock(&mut self, data_id: LockDataId) {
    let mut state = self.handler.clipboard_state.lock().unwrap();
    state.active_locks.push(data_id);
    drop(state);
  }

  fn on_unlock(&mut self, data_id: LockDataId) {
    let mut state = self.handler.clipboard_state.lock().unwrap();
    state.active_locks.retain(|id| id != &data_id);
    drop(state);
  }

  fn on_outgoing_locks_cleared(&mut self, data_ids: &[LockDataId]) {
    let mut download_state = self.handler.download_state.lock().unwrap();
    let mut clipboard_state = self.handler.clipboard_state.lock().unwrap();

    download_state.queue.clear();
    download_state.current = None;

    clipboard_state
      .active_locks
      .retain(|id| !data_ids.contains(id));

    drop(download_state);
    drop(clipboard_state);
  }
}

impl ClipboardHandler for CliprdrHandler {
  fn on_clipboard_change(&mut self) {
    let state = self.get_state();
    if !state.allow_next_event() {
      self.set_state(CliprdrState::Ready);
      return;
    }

    #[cfg(feature = "server")]
    {
      let state = self.get_state();
      if !matches!(state, CliprdrState::Ready) {
        return;
      }
    }

    let mut state = self.clipboard_state.lock().unwrap();
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

    if !state.files.is_empty() {
      drop(state);
      self.send_format_list();
      return;
    }

    if let Ok(image) = self.clipboard.get_image() {
      match image.to_png() {
        Ok(bytes) => {
          let bytes = bytes.get_bytes();
          match png_to_cf_dibv5(bytes) {
            Ok(bytes) => state.image = Some(bytes),
            Err(error) => tracing::error!(?error, "failed to convert png to cf dibv5"),
          }
        }
        Err(error) => {
          tracing::error!(?error, "Failed to encode image");
        }
      };
    }

    if let Ok(html) = self.clipboard.get_html() {
      state.html = Some(plain_html_to_cf_html(&html));
    }

    if let Ok(rtf) = self.clipboard.get_rich_text() {
      state.rich_text = Some(rtf);
    }

    if let Ok(text) = self.clipboard.get_text() {
      state.text = Some(text);
    }

    drop(state);
    self.send_format_list();
  }
}

impl CliprdrHandler {
  pub fn new() -> Self {
    Self {
      sender: None,
      state: Arc::new(Mutex::new(CliprdrState::Initialize)),
      download_state: Arc::new(Mutex::new(DownloadState::default())),
      clipboard_state: Arc::new(Mutex::new(ClipboardState::default())),
      clipboard: Arc::new(ClipboardContext::new().expect("creating clipboard instance failed")),
      watcher: Arc::new(Mutex::new(WatcherState {
        thread: None,
        shutdown: None,
      })),
    }
  }

  pub fn set_state(&self, state: CliprdrState) {
    let mut lock = self.state.lock().unwrap();
    *lock = state;
    drop(lock);
  }

  pub fn get_state(&self) -> CliprdrState {
    let lock = { self.state.lock().unwrap() };
    *lock
  }

  #[cfg(feature = "server")]
  pub fn send(&self, message: Event) -> Result<(), SendError<Event>> {
    if let Some(ref sender) = self.sender {
      Ok(sender.send(message)?)
    } else {
      panic!("expected sender to be set")
    }
  }

  #[cfg(not(feature = "server"))]

  pub fn send(&self, message: Event) -> Result<(), TrySendError<Event>> {
    if let Some(ref sender) = self.sender {
      Ok(sender.try_send(message)?)
    } else {
      panic!("expected sender to be set")
    }
  }

  pub fn send_format_list(&self) {
    let state = self.clipboard_state.lock().unwrap();

    if !state.files.is_empty() {
      match state.file_descriptors() {
        Ok(files) => {
          if let Err(error) = self.send(Event::Clipboard(ClipboardMessage::SendInitiateFileCopy(
            files,
          ))) {
            tracing::error!(?error, "Failed to send initiate file copy event");
          }
        }
        Err(error) => tracing::error!(?error, "Reading clipboard files failed"),
      }
      return;
    }

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

    if let Err(error) = self.send(Event::Clipboard(ClipboardMessage::SendInitiateCopy(
      formats,
    ))) {
      tracing::error!(?error, "Failed to send initiate copy event");
    }
  }

  pub fn send_text(&self, state: &mut ClipboardState) {
    if let Some(text) = &state.text {
      if let Err(error) = self.send(Event::Clipboard(ClipboardMessage::SendFormatData(
        OwnedFormatDataResponse::new_string(&text),
      ))) {
        tracing::error!(?error, "Failed to send text clipboard event");
      }
    }
  }

  pub fn send_html(&self, state: &mut ClipboardState) {
    if let Some(html) = &state.html {
      if let Err(error) = self.send(Event::Clipboard(ClipboardMessage::SendFormatData(
        OwnedFormatDataResponse::new_data(Cow::Owned(html.as_bytes().to_vec())),
      ))) {
        tracing::error!(?error, "Failed to send html clipboard event");
      }
    }
  }

  pub fn send_rich_text(&self, state: &mut ClipboardState) {
    if let Some(rich_text) = &state.rich_text {
      if let Err(error) = self.send(Event::Clipboard(ClipboardMessage::SendFormatData(
        OwnedFormatDataResponse::new_data(Cow::Owned(rich_text.as_bytes().to_vec())),
      ))) {
        tracing::error!(?error, "Failed to send rich_text clipboard event");
      }
    }
  }

  pub fn send_file_list(&self, state: &mut ClipboardState) {
    if !state.files.is_empty() {
      let files = match state.file_descriptors() {
        Ok(files) => files,
        Err(error) => {
          tracing::error!(?error, "reading clipboard files failed");
          return;
        }
      };

      if let Err(error) = self.send(Event::Clipboard(ClipboardMessage::SendFormatData(
        OwnedFormatDataResponse::new_file_list(&PackedFileList { files }).unwrap(),
      ))) {
        tracing::error!(?error, "failed to send file lists");
      }
    }
  }

  pub fn send_image(&self, state: &mut ClipboardState) {
    if let Some(image) = &state.image {
      if let Err(error) = self.send(Event::Clipboard(ClipboardMessage::SendFormatData(
        OwnedFormatDataResponse::new_data(Cow::Owned(image.clone())),
      ))) {
        tracing::error!(?error, "failed to send file lists");
      }
    }
  }

  fn request_next_download_chunk(&mut self) {
    let download_state = self.download_state.clone();
    let mut download_state = download_state.lock().unwrap();

    if download_state.current.is_none() {
      if let Some(mut task) = download_state.queue.pop() {
        if task.size == 0 {
          if let Err(error) = File::create(&task.path) {
            tracing::warn!(?error, "Failed to create local file for download");
          } else {
            download_state.downloaded.push(task.path);
          }

          drop(download_state);
          self.request_next_download_chunk();
          return;
        }

        if let Ok(file) = File::create(&task.path) {
          task.file = Some(file);
          download_state.current = Some(task);
        } else {
          tracing::error!(?task.path,"Failed to create local file for download");
          drop(download_state);
          self.request_next_download_chunk();
          return;
        }
      } else {
        if !download_state.downloaded.is_empty() {
          let paths: Vec<String> = download_state
            .downloaded
            .iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect();

          self.set_state(CliprdrState::Paste);
          if let Err(error) = self.clipboard.set_files(paths) {
            self.set_state(CliprdrState::Ready);
            tracing::error!(?error, "Failed to set downloaded files to clipboard");
          } else {
            tracing::info!("Downloaded clipboard files and published them to the local clipboard");
          }

          download_state.downloaded.clear();
          {
            let mut clipboard_state = self.clipboard_state.lock().unwrap();
            clipboard_state.clear();
          }
        }
      }
    }

    if let Some(task) = &download_state.current {
      let remaining = task.size.saturating_sub(task.received);
      if remaining == 0 {
        if let Some(task) = download_state.current.take() {
          download_state.downloaded.push(task.path);
        }

        drop(download_state);
        self.request_next_download_chunk();
        return;
      }

      let chunk_size = remaining.min(MAX_CLIPBOARD_CHUNK as u64) as u32;
      if let Err(error) = self.send(Event::Clipboard(ClipboardMessage::SendFileContentsRequest(
        FileContentsRequest {
          data_id: None,
          stream_id: task.index,
          index: task.index as i32,
          position: task.received,
          requested_size: chunk_size,
          flags: FileContentsFlags::RANGE,
        },
      ))) {
        tracing::error!(?error, "Failed to send file contents request");
      }
    }

    drop(download_state);
  }

  fn start_watcher(&self) {
    if self.watcher.lock().unwrap().thread.is_some() {
      tracing::warn!("Clipboard watcher already started");
      return;
    }

    tracing::info!("Starting clipboard watcher after cliprdr ready");
    let Ok(mut watcher) = ClipboardWatcherContext::new() else {
      tracing::error!("Failed to create clipboard watcher context");
      return;
    };

    let shutdown = watcher.get_shutdown_channel();

    let thread = std::thread::spawn({
      let this = self.clone();

      move || {
        tracing::info!("Clipboard watcher started");
        watcher.add_handler(this);
        watcher.start_watch();

        tracing::warn!("Clipboard watcher stopped");
      }
    });

    let mut watcher = self.watcher.lock().unwrap();
    watcher.thread = Some(thread);
    watcher.shutdown = Some(shutdown);

    drop(watcher);
  }
}

impl Drop for CliprdrHandler {
  fn drop(&mut self) {
    let (shutdown, thread) = {
      let mut watcher = self.watcher.lock().unwrap();
      (watcher.shutdown.take(), watcher.thread.take())
    };

    if let Some(shutdown) = shutdown {
      shutdown.stop();
    }

    if let Some(thread) = thread {
      if thread.thread().id() != std::thread::current().id() {
        let _ = thread.join();
      }
    }
  }
}
