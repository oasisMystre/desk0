use std::collections::VecDeque;

use ipc::{IpcSharedBuffer, IpcSharedBufferField};
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub enum IpcPackedClipboardData {
  Text(String),
  Html(String),
  RichText(String),
  Image(IpcSharedBufferField),
  Files(Vec<String>),
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub enum IpcUnpackedClipboardData {
  Text(String),
  Html(String),
  RichText(String),
  Image(IpcSharedBuffer),
  Files(Vec<String>),
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub enum IpcClipboardEvent {
  Start,
  Stop,
  Data(IpcPackedClipboardData),
}

impl IpcPackedClipboardData {
  pub fn unpack(
    self,
     buffers: &mut VecDeque<IpcSharedBuffer>,
  ) -> anyhow::Result<IpcUnpackedClipboardData> {
    Ok(match self {
      IpcPackedClipboardData::Text(value) => IpcUnpackedClipboardData::Text(value),
      IpcPackedClipboardData::Html(value) => IpcUnpackedClipboardData::Html(value),
      IpcPackedClipboardData::Files(files) => IpcUnpackedClipboardData::Files(files),
      IpcPackedClipboardData::RichText(value) => IpcUnpackedClipboardData::RichText(value),
      IpcPackedClipboardData::Image(IpcSharedBufferField(index)) => {
        let buffer = buffers
          .pop_front()
          .ok_or_else(|| anyhow::anyhow!("expect image buffer in position {index}"))?;
        IpcUnpackedClipboardData::Image(buffer)
      }
    })
  }
}
