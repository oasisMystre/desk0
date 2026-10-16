use ironrdp::{
  core::{
    Decode, DecodeError, DecodeResult, Encode, EncodeResult, ReadCursor, UnexpectedMessageTypeErr,
    WriteCursor, cast_length,
  },
  svc::SvcEncode,
};

#[repr(i32)]
#[derive(Clone, Copy)]
pub enum TerminalError {
  SpawnFailed = 1,
  ReadFailed = 2,
  WriteFailed = 3,
  ResizeFailed = 4,
  IoFailed(i32),
}

pub enum TerminalOutputEvent {
  Connected(u32),
  Exit {
    id: u32,
    code: i32,
    description: Option<String>,
  },
  Error {
    id: u32,
    code: TerminalError,
    description: Option<String>,
  },
  Data {
    id: u32,
    data: Vec<u8>,
  },
}

impl From<i32> for TerminalError {
  fn from(value: i32) -> Self {
    match value {
      1 => Self::SpawnFailed,
      2 => Self::ReadFailed,
      3 => Self::WriteFailed,
      4 => Self::ResizeFailed,
      value => Self::IoFailed(value),
    }
  }
}

impl From<TerminalError> for i32 {
  fn from(value: TerminalError) -> Self {
    match value {
      TerminalError::SpawnFailed => 1,
      TerminalError::ReadFailed => 2,
      TerminalError::WriteFailed => 3,
      TerminalError::ResizeFailed => 4,
      TerminalError::IoFailed(value) => value,
    }
  }
}

impl TerminalOutputEvent {
  const DATA: u8 = 0;
  const EXIT: u8 = 1;
  const ERROR: u8 = 2;
  const CONNECTED: u8 = 3;
}

impl<'de> Decode<'de> for TerminalOutputEvent {
  fn decode(src: &mut ReadCursor<'de>) -> DecodeResult<Self> {
    let discriminator = src.read_u8();

    let value = match discriminator {
      Self::DATA => {
        let id = src.read_u32();
        let data_len: usize = cast_length!("data_len", src.read_u16())?;
        let data = src.read_slice(data_len);

        Self::Data {
          id,
          data: data.to_vec(),
        }
      }
      Self::CONNECTED => {
        let id = src.read_u32();
        Self::Connected(id)
      }
      Self::EXIT | Self::ERROR => {
        let id = src.read_u32();
        let code = src.read_i32();
        let raw_description_len = src.try_read_u16().ok();
        let description = raw_description_len
          .map(|len| {
            let len: usize = cast_length!("description_len", len)?;
            let description = src.read_slice(len);
            Ok(if description.is_empty() {
              None
            } else {
              Some(String::from_utf8_lossy(description).to_string())
            })
          })
          .transpose()?
          .flatten();

        if discriminator == Self::EXIT {
          Self::Exit {
            id,
            code,
            description,
          }
        } else {
          Self::Error {
            id,
            description,
            code: code.into(),
          }
        }
      }
      _ => {
        return Err(DecodeError::unexpected_message_type(
          "unsupported terminal output discriminator",
          discriminator,
          Some(0),
        ));
      }
    };

    Ok(value)
  }
}

impl Encode for TerminalOutputEvent {
  fn encode(&self, dst: &mut WriteCursor<'_>) -> EncodeResult<()> {
    match self {
      Self::Data { id, data } => {
        dst.write_u8(Self::DATA);
        dst.write_u32(*id);
        dst.write_u16(cast_length!("data_len", data.len(), in: dst)?);
        dst.write_slice(data);
      }
      Self::Exit {
        id,
        code,
        description,
      } => {
        dst.write_u8(Self::EXIT);
        dst.write_u32(*id);
        dst.write_i32(*code);
        if let Some(description) = description {
          dst.write_u16(cast_length!("description_len", description.len(), in: dst)?);
          dst.write_slice(description.as_bytes());
        }
      }
      Self::Error {
        id,
        code,
        description,
      } => {
        dst.write_u8(Self::ERROR);
        dst.write_u32(*id);
        dst.write_i32((*code).into());
        if let Some(description) = description {
          dst.write_u16(cast_length!("description_len", description.len(), in: dst)?);
          dst.write_slice(description.as_bytes());
        }
      }
      Self::Connected(id) => {
        dst.write_u8(Self::CONNECTED);
        dst.write_u32(*id);
      }
    };

    Ok(())
  }

  fn name(&self) -> &'static str {
    "Desk0TerminalOutput"
  }

  fn size(&self) -> usize {
    let size = match self {
      TerminalOutputEvent::Connected(_) => size_of::<u8>(),
      TerminalOutputEvent::Data { data, .. } => size_of::<u8>() + size_of::<u16>() + data.len(),
      TerminalOutputEvent::Exit { description, .. }
      | TerminalOutputEvent::Error { description, .. } => {
        size_of::<u8>()
          + size_of::<i32>()
          + size_of::<u16>()
          + description.as_ref().map(|value| value.len()).unwrap_or(0)
      }
    };

    size_of::<u8>() + size
  }
}

impl SvcEncode for TerminalOutputEvent {}
