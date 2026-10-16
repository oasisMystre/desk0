use rkyv::{Archive, Deserialize, Serialize};

mod audio;
mod clipboard;
mod display;
mod input;

pub use audio::IpcAudioEvent;
pub use clipboard::{IpcPackedClipboardEvent, IpcUnpackedClipboardEvent};
pub use display::IpcDisplayEvent;
pub use input::{IpcKeyboardEvent, IpcMouseButton, IpcMouseEvent};

#[derive(Debug, Archive, Serialize, Deserialize)]
pub enum IpcInputEvent {
  Stop,
  Mouse(IpcMouseEvent),
  Keyboard(IpcKeyboardEvent),
}

#[derive(Debug, Archive, Serialize, Deserialize)]
pub enum IpcInboundEvent {
  Input(IpcInputEvent),
  Audio(IpcAudioEvent),
  Display(IpcDisplayEvent),
  Clipboard(IpcPackedClipboardEvent),
}
