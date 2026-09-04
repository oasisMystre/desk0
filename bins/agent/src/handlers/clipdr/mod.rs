use std::{fmt::Debug, sync::Arc, sync::Mutex, time::UNIX_EPOCH};

use clipboard_rs::{Clipboard, ClipboardContext, ClipboardHandler};
use ironrdp::{
  cliprdr::{
    backend::{ClipboardMessage, CliprdrBackend, CliprdrBackendFactory},
    pdu::{
      ClipboardFileAttributes, ClipboardFormat, ClipboardFormatId, ClipboardGeneralCapabilityFlags,
      FileContentsRequest, FileContentsResponse, FileDescriptor, FormatDataRequest,
      FormatDataResponse, LockDataId,
    },
  },
  core::impl_as_any,
  server::{CliprdrServerFactory, ServerEvent, ServerEventSender},
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::handlers::clipdr::types::ClipboardItem;

mod types;

const UNIX_TO_FILETIME: u64 = 116_444_736_000_000_000;

pub struct CliprdrHandler {
  sender: Option<UnboundedSender<ServerEvent>>,
}

struct CliprdrHandlerBackend {
  sender: UnboundedSender<ServerEvent>,
  clipboard: ClipboardContext,
  items: Arc<Mutex<Vec<ClipboardItem>>>,
}

impl Debug for CliprdrHandlerBackend {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CliprdrHandlerBackend")
      .field("sender", &self.sender)
      .finish()
  }
}

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

    Box::new(CliprdrHandlerBackend {
      sender,
      items: Arc::new(Mutex::new(vec![])),
      clipboard: ClipboardContext::new().expect("creating arboard instance failed"),
    })
  }
}

impl CliprdrServerFactory for CliprdrHandler {}

impl_as_any!(CliprdrHandlerBackend);

impl ClipboardHandler for CliprdrHandlerBackend {
  fn on_clipboard_change(&mut self) {
    let mut lock = self.items.lock().unwrap();
    if let Ok(text) = self.clipboard.get_text() {
      lock.push(ClipboardItem::Text(text));
    }

    if let Ok(files) = self.clipboard.get_files() {
      lock.extend(files.into_iter().map(|file| ClipboardItem::File(file)));
    }
  }
}

impl CliprdrBackend for CliprdrHandlerBackend {
  fn temporary_directory(&self) -> &str {
    "/tmp/cliprdr"
  }

  fn client_capabilities(&self) -> ClipboardGeneralCapabilityFlags {
    ClipboardGeneralCapabilityFlags::USE_LONG_FORMAT_NAMES
  }

  fn on_ready(&mut self) {
    todo!()
    
  }

  fn on_request_format_list(&mut self) {}

  fn on_process_negotiated_capabilities(&mut self, capabilities: ClipboardGeneralCapabilityFlags) {
    todo!()
  }

  fn on_remote_copy(&mut self, available_formats: &[ClipboardFormat]) {
    todo!()
  }

  fn on_format_data_request(&mut self, request: FormatDataRequest) {
    todo!()
  }

  fn on_format_data_response(&mut self, response: FormatDataResponse<'_>) {
    todo!()
  }

  fn on_file_contents_request(&mut self, request: FileContentsRequest) {
    todo!()
  }

  fn on_file_contents_response<'a>(&mut self, response: FileContentsResponse<'a>) {}

  fn on_lock(&mut self, data_id: LockDataId) {
    todo!()
  }

  fn on_unlock(&mut self, data_id: LockDataId) {
    todo!()
  }
}

impl CliprdrHandler {
  pub fn new() -> Self {
    Self { sender: None }
  }
}
