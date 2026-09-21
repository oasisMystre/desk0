use std::{fmt, sync::{Arc, Mutex}};

use ironrdp::{cliprdr::{backend::{ClipboardMessage, CliprdrBackend, CliprdrBackendFactory}, pdu::{ClipboardFormat, ClipboardFormatId, ClipboardGeneralCapabilityFlags, FileContentsRequest, FileContentsResponse, FormatDataRequest, FormatDataResponse, LockDataId, OwnedFormatDataResponse}}, client::rdp::RdpInputEvent, core::impl_as_any};
use tokio::sync::mpsc::UnboundedSender;

pub type ClipboardCallback = extern "C" fn(*const u8, usize);

#[derive(Clone)]
pub struct ClipboardBridge {
    text: Arc<Mutex<String>>,
    input: Arc<Mutex<Option<UnboundedSender<RdpInputEvent>>>>,
    callback: Option<ClipboardCallback>,
}

impl fmt::Debug for ClipboardBridge { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.debug_struct("ClipboardBridge").finish_non_exhaustive() } }

impl ClipboardBridge {
    pub fn new(callback: Option<ClipboardCallback>) -> Self { Self { text: Arc::new(Mutex::new(String::new())), input: Arc::new(Mutex::new(None)), callback } }
    pub fn set_input(&self, input: UnboundedSender<RdpInputEvent>) { *self.input.lock().unwrap() = Some(input); }
    pub fn set_text(&self, text: String) -> Result<(), ()> {
        *self.text.lock().unwrap() = text;
        self.send(ClipboardMessage::SendInitiateCopy(vec![ClipboardFormat::new(ClipboardFormatId::CF_UNICODETEXT)]))
    }
    fn send(&self, message: ClipboardMessage) -> Result<(), ()> { self.input.lock().unwrap().as_ref().ok_or(())?.send(RdpInputEvent::Clipboard(message)).map_err(|_| ()) }
    pub fn backend(&self) -> ClipboardBackend { ClipboardBackend { bridge: self.clone(), temp_dir: std::env::temp_dir().to_string_lossy().into_owned(), capabilities: ClipboardGeneralCapabilityFlags::empty() } }
}

impl CliprdrBackendFactory for ClipboardBridge { fn build_cliprdr_backend(&self) -> Box<dyn CliprdrBackend> { Box::new(self.backend()) } }

#[derive(Debug)]
pub struct ClipboardBackend { bridge: ClipboardBridge, temp_dir: String, capabilities: ClipboardGeneralCapabilityFlags }
impl_as_any!(ClipboardBackend);

impl ClipboardBackend {
    fn send_text(&self) { let text = self.bridge.text.lock().unwrap().clone(); let _ = self.bridge.send(ClipboardMessage::SendFormatData(OwnedFormatDataResponse::new_unicode_string(&text))); }
}

impl CliprdrBackend for ClipboardBackend {
    fn temporary_directory(&self) -> &str { &self.temp_dir }
    fn client_capabilities(&self) -> ClipboardGeneralCapabilityFlags { self.capabilities }
    fn on_ready(&mut self) { self.on_request_format_list(); }
    fn on_request_format_list(&mut self) { if !self.bridge.text.lock().unwrap().is_empty() { let _ = self.bridge.send(ClipboardMessage::SendInitiateCopy(vec![ClipboardFormat::new(ClipboardFormatId::CF_UNICODETEXT)])); } }
    fn on_process_negotiated_capabilities(&mut self, flags: ClipboardGeneralCapabilityFlags) { self.capabilities = flags; }
    fn on_remote_copy(&mut self, formats: &[ClipboardFormat]) { if let Some(format) = formats.iter().find(|f| f.id == ClipboardFormatId::CF_UNICODETEXT) { let _ = self.bridge.send(ClipboardMessage::SendInitiatePaste(format.id)); } }
    fn on_format_data_request(&mut self, request: FormatDataRequest) { if request.format == ClipboardFormatId::CF_UNICODETEXT { self.send_text(); } }
    fn on_format_data_response(&mut self, response: FormatDataResponse<'_>) {
        if response.is_error() { return; }
        let units: Vec<u16> = response.data().chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).take_while(|v| *v != 0).collect();
        let Ok(text) = String::from_utf16(&units) else { return };
        if let Some(callback) = self.bridge.callback { callback(text.as_ptr().cast(), text.len()); }
    }
    fn on_file_contents_request(&mut self, _: FileContentsRequest) {}
    fn on_file_contents_response(&mut self, _: FileContentsResponse<'_>) {}
    fn on_lock(&mut self, _: LockDataId) {}
    fn on_unlock(&mut self, _: LockDataId) {}
}

