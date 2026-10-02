use desk0_client::viewer::{Desk0Viewer, Desk0ViewerConfig};
use flutter_rust_bridge::frb;
use ironrdp::input::Operation;
use std::sync::Mutex;

use crate::{
  frb_generated::StreamSink,
  types::{InputEvent, SseDesktopSize, SseOutputEvent},
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

  pub fn connect(&mut self, events: StreamSink<SseOutputEvent>) -> anyhow::Result<()> {
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

  pub fn send_input_events(&mut self, events: Vec<InputEvent>) -> anyhow::Result<()> {
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

  pub fn stop(&mut self) {
    if let Ok(mut inner) = self.inner.lock() {
      if let Some(mut inner) = inner.take() {
        inner.shutdown();
      }
    }
  }
}
