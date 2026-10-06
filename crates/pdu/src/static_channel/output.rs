use ironrdp::{
  core::Encode, pdu::{PduError, PduErrorKind}, svc::SvcEncode,
};

use crate::terminal::{ TerminalOutput};

pub enum StaticChannelOutput {
  Terminal(TerminalOutput),
}

impl StaticChannelOutput {
  const TERMINAL: u8 = 0;
}

impl TryFrom<&[u8]> for StaticChannelOutput {
  type Error = PduError;

  fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
    let (discriminator, data) = value
      .split_first()
      .ok_or_else(|| PduError::new("empty data", PduErrorKind::Decode))?;

    let value = match *discriminator {
      Self::TERMINAL => Self::Terminal(TerminalOutput::try_from(data)?),
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

impl Encode for StaticChannelOutput {
  fn encode(&self, dst: &mut ironrdp::core::WriteCursor<'_>) -> ironrdp::core::EncodeResult<()> {
    let data = match self {
      StaticChannelOutput::Terminal(data) => data,
    };

    dst.write_u8(Self::TERMINAL);
    data.encode(dst)?;

    Ok(())
  }

  fn name(&self) -> &'static str {
    "Desk0StaticChannelOutput"
  }

  fn size(&self) -> usize {
    size_of::<u8>()
      + match self {
        Self::Terminal(data) => data.size(),
      }
  }
}

impl SvcEncode for StaticChannelOutput {}