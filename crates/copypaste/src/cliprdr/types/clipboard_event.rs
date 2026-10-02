#[derive(Clone)]
pub enum FileDownloadStatus {
  Initial { id: u32, total_size: u64 },
  Progress { id: u32, received: u64 },
  Completed { id: u32 },
}

#[derive(Clone)]
pub enum ClipboardEvent {
  Text,
  Image,
  Html,
  RichText,
  File(FileDownloadStatus),
}
