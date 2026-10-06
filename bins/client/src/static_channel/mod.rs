use std::{fmt::Debug, sync::Arc};

use ironrdp::{
  core::impl_as_any,
  pdu::{PduResult, gcc::ChannelName},
  svc::{SvcClientProcessor, SvcMessage, SvcProcessor},
};
use pdu::static_channel::StaticChannelOutput;

pub struct Desk0StaticChannel {
  callback: Arc<dyn Fn(StaticChannelOutput) + Send + Sync + 'static>,
}

impl Debug for Desk0StaticChannel {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Desk0StaticChannel").finish()
  }
}

impl Desk0StaticChannel {
  pub fn new<T>(callback: Arc<T>) -> Self
  where
    T: Fn(StaticChannelOutput) + Send + Sync + 'static,
  {
    Self { callback }
  }
}

impl_as_any!(Desk0StaticChannel);
impl SvcClientProcessor for Desk0StaticChannel {}

impl SvcProcessor for Desk0StaticChannel {
  fn channel_name(&self) -> ironrdp::pdu::gcc::ChannelName {
    ChannelName::new(*b"desk0123")
  }

  fn process(&mut self, payload: &[u8]) -> PduResult<Vec<SvcMessage>> {
    let payload = StaticChannelOutput::try_from(payload)?;
    (self.callback)(payload);

    Ok(vec![])
  }
}
