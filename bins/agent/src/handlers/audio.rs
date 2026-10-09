use std::sync::Arc;

use ironrdp::{
  rdpsnd::{
    pdu::AudioFormat,
    server::{NegotiatedFormat, RdpsndError, RdpsndServer},
  },
  server::{RdpsndServerHandler, StaticChannelFactory},
};

use crate::handlers::media::MediaHandler;

#[derive(Debug)]
pub struct AudioHandler;
pub struct AudioHandlerStaticChannelFactory {
  media: Arc<MediaHandler>,
}

impl AudioHandlerStaticChannelFactory {
  pub fn new(media: Arc<MediaHandler>) -> Self {
    Self { media }
  }
}

impl RdpsndServerHandler for AudioHandler {
  fn get_formats(&self) -> &[AudioFormat] {
    &[]
  }

  fn choose_format<'a>(&mut self, common: &'a [NegotiatedFormat]) -> Option<&'a NegotiatedFormat> {
    None
  }

  fn start(&mut self, format: &NegotiatedFormat) -> Result<(), Box<dyn RdpsndError>> {
    Ok(())
  }

  fn stop(&mut self) {}
}

impl StaticChannelFactory for AudioHandlerStaticChannelFactory {
  fn attach(&self, acceptor: &mut ironrdp::server::Acceptor) {
    acceptor.attach_static_channel(RdpsndServer::new(Box::new(AudioHandler)));
  }
}
