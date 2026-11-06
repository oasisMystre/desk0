use ironrdp::{
  core::{
    Decode, DecodeError, DecodeResult, Encode, EncodeResult, ReadCursor, UnexpectedMessageTypeErr,
    WriteCursor, cast_length,
  },
  svc::SvcEncode,
};

pub enum IoOutputEvent {
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

impl IoOutputEvent {
  const DOWNLOAD_STARTED: u8 = 0;
  const DOWNLOAD_PROGRESS: u8 = 1;
  const DOWNLOAD_COMPLETED: u8 = 2;
  const DOWNLOAD_FAILED: u8 = 3;
}

impl<'de> Decode<'de> for IoOutputEvent {
  fn decode(src: &mut ReadCursor<'de>) -> DecodeResult<Self> {
    let discriminator = src.read_u8();
    let value = match discriminator {
      Self::DOWNLOAD_STARTED => {
        let id = src.read_u32();
        Self::DownloadStarted(id)
      }
      Self::DOWNLOAD_PROGRESS => {
        let id = src.read_u32();
        let downloaded = src.read_u64();
        let total = src.try_read_u64().ok();
        Self::DownloadProgress {
          id,
          downloaded,
          total,
        }
      }
      Self::DOWNLOAD_COMPLETED => {
        let id = src.read_u32();
        let path_len: usize = cast_length!("path_len", src.read_u16())?;
        let path = src.read_slice(path_len);
        Self::DownloadCompleted {
          id,
          path: path.to_vec(),
        }
      }
      Self::DOWNLOAD_FAILED => {
        let id = src.read_u32();
        let description_len: usize = cast_length!("description_len", src.read_u16())?;
        let description = src.read_slice(description_len);
        Self::DownloadFailed {
          id,
          description: description.to_vec(),
        }
      }
      _ => {
        return Err(DecodeError::unexpected_message_type(
          "unsupported io output discriminator",
          discriminator,
          Some(0),
        ));
      }
    };

    Ok(value)
  }
}

impl Encode for IoOutputEvent {
  fn encode(&self, dst: &mut WriteCursor<'_>) -> EncodeResult<()> {
    match self {
      IoOutputEvent::DownloadStarted(id) => {
        dst.write_u8(Self::DOWNLOAD_STARTED);
        dst.write_u32(*id);
      }
      IoOutputEvent::DownloadProgress {
        id,
        downloaded,
        total,
      } => {
        dst.write_u8(Self::DOWNLOAD_PROGRESS);
        dst.write_u32(*id);
        dst.write_u64(*downloaded);
        if let Some(total) = total {
          dst.write_u64(*total);
        }
      }
      IoOutputEvent::DownloadCompleted { id, path } => {
        dst.write_u8(Self::DOWNLOAD_COMPLETED);
        dst.write_u32(*id);
        dst.write_u16(cast_length!("path_len", path.len(), in: dst)?);
        dst.write_slice(path);
      }
      IoOutputEvent::DownloadFailed { id, description } => {
        dst.write_u8(Self::DOWNLOAD_FAILED);
        dst.write_u32(*id);
        dst.write_u16(cast_length!("description_len", description.len(), in: dst)?);
        dst.write_slice(description);
      }
    }

    Ok(())
  }

  fn name(&self) -> &'static str {
    "IoOutputEvent"
  }

  fn size(&self) -> usize {
    let size = match self {
      IoOutputEvent::DownloadStarted(_) => size_of::<u32>(),
      IoOutputEvent::DownloadProgress { total, .. } => {
        size_of::<u32>() + size_of::<u64>() + if total.is_some() { size_of::<u64>() } else { 0 }
      }
      IoOutputEvent::DownloadCompleted { path, .. } => {
        size_of::<u32>() + size_of::<u16>() + path.len()
      }
      IoOutputEvent::DownloadFailed { description, .. } => {
        size_of::<u32>() + size_of::<u16>() + description.len()
      }
    };

    size_of::<u8>() + size
  }
}

impl SvcEncode for IoOutputEvent {}
