use std::{
  borrow::Cow,
  fs::File,
  io::{Read, Seek, SeekFrom},
  sync::{Arc, Mutex},
};

use clipboard_rs::ClipboardContext;
use ironrdp::cliprdr::{
  backend::{ClipboardMessage, CliprdrBackend, CliprdrBackendFactory},
  pdu::{
    ClipboardFormat, ClipboardFormatId, ClipboardFormatName, ClipboardGeneralCapabilityFlags,
    FileContentsRequest, FileContentsResponse, FileDescriptor, LockDataId, OwnedFormatDataResponse,
    PackedFileList,
  },
};
#[cfg(feature = "server")]
use ironrdp::server::{CliprdrServerFactory, ServerEventSender};
#[cfg(not(feature = "server"))]
use ironrdp_client::rdp::RdpInputSender;
#[cfg(not(feature = "server"))]
use tokio::sync::mpsc::error::TrySendError;
#[cfg(feature = "server")]
use tokio::sync::mpsc::{UnboundedSender, error::SendError};

use crate::cliprdr::{
  clipboard_watcher::{ClipboardStatus, ClipboardWatcherHandler},
  cliprdr_backend::{CF_HTML, CF_RTF, CliprdrHandlerBackend},
  types::{
    Event, Sender,
    clipboard_file::ClipboardState,
    download_state::{DownloadEvent, DownloadState},
  },
};

pub const MAX_CLIPBOARD_CHUNK: u32 = 1024 * 1024;

#[derive(Clone)]
pub struct CliprdrHandler {
  sender: Arc<Mutex<Sender>>,
  watcher: ClipboardWatcherHandler,
  download_state: Arc<Mutex<DownloadState>>,
}

impl CliprdrBackendFactory for CliprdrHandler {
  fn build_cliprdr_backend(&self) -> Box<dyn CliprdrBackend> {
    Box::new(self.build_backend())
  }
}

impl CliprdrHandler {
  fn build_backend(&self) -> CliprdrHandlerBackend {
    self.watcher.set_status(ClipboardStatus::Initialize);

    let temp_dir = std::env::temp_dir()
      .join("cliprdr")
      .to_string_lossy()
      .into_owned();

    CliprdrHandlerBackend {
      temp_dir,
      handler: self.clone(),
      capabilities: ClipboardGeneralCapabilityFlags::empty(),
    }
  }
}

#[cfg(feature = "server")]
impl CliprdrServerFactory for CliprdrHandler {}

#[cfg(feature = "server")]
impl ServerEventSender for CliprdrHandler {
  fn set_sender(&mut self, sender: UnboundedSender<Event>) {
    *self.sender.lock().unwrap() = Some(sender);
    self.start_watcher();
  }
}

#[cfg(not(feature = "server"))]
impl CliprdrHandler {
  pub fn set_sender(&mut self, sender: RdpInputSender) {
    *self.sender.lock().unwrap() = Some(sender);
    self.start_watcher();
  }
}

impl CliprdrHandler {
  pub fn new() -> Self {
    let sender: Arc<Mutex<Sender>> = Arc::new(Mutex::new(None));
    let callback_sender = Arc::clone(&sender);

    let watcher = ClipboardWatcherHandler::new(move |state| {
      let sender = callback_sender.lock().unwrap().clone();
      send_format_list(&state, &sender);
    });

    Self {
      sender,
      watcher,
      download_state: Arc::new(Mutex::new(DownloadState::default())),
    }
  }

  #[cfg(feature = "server")]
  pub fn send(&self, message: Event) -> Result<(), SendError<Event>> {
    let sender = self.sender.lock().unwrap();
    send(&sender, message)
  }

  #[cfg(not(feature = "server"))]
  pub fn send(&self, message: Event) -> Result<(), TrySendError<Event>> {
    let sender = self.sender.lock().unwrap();
    send(&sender, message)
  }

  pub fn send_format_list(&self) {
    let state = {
      let state = self.watcher.state.lock().unwrap();
      state
    };

    let sender = self.sender.lock().unwrap();

    send_format_list(&state, &sender);
  }

  pub fn send_text(&self) {
    let text = {
      let state = self.watcher.state.lock().unwrap();
      state.text.clone()
    };

    if let Some(text) = text {
      if let Err(error) = self.send(Event::Clipboard(ClipboardMessage::SendFormatData(
        OwnedFormatDataResponse::new_string(&text),
      ))) {
        tracing::error!(?error, "Failed to send text clipboard event");
      }
    }
  }

  pub fn send_html(&self) {
    let html = {
      let state = self.watcher.state.lock().unwrap();
      state.html.clone()
    };

    if let Some(html) = html {
      if let Err(error) = self.send(Event::Clipboard(ClipboardMessage::SendFormatData(
        OwnedFormatDataResponse::new_data(Cow::Owned(html.as_bytes().to_vec())),
      ))) {
        tracing::error!(?error, "Failed to send html clipboard event");
      }
    }
  }

