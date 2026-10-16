use std::fmt::Debug;

use rkyv::{Archive, Deserialize, Serialize};

mod audio;
mod clipboard;
mod display;

pub use audio::*;
pub use clipboard::*;
pub use display::*;

#[derive(Debug, Archive, Serialize, Deserialize)]
pub enum IpcOutboundEvent {
  Audio(IpcPackedAudioEvent),
  Display(IpcPackedDisplayEvent),
  Clipboard(IpcPackedClipboardEvent),
}
