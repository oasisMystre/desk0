use std::collections::VecDeque;

use anyhow::Ok;
use ipc::{IpcSharedBuffer, IpcSharedBufferField};
use ironrdp::server::DesktopSize;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Archive, Serialize, Deserialize, Clone, Copy)]
pub struct IpcDesktopSize {
  pub width: u16,
  pub height: u16,
}

#[derive(Clone, Copy, Debug, Archive, Serialize, Deserialize)]
pub struct IpcDisplayInfo {
  pub monitors: u32,
  pub size: IpcDesktopSize,
}

#[derive(Debug, Archive, Serialize, Deserialize)]
pub struct IpcDamageRect {
  pub x: i32,
  pub y: i32,
  pub width: u32,
  pub height: u32,
}

#[derive(Debug, Archive, Serialize, Deserialize)]
pub struct IpcPackedBitmapUpdate {
  pub x: u16,
  pub y: u16,
  pub width: u32,
  pub height: u32,
  pub source: String,
  pub first_frame: bool,
  pub pixels: IpcSharedBufferField,
  pub damage: Option<IpcSharedBufferField>,
}

#[derive(Debug, Archive, Serialize, Deserialize)]
pub struct IpcUnpackedBitmapUpdate {
  pub x: u16,
  pub y: u16,
  pub width: u32,
  pub height: u32,
  pub source: String,
  pub first_frame: bool,
  pub pixels: IpcSharedBuffer,
  pub damage: Option<IpcSharedBuffer>,
}

#[derive(Debug, Archive, Serialize, Deserialize)]
pub enum IpcPackedDisplayEvent {
  DisplayInfo(IpcDisplayInfo),
  BitmapUpdate(IpcPackedBitmapUpdate),
}

#[derive(Debug, Archive, Serialize, Deserialize)]
pub enum IpcUnpackedDisplayEvent {
  DisplayInfo(IpcDisplayInfo),
  BitmapUpdate(IpcUnpackedBitmapUpdate),
}

impl From<IpcDesktopSize> for DesktopSize {
  fn from(value: IpcDesktopSize) -> Self {
    Self {
      width: value.width,
      height: value.height,
    }
  }
}

impl From<IpcDamageRect> for pinray::Rect {
  fn from(value: IpcDamageRect) -> Self {
    Self {
      x: value.x,
      y: value.y,
      width: value.width,
      height: value.height,
    }
  }
}

impl IpcPackedBitmapUpdate {
  pub fn unpack(
    self,
    buffers: &mut VecDeque<IpcSharedBuffer>,
  ) -> anyhow::Result<IpcUnpackedBitmapUpdate> {
    let pixels = buffers
      .pop_front()
      .ok_or_else(|| anyhow::anyhow!("missing pixels buffer at {}", self.pixels.0))?;
    let damage = buffers.pop_front();

    Ok(IpcUnpackedBitmapUpdate {
      pixels,
      damage,
      x: self.x,
      y: self.y,
      width: self.width,
      height: self.height,
      source: self.source,
      first_frame: self.first_frame,
    })
  }
}

impl IpcPackedDisplayEvent {
  pub fn unpack(
    self,
    buffers: &mut VecDeque<IpcSharedBuffer>,
  ) -> anyhow::Result<IpcUnpackedDisplayEvent> {
    Ok(match self {
      IpcPackedDisplayEvent::DisplayInfo(info) => IpcUnpackedDisplayEvent::DisplayInfo(info),
      IpcPackedDisplayEvent::BitmapUpdate(update) => {
        IpcUnpackedDisplayEvent::BitmapUpdate(update.unpack(buffers)?)
      }
    })
  }
}
