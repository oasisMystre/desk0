use ironrdp::{
  core::{
    Decode, DecodeError, DecodeResult, Encode, EncodeResult, ReadCursor, UnexpectedMessageTypeErr,
    WriteCursor, cast_length,
  },
  svc::SvcEncode,
};

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Mode {
  Sync = 0,
  Async = 1,
}

pub enum TerminalInputEvent {
  Poll,
  Create(u32),
  Data {
    id: u32,
    mode: Mode,
    data: Vec<u8>,
  },
  Exit(u32),
  Resize {
    id: u32,
    rows: u16,
    cols: u16,
    pw: u16,
    ph: u16,
  },
}

impl Into<u8> for Mode {
  fn into(self) -> u8 {
    match self {
      Mode::Sync => 0,
      Mode::Async => 1,
    }
  }
}

impl TryInto<Mode> for u8 {
  type Error = DecodeError;

  fn try_into(self) -> Result<Mode, Self::Error> {
    let value = match self {
      0 => Mode::Sync,
      1 => Mode::Async,
      _ => {
        return Err(DecodeError::unexpected_message_type(
          "invalid mode",
          self,
          None,
        ));
      }
    };

    Ok(value)
  }
}

impl TerminalInputEvent {
  const POLL: u8 = 0;
  const DATA: u8 = 1;
  const EXIT: u8 = 2;
  const RESIZE: u8 = 3;
  const CREATE: u8 = 4;
}

impl<'de> Decode<'de> for TerminalInputEvent {
  fn decode(src: &mut ReadCursor<'de>) -> DecodeResult<Self> {
    let discriminator = src.read_u8();

    let value = match discriminator {
      Self::POLL => Self::Poll,
      Self::EXIT => {
        let id = src.read_u32();
        Self::Exit(id)
      }
      Self::CREATE => {
        let id = src.read_u32();
        Self::Create(id)
      }
      Self::DATA => {
        let id = src.read_u32();
        let data_len: usize = cast_length!("data_len", src.read_u16())?;
        let data = src.read_slice(data_len);
        let mode: Mode = src.read_u8().try_into()?;

        Self::Data {
          id,
          mode,
          data: data.to_vec(),
        }
      }
      Self::RESIZE => {
        let id = src.read_u32();
        let rows = src.read_u16();
        let cols = src.read_u16();
        let pw = src.read_u16();
        let ph = src.read_u16();

        Self::Resize {
          id,
          rows,
          cols,
          pw,
          ph,
        }
      }
      _ => {
        return Err(DecodeError::unexpected_message_type(
          "unsupported terminal input discriminator",
          discriminator,
          Some(0),
        ));
      }
    };

    Ok(value)
  }
}

impl Encode for TerminalInputEvent {
  fn encode(&self, dst: &mut WriteCursor<'_>) -> EncodeResult<()> {
    match self {
      Self::Poll => {
        dst.write_u8(Self::POLL);
      }
      Self::Create(id) => {
        dst.write_u8(Self::CREATE);
        dst.write_u32(*id);
      }
      Self::Exit(id) => {
        dst.write_u8(Self::EXIT);
        dst.write_u32(*id);
      }
      Self::Data { id, data, mode } => {
        dst.write_u8(Self::DATA);
        dst.write_u32(*id);
        dst.write_u8(mode.clone().into());
        dst.write_u16(cast_length!("data_len", data.len(), in: dst)?);
        dst.write_slice(data);
      }
      Self::Resize {
        id,
        rows,
        cols,
        pw,
        ph,
      } => {
        dst.write_u8(Self::RESIZE);
        dst.write_u32(*id);
        dst.write_u16(*rows);
        dst.write_u16(*cols);
        dst.write_u16(*pw);
        dst.write_u16(*ph);
      }
    };

    Ok(())
  }

  fn name(&self) -> &'static str {
    "Desk0TerminalInput"
  }

  fn size(&self) -> usize {
    let size = match self {
      Self::Poll => 0,
      Self::Create(_) => size_of::<u32>(),
      Self::Exit(_) => size_of::<u32>(),
      Self::Resize { .. } => size_of::<u32>() + 4 * size_of::<u16>(),
      Self::Data { data, .. } => size_of::<u32>() + size_of::<u8>() + size_of::<u16>() + data.len(),
    };

    size_of::<u8>() + size
  }
}

impl SvcEncode for TerminalInputEvent {}
