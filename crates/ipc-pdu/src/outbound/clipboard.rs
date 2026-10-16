use std::collections::VecDeque;

use ipc::{IpcSharedBuffer, IpcSharedBufferField};
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Archive, Serialize, Deserialize)]
pub struct IpcPackedClipboardSnapshot {
  pub text: Option<String>,
  pub html: Option<String>,
  pub rich_text: Option<String>,
  pub files: Option<Vec<String>>,
  pub image: Option<IpcSharedBufferField>,
}

#[derive(Debug, Archive, Serialize, Deserialize)]
pub struct IpcUnpackedClipboardSnapshot {
  pub text: Option<String>,
  pub html: Option<String>,
  pub rich_text: Option<String>,
  pub files: Option<Vec<String>>,
  pub image: Option<IpcSharedBuffer>,
}

#[derive(Debug, Archive, Serialize, Deserialize)]
pub enum IpcPackedClipboardEvent {
  Snapshot(IpcPackedClipboardSnapshot),
}

#[derive(Debug, Archive, Serialize, Deserialize)]
pub enum IpcUnpackedClipboardEvent {
  Snapshot(IpcUnpackedClipboardSnapshot),
}

impl IpcPackedClipboardSnapshot {
  pub fn unpack(
    self,
    buffers: &mut VecDeque<IpcSharedBuffer>,
  ) -> anyhow::Result<IpcUnpackedClipboardSnapshot> {
    let image = match self.image {
      Some(field) => Some(
        buffers
          .pop_front()
          .ok_or_else(|| anyhow::anyhow!("missing image buffer at {}", field.0))?,
      ),
      None => None,
    };

    Ok(IpcUnpackedClipboardSnapshot {
      image,
      text: self.text,
      html: self.html,
      files: self.files,
      rich_text: self.rich_text,
    })
  }
}

impl IpcPackedClipboardEvent {
  pub fn unpack(
    self,
    buffers: &mut VecDeque<IpcSharedBuffer>,
  ) -> anyhow::Result<IpcUnpackedClipboardEvent> {
    Ok(match self {
      IpcPackedClipboardEvent::Snapshot(snapshot) => {
        IpcUnpackedClipboardEvent::Snapshot(snapshot.unpack(buffers)?)
      }
    })
  }
}
