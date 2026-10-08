use copypaste::cliprdr::types::clipboard_event::{ClipboardEvent, FileDownloadStatus};

#[repr(C)]
#[derive(Clone)]
pub enum SseFileDownloadStatus {
  Download {
    id: u32,
    total_size: u64,
  },
  Progress {
    id: u32,
    total_size: u64,
    received: u64,
  },
  Completed {
    id: u32,
  },
}

#[repr(C)]
#[derive(Clone)]
pub enum SseClipboardEvent {
  Text,
  Image,
  Html,
  RichText,
  File(SseFileDownloadStatus),
}

impl From<FileDownloadStatus> for SseFileDownloadStatus {
  fn from(value: FileDownloadStatus) -> Self {
    match value {
      FileDownloadStatus::Download { id, total_size } => {
        SseFileDownloadStatus::Download { id, total_size }
      }
      FileDownloadStatus::Progress {
        id,
        total_size,
        received,
      } => SseFileDownloadStatus::Progress {
        id,
        total_size,
        received,
      },
      FileDownloadStatus::Completed { id } => SseFileDownloadStatus::Completed { id },
    }
  }
}

impl From<ClipboardEvent> for SseClipboardEvent {
  fn from(value: ClipboardEvent) -> Self {
    match value {
      ClipboardEvent::Text => SseClipboardEvent::Text,
      ClipboardEvent::Image => SseClipboardEvent::Image,
      ClipboardEvent::Html => SseClipboardEvent::Html,
      ClipboardEvent::RichText => SseClipboardEvent::RichText,
      ClipboardEvent::File(status) => SseClipboardEvent::File(status.into()),
    }
  }
}
