use ironrdp::{
  core::Encode,
  pdu::{PduError, PduErrorKind},
};

pub enum TerminalOutput {
  StdOut(Vec<u8>),
  StdErr(Vec<u8>),
}

impl TerminalOutput {
  const STDOUT: u8 = 0;
  const STDERR: u8 = 1;

  pub fn len(&self) -> usize {
    match self {
      TerminalOutput::StdOut(data) | TerminalOutput::StdErr(data) => data.len(),
    }
  }

  pub fn is_empty(&self) -> bool {
    match self {
      TerminalOutput::StdOut(data) | TerminalOutput::StdErr(data) => data.is_empty(),
    }
  }
}

impl TryFrom<&[u8]> for TerminalOutput {
  type Error = PduError;

  fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
    let (discriminator, data) = value
      .split_first()
      .ok_or_else(|| PduError::new("empty terminal message", PduErrorKind::Decode))?;

    let value = match *discriminator {
      Self::STDOUT => Self::StdOut(data.to_vec()),
      Self::STDERR => Self::StdErr(data.to_vec()),
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

impl Encode for TerminalOutput {
  fn encode(&self, dst: &mut ironrdp::core::WriteCursor<'_>) -> ironrdp::core::EncodeResult<()> {
    let (discriminator, data) = match self {
      TerminalOutput::StdOut(data) => (Self::STDOUT, data),
      TerminalOutput::StdErr(data) => (Self::STDERR, data),
    };

    dst.write_u8(discriminator);
    dst.write_slice(data);

    Ok(())
  }

  fn name(&self) -> &'static str {
    "Desk0TerminalOutput"
  }

  fn size(&self) -> usize {
    size_of::<u8>()
      + match self {
        Self::StdOut(data) | Self::StdErr(data) => data.len(),
      }
  }
}
