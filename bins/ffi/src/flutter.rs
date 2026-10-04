use desk0_client::viewer::{Desk0Viewer, Desk0ViewerConfig};
use flutter_rust_bridge::frb;
use ironrdp::{client::rdp::RdpInputEvent, input::Operation};
use shared::init::init_tracing;
use std::sync::Mutex;

use crate::{
  SseInputEvent,
  frb_generated::StreamSink,
  types::{SseDesktopSize, SseOutputEvent},
};

#[frb(opaque)]
pub struct ViewerHandle {
  token: String,
  endpoint: String,
  size: Option<SseDesktopSize>,
  inner: Mutex<Option<Desk0Viewer>>,
}

impl ViewerHandle {
  #[frb(sync)]
  pub fn new(token: String, endpoint: String, size: Option<SseDesktopSize>) -> ViewerHandle {
    ViewerHandle {
      size,
      token,
      endpoint,
      inner: Mutex::new(None),
    }
  }

  pub fn connect(&self, events: StreamSink<SseOutputEvent>) -> anyhow::Result<()> {
    init_tracing()?;

    let clipboard_events = events.clone();
    let viewer = Desk0Viewer::with_callback(
      Desk0ViewerConfig {
        token: &self.token,
        endpoint: &self.endpoint,
        desktop_size: self.size.map(|value| winit::dpi::PhysicalSize {
          width: value.width,
          height: value.height,
        }),
      },
      move |event| {
        let _ = events.add(SseOutputEvent::from(event));
      },
      move |event| {
        let _ = clipboard_events.add(SseOutputEvent::Clipboard(event.into()));
      },
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))?;

    *self
      .inner
      .lock()
      .map_err(|_| anyhow::anyhow!("viewer lock poisoned"))? = Some(viewer);
    Ok(())
  }

  pub fn send_input_events(&mut self, events: Vec<SseInputEvent>) -> anyhow::Result<()> {
    let mut inner = self
      .inner
      .lock()
      .map_err(|_| anyhow::anyhow!("viewer lock poisoned"))?;
    let viewer = inner
      .as_mut()
      .ok_or_else(|| anyhow::anyhow!("viewer is not connected"))?;
    let operations: Vec<Operation> = events.into_iter().map(Into::into).collect();
    viewer.apply_and_send_fast_path_events(operations);

    Ok(())
  }

  pub fn resize(
    &self,
    width: u16,
    height: u16,
    scale_factor: u32,
    physical_size: Option<(u32, u32)>,
  ) -> anyhow::Result<()> {
    let mut inner = self
      .inner
      .lock()
      .map_err(|_| anyhow::anyhow!("viewer lock poisoned"))?;
    let viewer = inner
      .as_mut()
      .ok_or_else(|| anyhow::anyhow!("viewer is not connected"))?;

    if let Err(error) = viewer.input.try_send(RdpInputEvent::Resize {
      width,
      height,
      scale_factor,
      physical_size,
    }) {
      tracing::error!(?error, "sending resize event failed");
    };

    Ok(())
  }
}
