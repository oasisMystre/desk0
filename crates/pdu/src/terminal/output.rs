use ironrdp::{
  core::{Decode, DecodeError, Encode, UnexpectedMessageTypeErr},
  svc::SvcEncode,
};

pub enum TerminalOutputEvent {
  Connected(u8),
  Exit { id: u8, code: u8 },
  Data { id: u8, data: Vec<u8> },
}

impl TerminalOutputEvent {
  const DATA: u8 = 0;
  const EXIT: u8 = 1;
  const CONNECTED: u8 = 2;
}

impl<'de> Decode<'de> for TerminalOutputEvent {
  fn decode(src: &mut ironrdp::core::ReadCursor<'de>) -> ironrdp::core::DecodeResult<Self> {
    let discriminator = src.read_u8();

    let value = match discriminator {
      Self::DATA => {
        let id = src.read_u8();
        let length = src.read_u64();
        let data = src.read_slice(length as usize);

        Self::Data {
          id,
          data: data.to_vec(),
        }
      }
      Self::EXIT => {
        let id = src.read_u8();
        let code = src.read_u8();
        Self::Exit { id, code }
      }
      Self::CONNECTED => {
        let id = src.read_u8();
        Self::Connected(id)
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
  fn encode(&self, dst: &mut ironrdp::core::WriteCursor<'_>) -> ironrdp::core::EncodeResult<()> {
    match self {
      Self::Data { id, data } => {
        dst.write_u8(Self::DATA);
        dst.write_u8(*id);
        dst.write_u64(data.len() as u64);
        dst.write_slice(data);
      }
      Self::Exit { id, code } => {
        dst.write_u8(Self::EXIT);
        dst.write_u8(*id);
        dst.write_u8(*code);
      }
      Self::Connected(id) => {
        dst.write_u8(Self::CONNECTED);
        dst.write_u8(*id);
      }
    };

    Ok(())
  }

  fn name(&self) -> &'static str {
    "Desk0TerminalOutput"
  }

  fn size(&self) -> usize {
    let size = match self {
      TerminalOutputEvent::Exit { .. } => 2 * size_of::<u8>(),
      TerminalOutputEvent::Connected(_) => size_of::<u8>(),
      TerminalOutputEvent::Data { data, .. } => size_of::<u8>() + size_of::<u64>() + data.len(),
    };

    size_of::<u8>() + size
  }
}

impl SvcEncode for TerminalOutputEvent {}
