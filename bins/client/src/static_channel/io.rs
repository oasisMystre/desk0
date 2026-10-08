use std::{fmt::Debug, sync::Arc};

use ironrdp::{
  core::{decode, impl_as_any},
  pdu::{PduError, PduErrorExt, PduResult},
  svc::{SvcClientProcessor, SvcMessage, SvcProcessor},
};
use pdu::io::{IO_STATIC_CHANNEL_NAME, IoOutputEvent};

pub struct IoStaticChannel {
  callback: Arc<dyn Fn(IoOutputEvent) + Send + Sync + 'static>,
}

impl Debug for IoStaticChannel {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("IoStaticChannel").finish()
  }
}

impl IoStaticChannel {
  pub fn new<F>(callback: F) -> Self
  where
    F: Fn(IoOutputEvent) + Send + Sync + 'static,
  {
    Self {
      callback: Arc::new(callback),
    }
  }
}

impl_as_any!(IoStaticChannel);
impl SvcClientProcessor for IoStaticChannel {}

impl SvcProcessor for IoStaticChannel {
  fn channel_name(&self) -> ironrdp::pdu::gcc::ChannelName {
    IO_STATIC_CHANNEL_NAME
  }

  fn process(&mut self, payload: &[u8]) -> PduResult<Vec<SvcMessage>> {
    let payload = decode::<IoOutputEvent>(payload)
      .map_err(|error| PduError::decode("io static channel output", error))?;
    (self.callback)(payload);

    Ok(vec![])
  }
}
