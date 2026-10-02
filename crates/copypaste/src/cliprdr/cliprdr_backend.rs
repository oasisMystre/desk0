use std::fmt::Debug;

use clipboard_rs::{Clipboard, RustImageData, common::RustImage};
use image::ImageFormat;
use ironrdp::{
  cliprdr::{
    backend::{ClipboardMessage, CliprdrBackend},
    pdu::{
      ClipboardFormat, ClipboardFormatId, ClipboardFormatName, ClipboardGeneralCapabilityFlags,
      FileContentsRequest, FileContentsResponse, FileDescriptor, FormatDataRequest,
      FormatDataResponse, LockDataId,
    },
  },
  core::impl_as_any,
};
use ironrdp_cliprdr_format::{
  bitmap::{dib_to_png, dibv5_to_png},
  html::cf_html_to_plain_html,
};

use crate::cliprdr::{
  clipboard_watcher::ClipboardStatus,
  cliprdr_handler::CliprdrHandler,
  types::{Event, download_state::DownloadEvent},
};

pub const CF_RTF: u32 = 0xC001;
pub const CF_HTML: u32 = 0xC002;
pub const MAX_CLIPBOARD_CHUNK: u32 = 1024 * 1024;

pub struct CliprdrHandlerBackend {
  pub(crate) temp_dir: String,
  pub(crate) handler: CliprdrHandler,
  pub(crate) capabilities: ClipboardGeneralCapabilityFlags,
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
    self.handler.set_status(ClipboardStatus::Initialize);
  }
}

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
    self.handler.set_status(ClipboardStatus::Ready);
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
    match request.format {
      ClipboardFormatId::CF_UNICODETEXT => {
        self.handler.send_text();
      }
      ClipboardFormatId::CF_HDROP => {
        self.handler.send_file_list();
      }
      ClipboardFormatId::CF_DIB => {
        self.handler.send_image();
      }
      ClipboardFormatId::CF_DIBV5 => {
        self.handler.send_image();
      }
      ClipboardFormatId(CF_RTF) => {
        self.handler.send_rich_text();
      }
      ClipboardFormatId(CF_HTML) => {
        self.handler.send_html();
      }
      _ => {}
    }

    self.handler.set_status(ClipboardStatus::Copy);
  }

  fn on_format_data_response(&mut self, response: FormatDataResponse<'_>) {
    self.handler.set_status(ClipboardStatus::Paste);

    let clipboard = self.handler.clipboard();
    if let Ok(text) = response.to_string() {
      if !text.is_empty() {
        if let Err(error) = clipboard.set_text(text) {
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
      if let Err(error) = clipboard.set_html(data.to_string()) {
        tracing::error!(?error, "Failed to paste HTML");
      }

      return;
    }

    if let Ok(data) = std::str::from_utf8(data) {
      if data.starts_with("{\\rtf1") {
        if let Err(error) = clipboard.set_rich_text(data.to_string()) {
          tracing::error!(?error, "Failed to paste rich text");
        }

        return;
      }
    }

    if let Ok(data) = dibv5_to_png(data) {
      if let Ok(image) = image::load_from_memory_with_format(&data, ImageFormat::Png) {
        let rust_image = RustImageData::from_dynamic_image(image);
        if let Err(error) = clipboard.set_image(rust_image) {
          tracing::error!(?error, "Failed to paste image");
        }
      }
      return;
    }

    if let Ok(data) = dib_to_png(data) {
      if let Ok(image) = image::load_from_memory_with_format(&data, ImageFormat::Png) {
        let rust_image = RustImageData::from_dynamic_image(image);
        if let Err(error) = clipboard.set_image(rust_image) {
          tracing::error!(?error, "Failed to paste image");
        }
      }
      return;
    }
  }

  fn on_file_contents_request(&mut self, request: FileContentsRequest) {
    self.handler.send_files(request);
  }

  fn on_file_contents_response<'a>(&mut self, response: FileContentsResponse<'a>) {
    if response.is_error() {
      tracing::warn!(
        stream_id = response.stream_id(),
        "remote file contents request failed"
      );
      self.handler.abort_download_chunk(response.stream_id());
      self.pump_download();
      return;
    }

    if let Err(error) = self
      .handler
      .request_next_chunk(response.stream_id(), response.data())
    {
      tracing::error!(?error, "failed to store clipboard file chunk");
      self.handler.clear_download();
      return;
    }

    self.pump_download();
  }

  fn on_remote_file_list(&mut self, files: &[FileDescriptor], _: Option<u32>) {
    self.handler.request_files(&self.temp_dir, files);
    self.pump_download();
  }

  fn on_lock(&mut self, data_id: LockDataId) {
    self.handler.add_lock(data_id);
  }

  fn on_unlock(&mut self, data_id: LockDataId) {
    self.handler.remove_lock(data_id);
  }

  fn on_outgoing_locks_cleared(&mut self, data_ids: &[LockDataId]) {
    self.handler.remove_locks(Some(data_ids));
    self.handler.clear_download();
  }
}

impl CliprdrHandlerBackend {
  fn pump_download(&mut self) {
    while let Some(event) = self.handler.next_download_event() {
      match event {
        DownloadEvent::DownloadComplete(files) => {
          self.handler.set_status(ClipboardStatus::Paste);
          let clipboard = self.handler.clipboard();
          if let Err(error) = clipboard.set_files(
            files
              .iter()
              .map(|path| path.to_string_lossy().to_string())
              .collect(),
          ) {
            tracing::error!(?error, "failed to publish downloaded files to clipboard");
            self.handler.set_status(ClipboardStatus::Ready);
          }
          self.handler.clear();
        }
        DownloadEvent::SendFileContentsRequest(event) => {
          if let Err(error) = self.handler.send(event) {
            tracing::error!(?error, "sending file contents request failed");
          }
          break;
        }
      }
    }

    self.handler.reset_state_if_download_idle();
  }
}
