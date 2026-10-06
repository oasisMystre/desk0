use ironrdp::{
  core::Encode, pdu::{PduError, PduErrorKind}, svc::SvcEncode,
};

use crate::terminal::TerminalInput;

pub enum StaticChannelInput {
  Terminal(TerminalInput),
}

impl StaticChannelInput {
  const TERMINAL: u8 = 0;
}

impl TryFrom<&[u8]> for StaticChannelInput {
  type Error = PduError;

  fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
    let (discriminator, data) = value
      .split_first()
      .ok_or_else(|| PduError::new("empty data", PduErrorKind::Decode))?;

    let value = match *discriminator {
      Self::TERMINAL => Self::Terminal(TerminalInput::try_from(data)?),
      _ => {
        return Err(PduError::new(
          "unsupported discriminator: {discriminator}",
          PduErrorKind::Decode,
        ));
      }
    };

    Ok(value)
  }
}

impl Encode for StaticChannelInput {
  fn encode(&self, dst: &mut ironrdp::core::WriteCursor<'_>) -> ironrdp::core::EncodeResult<()> {
    let data = match self {
      StaticChannelInput::Terminal(data) => data,
    };

    dst.write_u8(Self::TERMINAL);
    data.encode(dst)?;

    Ok(())
  }

  fn name(&self) -> &'static str {
    "Desk0StaticChannelInput"
  }

  fn size(&self) -> usize {
    size_of::<u8>()
      + match self {
        Self::Terminal(data) => data.size(),
      }
  }
}


impl SvcEncode for StaticChannelInput {}