  pub fn send_rich_text(&self) {
    let rich_text = {
      let state = self.watcher.state.lock().unwrap();
      state.rich_text.clone()
    };

    if let Some(rich_text) = rich_text {
      if let Err(error) = self.send(Event::Clipboard(ClipboardMessage::SendFormatData(
        OwnedFormatDataResponse::new_data(Cow::Owned(rich_text.as_bytes().to_vec())),
      ))) {
        tracing::error!(?error, "Failed to send rich_text clipboard event");
      }
    }
  }

  pub fn send_file_list(&self) {
    let files = {
      let state = self.watcher.state.lock().unwrap();
      match state.file_descriptors() {
        Ok(files) => files,
        Err(error) => {
          tracing::error!(?error, "reading clipboard files failed");
          return;
        }
      }
    };

    if !files.is_empty() {
      if let Err(error) = self.send(Event::Clipboard(ClipboardMessage::SendFormatData(
        OwnedFormatDataResponse::new_file_list(&PackedFileList { files }).unwrap(),
      ))) {
        tracing::error!(?error, "failed to send file lists");
      }
    }
  }

  pub fn send_image(&self) {
    let image = {
      let state = self.watcher.state.lock().unwrap();
      state.image.clone()
    };

    if let Some(image) = image {
      if let Err(error) = self.send(Event::Clipboard(ClipboardMessage::SendFormatData(
        OwnedFormatDataResponse::new_data(Cow::Owned(image.clone())),
      ))) {
        tracing::error!(?error, "failed to send file lists");
      }
    }
  }

  pub fn send_files(&self, request: FileContentsRequest) {
    let path = {
      let state = self.watcher.state.lock().unwrap();
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
        if let Err(error) = self.send(Event::Clipboard(
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

        if let Err(error) = self.send(Event::Clipboard(
          ClipboardMessage::SendFileContentsResponse(FileContentsResponse::new_error(
            request.stream_id,
          )),
        )) {
          tracing::error!(?error, "Failed to send file content response");
        }
      }
    }
  }

  pub fn set_status(&self, status: ClipboardStatus) {
    self.watcher.set_status(status);
  }

  pub fn get_status(&self) -> ClipboardStatus {
    self.watcher.get_status()
  }

  pub fn add_lock(&self, lock: LockDataId) {
    let mut state = self.watcher.state.lock().unwrap();
    state.active_locks.push(lock);
  }

  pub fn remove_lock(&self, lock: LockDataId) {
    let mut state = self.watcher.state.lock().unwrap();
    state.active_locks.retain(|element| element != &lock);
  }

  pub fn remove_locks(&self, locks: Option<&[LockDataId]>) {
    let mut state = self.watcher.state.lock().unwrap();
    match locks {
      Some(locks) => state.active_locks.retain(|lock| !locks.contains(lock)),
      None => state.active_locks.clear(),
    }
  }

  pub fn clipboard(&self) -> Arc<ClipboardContext> {
    Arc::clone(&self.watcher.clipboard)
  }

  pub fn start_watcher(&self) {
    self.watcher.start()
  }

  pub fn clear(&self) {
    self.watcher.clear()
  }

  pub fn request_next_chunk(&self, stream_id: u32, data: &[u8]) -> std::io::Result<()> {
    let mut state = self.download_state.lock().unwrap();
    state.download_chunk(stream_id, data)
  }

  pub fn abort_download_chunk(&self, stream_id: u32) -> bool {
    self.download_state.lock().unwrap().abort_current(stream_id)
  }

  pub fn request_files(&self, tmp_dir: &str, fd: &[FileDescriptor]) {
    let mut state = self.download_state.lock().unwrap();

    state.request_file(tmp_dir, fd)
  }

  pub fn reset_state_if_download_idle(&self) {
    let is_idle = { self.download_state.lock().unwrap().is_idle() };
    if is_idle {
      tracing::debug!("active locks cleared");
      self.watcher.clear();
      self.remove_locks(None);
    }
  }

  pub fn clear_download(&self) {
    self.download_state.lock().unwrap().clear();
  }

  pub fn next_download_event(&self) -> Option<DownloadEvent> {
    self.download_state.lock().unwrap().next_event()
  }
}

#[cfg(feature = "server")]
pub fn send(sender: &Sender, message: Event) -> Result<(), SendError<Event>> {
  if let Some(sender) = sender {
    Ok(sender.send(message)?)
  } else {
    Err(SendError(message))
  }
}

#[cfg(not(feature = "server"))]
pub fn send(sender: &Sender, message: Event) -> Result<(), TrySendError<Event>> {
  if let Some(sender) = sender {
    Ok(sender.try_send(message)?)
  } else {
    Err(TrySendError::Closed(message))
  }
}

fn send_format_list(state: &ClipboardState, sender: &Sender) {
  if !state.files.is_empty() {
    match state.file_descriptors() {
      Ok(files) => {
        if let Err(error) = send(
          &sender,
          Event::Clipboard(ClipboardMessage::SendInitiateFileCopy(files)),
        ) {
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

  if let Err(error) = send(
    sender,
    Event::Clipboard(ClipboardMessage::SendInitiateCopy(formats)),
  ) {
    tracing::error!(?error, "Failed to send initiate copy event");
  }
}
