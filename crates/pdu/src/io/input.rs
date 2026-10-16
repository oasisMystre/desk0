use ironrdp::{core::{Decode, DecodeError, Encode, UnexpectedMessageTypeErr, cast_length}, svc::SvcEncode};

pub enum IoInputEvent {
  Poll,
  Download {
    id: u32,
    uri: Vec<u8>,
    path: Vec<u8>,
  },
}

impl IoInputEvent {
  const POLL: u8 = 0;
  const DOWNLOAD: u8 = 1;
}

impl<'de> Decode<'de> for IoInputEvent {
  fn decode(src: &mut ironrdp::core::ReadCursor<'de>) -> ironrdp::core::DecodeResult<Self> {
    let discriminator = src.read_u8();
    let value = match discriminator {
      Self::POLL => Self::Poll,
      Self::DOWNLOAD => {
        let id = src.read_u32();
        let uri_len: usize = cast_length!("uri_len", src.read_u16())?;
        let uri = src.read_slice(uri_len);
        let path_len: usize = cast_length!("path_len", src.read_u16())?;
        let path = src.read_slice(path_len);

        Self::Download {
          id,
          uri: uri.to_vec(),
          path: path.to_vec(),
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

impl Encode for IoInputEvent {
  fn encode(&self, dst: &mut ironrdp::core::WriteCursor<'_>) -> ironrdp::core::EncodeResult<()> {
    match self {
      Self::Poll => dst.write_u8(Self::POLL),
      IoInputEvent::Download { id, uri, path } => {
        dst.write_u8(Self::DOWNLOAD);
        dst.write_u32(*id);
        dst.write_u16(cast_length!("uri_len", uri.len(), in: dst)?);
        dst.write_slice(uri);
        dst.write_u16(cast_length!("path_len", uri.len(), in: dst)?);
        dst.write_slice(path);
      }
    };

    Ok(())
  }

  fn name(&self) -> &'static str {
    "IoInputEvent"
  }

  fn size(&self) -> usize {
    let size = match self {
      IoInputEvent::Poll => 0,
      IoInputEvent::Download { uri, path, .. } => {
        size_of::<u32>() + size_of::<u16>() + uri.len() + size_of::<u16>() + path.len()
      }
    };
    size_of::<u8>() + size
  }
}

impl SvcEncode for IoInputEvent {}