#[repr(C)]
#[derive(Clone)]
pub enum FileDownloadStatus {
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
pub enum ClipboardEvent {
  Text,
  Image,
  Html,
  RichText,
  File(FileDownloadStatus),
}
