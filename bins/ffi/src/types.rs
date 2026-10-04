use copypaste::cliprdr::types::clipboard_event::{ClipboardEvent, FileDownloadStatus};
use flutter_rust_bridge::frb;
use ironrdp::{
  client::rdp::{AutoReconnectDecision, RdpOutputEvent},
  pdu::geometry::Rectangle,
  session::GracefulDisconnectReason,
};
use std::ffi::c_char;

#[derive(Clone, Copy)]
pub enum SseMouseButton {
  Left = 0,
  Middle = 1,
  Right = 2,
  X1 = 3,
  X2 = 4,
}

#[derive(Clone)]
pub enum SseInputEvent {
  WheelRotations {
    is_vertical: bool,
    rotation_units: i16,
  },
  MouseButtonPressed(SseMouseButton),
  MouseButtonReleased(SseMouseButton),
  MousePosition {
    x: u16,
    y: u16,
  },
  KeyPressed {
    code: u8,
    extended: bool,
  },
  KeyReleased {
    code: u8,
    extended: bool,
  },
}

impl Into<ironrdp::input::MouseButton> for SseMouseButton {
  fn into(self) -> ironrdp::input::MouseButton {
    match self {
      SseMouseButton::Left => ironrdp::input::MouseButton::Left,
      SseMouseButton::Middle => ironrdp::input::MouseButton::Middle,
      SseMouseButton::Right => ironrdp::input::MouseButton::Right,
      SseMouseButton::X1 => ironrdp::input::MouseButton::X1,
      SseMouseButton::X2 => ironrdp::input::MouseButton::X2,
    }
  }
}

impl Into<ironrdp::input::Operation> for SseInputEvent {
  fn into(self) -> ironrdp::input::Operation {
    match self {
      SseInputEvent::MousePosition { x, y } => {
        ironrdp::input::Operation::MouseMove(ironrdp::input::MousePosition { x, y })
      }
      SseInputEvent::WheelRotations {
        is_vertical,
        rotation_units,
      } => ironrdp::input::Operation::WheelRotations(ironrdp::input::WheelRotations {
        is_vertical,
        rotation_units,
      }),
      SseInputEvent::MouseButtonPressed(button) => {
        ironrdp::input::Operation::MouseButtonPressed(button.into())
      }
      SseInputEvent::MouseButtonReleased(button) => {
        ironrdp::input::Operation::MouseButtonReleased(button.into())
      }
      SseInputEvent::KeyPressed { code, extended } => {
        ironrdp::input::Operation::KeyPressed(ironrdp::input::Scancode::from_u8(extended, code))
      }
      SseInputEvent::KeyReleased { code, extended } => {
        ironrdp::input::Operation::KeyReleased(ironrdp::input::Scancode::from_u8(extended, code))
      }
    }
  }
}

#[repr(C)]
#[derive()]
pub enum OutputEvent {
  Terminated,
  ConnectionFailure(*const c_char),
  Image {
    width: u16,
    height: u16,
    buffer_len: usize,
    buffer: *mut u32,
  },
  PointerHidden,
  PointerDefault,
  PointerPosition {
    x: u16,
    y: u16,
  },
  PointerBitmap {
    width: u16,
    height: u16,
    bitmap_data_len: usize,
    bitmap_data: *mut u8,
    hotspot_x: u16,
    hotspot_y: u16,
  },
}

#[derive(Clone)]
pub enum SseDisconnectReason {
  UserInitiated,
  ServerInitiated,
  Other(String),
}

#[repr(C)]
#[derive(Clone)]
pub enum SseFileDownloadStatus {
  Download {
    id: u32,
    total_size: u64,
  },
  Progress {
    id: u32,
    total_size: u64,
    received: u64,
  },
  Completed {
    id: u32,
  },
}

#[repr(C)]
#[derive(Clone)]
pub enum SseClipboardEvent {
  Text,
  Image,
  Html,
  RichText,
  File(SseFileDownloadStatus),
}

#[repr(C)]
#[derive(Clone)]
pub struct SseInclusizeRectangle {
  pub left: u16,
  pub top: u16,
  pub right: u16,
  pub bottom: u16,
  pub width: u16,
  pub height: u16,
}

