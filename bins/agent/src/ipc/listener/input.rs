use std::sync::Arc;

use ipc_pdu::{
  Listener,
  inbound::{IpcInboundEvent, IpcInputEvent},
};
use ironrdp::server::{KeyboardEvent, MouseEvent, RdpServerInputHandler};

pub struct InputHandler {
  ipc: Arc<Listener>,
}

unsafe impl Send for InputHandler {}

impl InputHandler {
  pub fn new(ipc: Arc<Listener>) -> anyhow::Result<Self> {
    Ok(Self { ipc })
  }
}

impl RdpServerInputHandler for InputHandler {
  fn keyboard(&mut self, event: KeyboardEvent) {
    if let Err(error) = self
      .ipc
      .blocking_send(IpcInboundEvent::Input(IpcInputEvent::Keyboard(
        event.into(),
      )))
    {
      tracing::error!(?error, "failed send keyboard event");
    }
  }

  fn mouse(&mut self, event: MouseEvent) {
    if let Err(error) = self
      .ipc
      .blocking_send(IpcInboundEvent::Input(IpcInputEvent::Mouse(event.into())))
    {
      tracing::error!(?error, "failed send keyboard event");
    }
  }
}
