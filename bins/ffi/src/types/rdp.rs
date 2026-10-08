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
pub enum SseDisconnectReason {
  UserInitiated,
  ServerInitiated,
  Other(String),
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

impl From<ironrdp::session::GracefulDisconnectReason> for SseDisconnectReason {
  fn from(value: ironrdp::session::GracefulDisconnectReason) -> Self {
    match value {
      ironrdp::session::GracefulDisconnectReason::UserInitiated => Self::UserInitiated,
      ironrdp::session::GracefulDisconnectReason::ServerInitiated => Self::ServerInitiated,
      ironrdp::session::GracefulDisconnectReason::Other(value) => Self::Other(value),
    }
  }
}

impl From<ironrdp::client::rdp::RdpOutputEvent> for SseOutputEvent {
  fn from(value: ironrdp::client::rdp::RdpOutputEvent) -> Self {
    match value {
      ironrdp::client::rdp::RdpOutputEvent::Connected => Self::Connected,
      ironrdp::client::rdp::RdpOutputEvent::PointerHidden => Self::PointerHidden,
      ironrdp::client::rdp::RdpOutputEvent::PointerDefault => Self::PointerDefault,
      ironrdp::client::rdp::RdpOutputEvent::AutoReconnected => Self::AutoReconnected,
      ironrdp::client::rdp::RdpOutputEvent::MonitorLayout(monitors) => Self::MonitorLayout(
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
      ironrdp::client::rdp::RdpOutputEvent::LoginComplete => Self::LoginComplete,
      ironrdp::client::rdp::RdpOutputEvent::PointerPosition { x, y } => {
        Self::PointerPosition { x, y }
      }
      ironrdp::client::rdp::RdpOutputEvent::PointerBitmap(pointer) => Self::PointerBitmap {
        width: pointer.width,
        height: pointer.height,
        hotspot_x: pointer.hotspot_x,
        hotspot_y: pointer.hotspot_y,
        bitmap_data: pointer.bitmap_data.clone(),
      },
      ironrdp::client::rdp::RdpOutputEvent::WindowingOrders(items) => Self::WindowingOrders(items),
      ironrdp::client::rdp::RdpOutputEvent::AutoReconnecting {
        disconnect_reason,
        attempt,
        maximum_attempts,
        response,
      } => {
        let _ = response.send(ironrdp::client::rdp::AutoReconnectDecision::Continue);
        Self::AutoReconnecting {
          attempt,
          maximum_attempts,
          disconnect_reason,
        }
      }
      ironrdp::client::rdp::RdpOutputEvent::ConnectionFailure(error) => {
        Self::ConnectionFailure(error.to_string())
      }
      ironrdp::client::rdp::RdpOutputEvent::Terminated(reason) => match reason {
        Ok(reason) => Self::TerminatedSuccess(reason.into()),
        Err(error) => Self::TerminatedError(error.to_string()),
      },
      ironrdp::client::rdp::RdpOutputEvent::Image { .. }
      | ironrdp::client::rdp::RdpOutputEvent::DesktopUpdate(_)
      | ironrdp::client::rdp::RdpOutputEvent::RailHandshake { .. }
      | ironrdp::client::rdp::RdpOutputEvent::RailDesktopSynchronized { .. }
      | ironrdp::client::rdp::RdpOutputEvent::RailPostHandshakeQueueReleased { .. }
      | ironrdp::client::rdp::RdpOutputEvent::RailExecuteResult(_)
      | ironrdp::client::rdp::RdpOutputEvent::RailExecuteFailed { .. }
      | ironrdp::client::rdp::RdpOutputEvent::RailApplicationId { .. }
      | ironrdp::client::rdp::RdpOutputEvent::RailControl(_)
      | ironrdp::client::rdp::RdpOutputEvent::PostLogonDisplayRedraw
      | ironrdp::client::rdp::RdpOutputEvent::MalformedBitmapDisplayRedraw
      | ironrdp::client::rdp::RdpOutputEvent::DisplayResizeFallback(_) => Self::Ignored,
    }
  }
}
