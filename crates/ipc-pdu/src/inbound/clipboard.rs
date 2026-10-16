use std::collections::VecDeque;

use rkyv::{Archive, Deserialize, Serialize};
use ipc::{IpcSharedBuffer, IpcSharedBufferField};

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub enum IpcPackedClipboardEvent {
  Text(String),
  Html(String),
  RichText(String),
  Image(IpcSharedBufferField),
  Files(Vec<String>),
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub enum IpcUnpackedClipboardEvent {
  Text(String),
  Html(String),
  RichText(String),
  Image(IpcSharedBuffer),
  Files(Vec<String>),
}

impl IpcPackedClipboardEvent {
  pub fn unpack(
    self,
    mut buffers: VecDeque<IpcSharedBuffer>,
  ) -> anyhow::Result<IpcUnpackedClipboardEvent> {
    Ok(match self {
      IpcPackedClipboardEvent::Text(value) => IpcUnpackedClipboardEvent::Text(value),
      IpcPackedClipboardEvent::Html(value) => IpcUnpackedClipboardEvent::Html(value),
      IpcPackedClipboardEvent::Files(files) => IpcUnpackedClipboardEvent::Files(files),
      IpcPackedClipboardEvent::RichText(value) => IpcUnpackedClipboardEvent::RichText(value),
      IpcPackedClipboardEvent::Image(IpcSharedBufferField(index)) => {
        let buffer = buffers
          .pop_front()
          .ok_or_else(|| anyhow::anyhow!("expect image buffer in position {index}"))?;
        IpcUnpackedClipboardEvent::Image(buffer)
      }
    })
  }
}
