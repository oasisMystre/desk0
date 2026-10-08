#[derive(Clone)]
pub enum SseIoInputEvent {
  Poll,
  Download {
    id: u32,
    uri: Vec<u8>,
    path: Vec<u8>,
  },
}

#[derive(Clone)]
pub enum SseIoOutputEvent {
  DownloadStarted(u32),
  DownloadProgress {
    id: u32,
    downloaded: u64,
    total: Option<u64>,
  },
  DownloadCompleted {
    id: u32,
    path: Vec<u8>,
  },
  DownloadFailed {
    id: u32,
    description: Vec<u8>,
  },
}

impl Into<pdu::io::IoInputEvent> for SseIoInputEvent {
  fn into(self) -> pdu::io::IoInputEvent {
    match self {
      SseIoInputEvent::Poll => pdu::io::IoInputEvent::Poll,
      SseIoInputEvent::Download { id, uri, path } => {
        pdu::io::IoInputEvent::Download { id, uri, path }
      }
    }
  }
}

impl From<pdu::io::IoOutputEvent> for SseIoOutputEvent {
  fn from(value: pdu::io::IoOutputEvent) -> Self {
    match value {
      pdu::io::IoOutputEvent::DownloadStarted(id) => Self::DownloadStarted(id),
      pdu::io::IoOutputEvent::DownloadProgress {
        id,
        total,
        downloaded,
      } => Self::DownloadProgress {
        id,
        total,
        downloaded,
      },
      pdu::io::IoOutputEvent::DownloadCompleted { id, path } => {
        Self::DownloadCompleted { id, path }
      }
      pdu::io::IoOutputEvent::DownloadFailed { id, description } => {
        Self::DownloadFailed { id, description }
      }
    }
  }
}
