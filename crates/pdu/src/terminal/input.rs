use ironrdp::{
  core::Encode,
  pdu::{PduError, PduErrorKind},
};

pub enum TerminalInput {
  StdIn(Vec<u8>),
  Output,
}

impl TerminalInput {
  const STDIN: u8 = 0;
  const OUTPUT: u8 = 0;

  pub fn len(&self) -> usize {
    match self {
      TerminalInput::StdIn(data) => data.len(),
      TerminalInput::Output => 0,
    }
  }
}

impl TryFrom<&[u8]> for TerminalInput {
  type Error = PduError;

  fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
    let (discriminator, data) = value
      .split_first()
      .ok_or_else(|| PduError::new("empty terminal message", PduErrorKind::Decode))?;

    let value = match *discriminator {
      Self::STDIN => Self::StdIn(data.to_vec()),
      _ => {
        return Err(PduError::new(
          "unsupported terminal discriminator: {discriminator}",
          PduErrorKind::Decode,
        ));
      }
    };

    Ok(value)
  }
}

impl Encode for TerminalInput {
  fn encode(&self, dst: &mut ironrdp::core::WriteCursor<'_>) -> ironrdp::core::EncodeResult<()> {
    let (discriminator, data) = match self {
      TerminalInput::StdIn(data) => (Self::STDIN, Some(data)),
      TerminalInput::Output => (Self::OUTPUT, None),
    };

    dst.write_u8(discriminator);
    if let Some(data) = data {
      dst.write_slice(data);
    }

    Ok(())
  }

  fn name(&self) -> &'static str {
    "Desk0TerminalInput"
  }

  fn size(&self) -> usize {
    size_of::<u8>() + self.len()
  }
}
