use std::{fmt::Debug, sync::Arc};

use ironrdp::{
  core::{decode, impl_as_any},
  pdu::{PduError, PduErrorExt, PduResult},
  svc::{SvcClientProcessor, SvcMessage, SvcProcessor},
};
use pdu::terminal::{TERMINAL_STATIC_CHANNEL_NAME, TerminalOutputEvent};

pub struct TerminalStaticChannel {
  callback: Arc<dyn Fn(TerminalOutputEvent) + Send + Sync + 'static>,
}

impl Debug for TerminalStaticChannel {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("TerminalStaticChannel").finish()
  }
}

impl TerminalStaticChannel {
  pub fn new<T>(callback: T) -> Self
  where
    T: Fn(TerminalOutputEvent) + Send + Sync + 'static,
  {
    Self {
      callback: Arc::new(callback),
    }
  }
}

impl_as_any!(TerminalStaticChannel);
impl SvcClientProcessor for TerminalStaticChannel {}

impl SvcProcessor for TerminalStaticChannel {
  fn channel_name(&self) -> ironrdp::pdu::gcc::ChannelName {
    TERMINAL_STATIC_CHANNEL_NAME
  }

  fn process(&mut self, payload: &[u8]) -> PduResult<Vec<SvcMessage>> {
    let payload = decode::<TerminalOutputEvent>(payload)
      .map_err(|error| PduError::decode("static channel output", error))?;
    (self.callback)(payload);

    Ok(vec![])
  }
}
