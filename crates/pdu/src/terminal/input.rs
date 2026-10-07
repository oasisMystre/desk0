use ironrdp::{
  core::{Decode, DecodeError, Encode, UnexpectedMessageTypeErr},
  svc::SvcEncode,
};

pub enum TerminalInputEvent {
  Create(u8),
  Output,
  Data {
    id: u8,
    data: Vec<u8>,
  },
  Resize {
    id: u8,
    rows: u16,
    cols: u16,
    pw: u16,
    ph: u16,
  },
}

impl TerminalInputEvent {
  const CREATE: u8 = 0;
  const OUTPUT: u8 = 1;
  const DATA: u8 = 2;
  const RESIZE: u8 = 3;
}

impl<'de> Decode<'de> for TerminalInputEvent {
  fn decode(src: &mut ironrdp::core::ReadCursor<'de>) -> ironrdp::core::DecodeResult<Self> {
    let discriminator = src.read_u8();

    let value = match discriminator {
      Self::CREATE => {
        let id = src.read_u8();
        Self::Create(id)
      }
      Self::OUTPUT => Self::Output,
      Self::DATA => {
        let id = src.read_u8();
        let length = src.read_u64() as usize;
        let data = src.read_slice(length);

        Self::Data {
          id,
          data: data.to_vec(),
        }
      }
      Self::RESIZE => {
        let id = src.read_u8();
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
  fn encode(&self, dst: &mut ironrdp::core::WriteCursor<'_>) -> ironrdp::core::EncodeResult<()> {
    match self {
      Self::Create(id) => {
        dst.write_u8(Self::CREATE);
        dst.write_u8(*id);
      }
      Self::Output => {
        dst.write_u8(Self::OUTPUT);
      }
      Self::Data { id, data } => {
        dst.write_u8(Self::DATA);
        dst.write_u8(*id);
        dst.write_u64(data.len() as u64);
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
        dst.write_u8(*id);
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
      Self::Output => 0,
      Self::Create(_) => size_of::<u8>(),
      Self::Resize { .. } => size_of::<u8>() + 4 * size_of::<u16>(),
      Self::Data { data, .. } => size_of::<u8>() + size_of::<u64>() + data.len(),
    };

    size_of::<u8>() + size
  }
}

impl SvcEncode for TerminalInputEvent {}
