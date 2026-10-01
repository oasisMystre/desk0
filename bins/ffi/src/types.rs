use flutter_rust_bridge::frb;
use ironrdp::{client::rdp::RdpOutputEvent, session::GracefulDisconnectReason};
use std::ffi::c_char;

#[repr(C)]
#[derive(Clone, Copy)]
pub enum MouseButton {
  Left = 0,
  Middle = 1,
  Right = 2,
  X1 = 3,
  X2 = 4,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum InputEvent {
  WheelRotations {
    is_vertical: bool,
    rotation_units: i16,
  },
  MouseButtonPressed(MouseButton),
  MouseButtonReleased(MouseButton),
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

impl Into<ironrdp::input::MouseButton> for MouseButton {
  fn into(self) -> ironrdp::input::MouseButton {
    match self {
      MouseButton::Left => ironrdp::input::MouseButton::Left,
      MouseButton::Middle => ironrdp::input::MouseButton::Middle,
      MouseButton::Right => ironrdp::input::MouseButton::Right,
      MouseButton::X1 => ironrdp::input::MouseButton::X1,
      MouseButton::X2 => ironrdp::input::MouseButton::X2,
    }
  }
}

impl Into<ironrdp::input::Operation> for InputEvent {
  fn into(self) -> ironrdp::input::Operation {
    match self {
      InputEvent::MousePosition { x, y } => {
        ironrdp::input::Operation::MouseMove(ironrdp::input::MousePosition { x, y })
      }
      InputEvent::WheelRotations {
        is_vertical,
        rotation_units,
      } => ironrdp::input::Operation::WheelRotations(ironrdp::input::WheelRotations {
        is_vertical,
        rotation_units,
      }),
      InputEvent::MouseButtonPressed(button) => {
        ironrdp::input::Operation::MouseButtonPressed(button.into())
      }
      InputEvent::MouseButtonReleased(button) => {
        ironrdp::input::Operation::MouseButtonReleased(button.into())
      }
      InputEvent::KeyPressed { code, extended } => {
        ironrdp::input::Operation::KeyPressed(ironrdp::input::Scancode::from_u8(extended, code))
      }
      InputEvent::KeyReleased { code, extended } => {
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
pub enum SseOutputEvent {
  TerminatedError(String),
  TerminatedSuccess(SseDisconnectReason),
  ConnectionFailure(String),
  Image {
    width: u16,
    height: u16,
    buffer: Vec<u32>,
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
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SseDesktopSize {
  pub width: u16,
  pub height: u16,
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
      ironrdp::client::rdp::RdpOutputEvent::Image {
        buffer,
        width,
        height,
      } => Self::Image {
        buffer,
        width: width.get(),
        height: height.get(),
      },
      ironrdp::client::rdp::RdpOutputEvent::ConnectionFailure(error) => {
        SseOutputEvent::ConnectionFailure(error.to_string())
      }
      ironrdp::client::rdp::RdpOutputEvent::PointerHidden => Self::PointerHidden,
      ironrdp::client::rdp::RdpOutputEvent::PointerDefault => Self::PointerDefault,
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
      ironrdp::client::rdp::RdpOutputEvent::Terminated(reason) => match reason {
        Ok(reason) => Self::TerminatedSuccess(reason.into()),
        Err(error) => Self::TerminatedError(error.to_string()),
      },
    }
  }
}
