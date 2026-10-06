use copypaste::cliprdr::types::clipboard_event::{ClipboardEvent, FileDownloadStatus};
use ironrdp::{
  client::rdp::{AutoReconnectDecision, RdpOutputEvent},
  session::GracefulDisconnectReason,
};

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

#[repr(C)]
#[derive(Clone)]
pub enum SseTerminalInput {
  StdIn(Vec<u8>),
  Output,
}

#[repr(C)]
#[derive(Clone)]
pub enum SseTerminalOutput {
  StdOut(Vec<u8>),
  StdErr(Vec<u8>),
}

#[repr(C)]
#[derive(Clone)]
pub enum SseStaticChannelInput {
  Terminal(SseTerminalInput),
}

#[repr(C)]
#[derive(Clone)]
pub enum SseStaticChannelOutput {
  Terminal(SseTerminalOutput),
}

#[repr(C)]
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
pub enum SseOutputEvent {
  Connected,
  LoginComplete,
  PointerHidden,
  PointerDefault,
  PointerPosition {
    x: u16,
    y: u16,
  },
  PointerBitmap {
    width: u16,
    height: u16,
    hotspot_x: u16,
    hotspot_y: u16,
    bitmap_data: Vec<u8>,
  },
  Resize {
    width: u16,
    height: u16,
  },
  WindowingOrders(Vec<u8>),
  MonitorLayout(Vec<SseMonitor>),
  TerminatedError(String),
  TerminatedSuccess(SseDisconnectReason),
  ConnectionFailure(String),
  AutoReconnecting {
    disconnect_reason: u32,
    attempt: u32,
    maximum_attempts: u32,
  },
  AutoReconnected,
  Ignored,
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

impl Into<pdu::terminal::TerminalInput> for SseTerminalInput {
  fn into(self) -> pdu::terminal::TerminalInput {
    match self {
      SseTerminalInput::StdIn(data) => pdu::terminal::TerminalInput::StdIn(data),
      SseTerminalInput::Output => pdu::terminal::TerminalInput::Output,
    }
  }
}

impl Into<pdu::static_channel::StaticChannelInput> for SseStaticChannelInput {
  fn into(self) -> pdu::static_channel::StaticChannelInput {
    match self {
      SseStaticChannelInput::Terminal(value) => {
        pdu::static_channel::StaticChannelInput::Terminal(value.into())
      }
    }
  }
}

impl From<pdu::terminal::TerminalOutput> for SseTerminalOutput {
  fn from(value: pdu::terminal::TerminalOutput) -> Self {
    match value {
      pdu::terminal::TerminalOutput::StdOut(data) => Self::StdOut(data),
      pdu::terminal::TerminalOutput::StdErr(data) => Self::StdErr(data),
    }
  }
}

impl From<pdu::static_channel::StaticChannelOutput> for SseStaticChannelOutput {
  fn from(value: pdu::static_channel::StaticChannelOutput) -> Self {
    match value {
      pdu::static_channel::StaticChannelOutput::Terminal(value) => Self::Terminal(value.into()),
    }
  }
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
      RdpOutputEvent::Image { .. }
      | RdpOutputEvent::DesktopUpdate(_)
      | RdpOutputEvent::RailHandshake { .. }
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
