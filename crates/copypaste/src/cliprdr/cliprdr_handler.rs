use std::{
  borrow::Cow,
  fs::File,
  io::{Read, Seek, SeekFrom},
  sync::{Arc, Mutex},
};

use clipboard_rs::{Clipboard, common::RustImageData};
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
use ironrdp::client::rdp::RdpInputSender;
#[cfg(not(feature = "server"))]
use tokio::sync::mpsc::error::TrySendError;
#[cfg(feature = "server")]
use tokio::sync::mpsc::{UnboundedSender, error::SendError};

use crate::cliprdr::{
  clipboard_watcher::{ClipboardStatus, ClipboardWatcherHandler},
  cliprdr_backend::{CF_HTML, CF_RTF, CliprdrHandlerBackend},
  types::{
    Event, Sender,
    clipboard_event::ClipboardEvent,
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
  callback: Option<Arc<dyn Fn(ClipboardEvent) + Send + Sync>>,
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
  pub fn new_inner(callback: Option<Arc<dyn Fn(ClipboardEvent) + Send + Sync + 'static>>) -> Self {
    let sender: Arc<Mutex<Sender>> = Arc::new(Mutex::new(None));
    let callback_sender = Arc::clone(&sender);

    let watcher = ClipboardWatcherHandler::new(move |state| {
      let sender = callback_sender.lock().unwrap().clone();
      send_format_list(&state, &sender);
    });

    Self {
      sender,
      watcher,
      callback,
      download_state: Arc::new(Mutex::new(DownloadState::default())),
    }
  }

  pub fn new() -> Self {
    Self::new_inner(None)
  }

  pub fn with_callback<T>(callback: T) -> Self
  where
    T: Fn(ClipboardEvent) + Send + Sync + 'static,
  {
    Self::new_inner(Some(Arc::new(callback)))
  }

  #[cfg(feature = "server")]
  pub(crate) fn send(&self, message: Event) -> Result<(), SendError<Event>> {
    let sender = self.sender.lock().unwrap();
    send(&sender, message)
  }

  #[cfg(not(feature = "server"))]
  pub(crate) fn send(&self, message: Event) -> Result<(), TrySendError<Event>> {
    let sender = self.sender.lock().unwrap();
    send(&sender, message)
  }

  pub(crate) fn send_format_list(&self) {
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

  pub(crate) fn send_html(&self) {
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

  pub(crate) fn send_rich_text(&self) {
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

  pub(crate) fn send_file_list(&self) {
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

  pub(crate) fn send_image(&self) {
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

  pub(crate) fn send_files(&self, request: FileContentsRequest) {
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

  pub(crate) fn set_status(&self, status: ClipboardStatus) {
    self.watcher.set_status(status);
  }

  pub fn get_status(&self) -> ClipboardStatus {
    self.watcher.get_status()
  }

  pub(crate) fn add_lock(&self, lock: LockDataId) {
    let mut state = self.watcher.state.lock().unwrap();
    state.active_locks.push(lock);
  }

  pub(crate) fn remove_lock(&self, lock: LockDataId) {
    let mut state = self.watcher.state.lock().unwrap();
    state.active_locks.retain(|element| element != &lock);
  }

  pub(crate) fn remove_locks(&self, locks: Option<&[LockDataId]>) {
    let mut state = self.watcher.state.lock().unwrap();
    match locks {
      Some(locks) => state.active_locks.retain(|lock| !locks.contains(lock)),
      None => state.active_locks.clear(),
    }
  }

  pub(crate) fn start_watcher(&self) {
    self.watcher.start()
  }

  fn clear(&self) {
    self.watcher.clear()
  }

  pub(crate) fn request_next_chunk(&self, stream_id: u32, data: &[u8]) -> std::io::Result<()> {
    let mut state = self.download_state.lock().unwrap();
    let event = state.download_chunk(stream_id, data)?;
    if let Some(callback) = &self.callback {
      (callback)(event);
    }

    Ok(())
  }

  pub(crate) fn abort_download_chunk(&self, stream_id: u32) -> bool {
    self.download_state.lock().unwrap().abort_current(stream_id)
  }

  pub(crate) fn request_files(&self, tmp_dir: &str, fd: &[FileDescriptor]) {
    let mut state = self.download_state.lock().unwrap();

    if let Some(event) = state.request_file(tmp_dir, fd) {
      if let Some(callback) = &self.callback {
        (callback)(event);
      }
    }
  }

  fn reset_state_if_download_idle(&self) {
    let is_idle = { self.download_state.lock().unwrap().is_idle() };
    if is_idle {
      tracing::debug!("active locks cleared");
      self.watcher.clear();
      self.remove_locks(None);
    }
  }

  pub(crate) fn clear_download(&self) {
    self.download_state.lock().unwrap().clear();
  }

  pub(crate) fn pump_download(&mut self) {
    while let Some(event) = self.download_state.lock().unwrap().next_event() {
      match event {
        DownloadEvent::DownloadComplete(files) => {
          self.set_status(ClipboardStatus::Paste);
          if let Err(error) = self.watcher.clipboard.set_files(
            files
              .iter()
              .map(|path| path.to_string_lossy().to_string())
              .collect(),
          ) {
            tracing::error!(?error, "failed to publish downloaded files to clipboard");
            self.set_status(ClipboardStatus::Ready);
          }
          self.clear();
        }
        DownloadEvent::SendFileContentsRequest(event) => {
          if let Err(error) = self.send(event) {
            tracing::error!(?error, "sending file contents request failed");
          }
          break;
        }
      }
    }

    self.reset_state_if_download_idle();
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

#[macro_export]
macro_rules! clipboard_setter {
    (
        $(#[$meta:meta])*
        $name:ident($($arg:ident : $ty:ty),*) => $method:ident($($expr:expr),*) => $event:expr
    ) => {
        $(#[$meta])*
        pub fn $name(&self, $($arg: $ty),*) -> clipboard_rs::Result<()> {
            self.watcher.clipboard.$method($($expr),*)?;
            if let Some(callback) = &self.callback {
                callback($event);
            }
            Ok(())
        }
    };
}

impl CliprdrHandler {
  clipboard_setter!(
      set_text(data: String)
      => set_text(data.to_string())
      => ClipboardEvent::Text
  );

  clipboard_setter!(
      set_html(data: String)
      => set_html(data.to_string())
      => ClipboardEvent::Html
  );

  clipboard_setter!(
      set_rich_text(data: String)
      => set_rich_text(data.to_string())
      => ClipboardEvent::RichText
  );

  clipboard_setter!(
      set_image(data: RustImageData)
      => set_image(data)
      => ClipboardEvent::Image
  );
}