#[repr(C)]
#[derive(Clone)]
pub enum SseOutputEvent {
  Connected,
  LoginComplete,
  MonitorLayout(Vec<SseMonitor>),
  Clipboard(SseClipboardEvent),
  TerminatedError(String),
  TerminatedSuccess(SseDisconnectReason),
  ConnectionFailure(String),
  Image {
    width: u16,
    height: u16,
    buffer: Vec<u32>,
  },
  DesktopUpdate {
    width: u16,
    height: u16,
    buffer: Vec<u32>,
    region: SseInclusizeRectangle,
  },
  PointerHidden,
  PointerDefault,
  PointerPosition {
    x: u16,
    y: u16,
  },
  PointerBitmap {
    width: u16,
    height: u16,
    bitmap_data: Vec<u8>,
    hotspot_x: u16,
    hotspot_y: u16,
  },
  WindowingOrders(Vec<u8>),
  Ignored,
  AutoReconnecting {
    disconnect_reason: u32,
    attempt: u32,
    maximum_attempts: u32,
  },
  AutoReconnected,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SseDesktopSize {
  pub width: u16,
  pub height: u16,
}

#[derive(Clone)]
pub struct SseMonitor {
  pub left: i32,
  pub top: i32,
  pub right: i32,
  pub bottom: i32,
  pub flags: u32,
}

impl From<FileDownloadStatus> for SseFileDownloadStatus {
  fn from(value: FileDownloadStatus) -> Self {
    match value {
      FileDownloadStatus::Download { id, total_size } => {
        SseFileDownloadStatus::Download { id, total_size }
      }
      FileDownloadStatus::Progress {
        id,
        total_size,
        received,
      } => SseFileDownloadStatus::Progress {
        id,
        total_size,
        received,
      },
      FileDownloadStatus::Completed { id } => SseFileDownloadStatus::Completed { id },
    }
  }
}

impl From<ClipboardEvent> for SseClipboardEvent {
  fn from(value: ClipboardEvent) -> Self {
    match value {
      ClipboardEvent::Text => SseClipboardEvent::Text,
      ClipboardEvent::Image => SseClipboardEvent::Image,
      ClipboardEvent::Html => SseClipboardEvent::Html,
      ClipboardEvent::RichText => SseClipboardEvent::RichText,
      ClipboardEvent::File(status) => SseClipboardEvent::File(status.into()),
    }
  }
}

impl From<GracefulDisconnectReason> for SseDisconnectReason {
  fn from(value: GracefulDisconnectReason) -> Self {
    match value {
      GracefulDisconnectReason::UserInitiated => Self::UserInitiated,
      GracefulDisconnectReason::ServerInitiated => Self::ServerInitiated,
      GracefulDisconnectReason::Other(value) => Self::Other(value),
    }
  }
}

impl From<RdpOutputEvent> for SseOutputEvent {
  fn from(value: RdpOutputEvent) -> Self {
    match value {
      RdpOutputEvent::Connected => Self::Connected,
      RdpOutputEvent::PointerHidden => Self::PointerHidden,
      RdpOutputEvent::PointerDefault => Self::PointerDefault,
      RdpOutputEvent::AutoReconnected => Self::AutoReconnected,
      RdpOutputEvent::MonitorLayout(monitors) => Self::MonitorLayout(
        monitors
          .into_iter()
          .map(|monitor| SseMonitor {
            left: monitor.left,
            top: monitor.top,
            right: monitor.right,
            bottom: monitor.bottom,
            flags: monitor.flags.bits(),
          })
          .collect(),
      ),
      RdpOutputEvent::LoginComplete => Self::LoginComplete,
      RdpOutputEvent::Image {
        buffer,
        width,
        height,
      } => Self::Image {
        buffer,
        width: width.get(),
        height: height.get(),
      },
      RdpOutputEvent::DesktopUpdate(update) => {
        let (buffer, width, height, region) = update.into_parts();
        Self::DesktopUpdate {
          buffer,
          width: width.get(),
          height: height.get(),
          region: SseInclusizeRectangle {
            left: region.left,
            top: region.top,
            right: region.right,
            bottom: region.bottom,
            width: region.width(),
            height: region.height(),
          },
        }
      }
      RdpOutputEvent::PointerPosition { x, y } => Self::PointerPosition { x, y },
      RdpOutputEvent::PointerBitmap(pointer) => Self::PointerBitmap {
        width: pointer.width,
        height: pointer.height,
        hotspot_x: pointer.hotspot_x,
        hotspot_y: pointer.hotspot_y,
        bitmap_data: pointer.bitmap_data.clone(),
      },
      RdpOutputEvent::WindowingOrders(items) => Self::WindowingOrders(items),
      RdpOutputEvent::AutoReconnecting {
        disconnect_reason,
        attempt,
        maximum_attempts,
        response,
      } => {
        let _ = response.send(AutoReconnectDecision::Continue);
        Self::AutoReconnecting {
          attempt,
          maximum_attempts,
          disconnect_reason,
        }
      }
      RdpOutputEvent::ConnectionFailure(error) => Self::ConnectionFailure(error.to_string()),
      RdpOutputEvent::Terminated(reason) => match reason {
        Ok(reason) => Self::TerminatedSuccess(reason.into()),
        Err(error) => Self::TerminatedError(error.to_string()),
      },
      RdpOutputEvent::RailHandshake { .. }
      | RdpOutputEvent::RailDesktopSynchronized { .. }
      | RdpOutputEvent::RailPostHandshakeQueueReleased { .. }
      | RdpOutputEvent::RailExecuteResult(_)
      | RdpOutputEvent::RailExecuteFailed { .. }
      | RdpOutputEvent::RailApplicationId { .. }
      | RdpOutputEvent::RailControl(_)
      | RdpOutputEvent::PostLogonDisplayRedraw
      | RdpOutputEvent::MalformedBitmapDisplayRedraw
      | RdpOutputEvent::DisplayResizeFallback(_) => Self::Ignored,
    }
  }
}
