use ironrdp::{
  pdu::input::fast_path::SynchronizeFlags,
  server::{KeyboardEvent, MouseButton, MouseEvent},
};
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Archive, Serialize, Deserialize)]
pub enum IpcSynchronizeFlags {
  ScrollLock = 0x01,
  NumLock = 0x02,
  CapsLock = 0x04,
  KanaLock = 0x08,
  Invariant = 0xFF,
}

#[derive(Debug, Archive, Serialize, Deserialize)]
pub enum IpcKeyboardEvent {
  UnicodePressed(u16),
  UnicodeReleased(u16),
  Syncronize(IpcSynchronizeFlags),
  Pressed { code: u8, extended: bool },
  Released { code: u8, extended: bool },
}

#[derive(Debug, Archive, Serialize, Deserialize)]
pub enum IpcMouseButton {
  Left,
  Right,
  Middle,
  X1,
  X2,
  Invariant,
}

#[derive(Debug, Archive, Serialize, Deserialize)]
pub enum IpcMouseEvent {
  Invariant,
  Move {
    x: u16,
    y: u16,
  },
  RelMove {
    x: i32,
    y: i32,
  },
  Button {
    x: u16,
    y: u16,
    pressed: bool,
    button: IpcMouseButton,
  },
  ButtonRel {
    x: i32,
    y: i32,
    pressed: bool,
    button: IpcMouseButton,
  },
  Scroll {
    x: i32,
    y: i32,
  },
  VerticalScroll {
    value: i16,
  },
  HorizontalScroll {
    value: i16,
  },
}

impl From<SynchronizeFlags> for IpcSynchronizeFlags {
  fn from(value: SynchronizeFlags) -> Self {
    match value {
      SynchronizeFlags::CAPS_LOCK => IpcSynchronizeFlags::CapsLock,
      SynchronizeFlags::KANA_LOCK => IpcSynchronizeFlags::KanaLock,
      SynchronizeFlags::NUM_LOCK => IpcSynchronizeFlags::NumLock,
      SynchronizeFlags::SCROLL_LOCK => IpcSynchronizeFlags::ScrollLock,
      _ => IpcSynchronizeFlags::Invariant,
    }
  }
}

impl From<KeyboardEvent> for IpcKeyboardEvent {
  fn from(value: KeyboardEvent) -> Self {
    match value {
      KeyboardEvent::UnicodePressed(code) => Self::UnicodePressed(code),
      KeyboardEvent::UnicodeReleased(code) => Self::UnicodeReleased(code),
      KeyboardEvent::Synchronize(flags) => Self::Syncronize(flags.into()),
      KeyboardEvent::Pressed { code, extended } => Self::Pressed { code, extended },
      KeyboardEvent::Released { code, extended } => Self::Released { code, extended },
    }
  }
}

impl From<MouseButton> for IpcMouseButton {
  fn from(value: MouseButton) -> Self {
    match value {
      MouseButton::X1 => Self::X1,
      MouseButton::X2 => Self::X2,
      MouseButton::Left => Self::Left,
      MouseButton::Right => Self::Right,
      MouseButton::Middle => Self::Middle,
      _ => Self::Invariant,
    }
  }
}

impl From<MouseEvent> for IpcMouseEvent {
  fn from(value: MouseEvent) -> Self {
    match value {
      MouseEvent::Move { x, y } => Self::Move { x, y },
      MouseEvent::RelMove { x, y } => Self::RelMove { x, y },
      MouseEvent::Scroll { x, y } => Self::Scroll { x, y },
      MouseEvent::VerticalScroll { value } => Self::VerticalScroll { value },
      MouseEvent::HorizontalScroll { value } => Self::HorizontalScroll { value },
      MouseEvent::Button {
        x,
        y,
        button,
        pressed,
      } => Self::Button {
        x,
        y,
        pressed,
        button: button.into(),
      },
      MouseEvent::ButtonRel {
        x,
        y,
        button,
        pressed,
      } => Self::ButtonRel {
        x,
        y,
        pressed,
        button: button.into(),
      },
      _ => Self::Invariant,
    }
  }
}
