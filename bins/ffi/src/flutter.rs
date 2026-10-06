use desk0_client::viewer::{Desk0Viewer, Desk0ViewerConfig};
use flutter_rust_bridge::frb;
use ironrdp::{
  client::rdp::{RdpInputEvent, RdpOutputEvent},
  input::Operation,
  pdu::gcc::ChannelName,
  svc::SvcMessage,
};
use shared::init::init_tracing;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::error::TrySendError;

use crate::{
  SseInputEvent, SseStaticChannelInput,
  frame::TextureHandle,
  frb_generated::StreamSink,
  types::{SseDesktopSize, SseOutputEvent},
};

#[frb(opaque)]
pub struct ViewerHandle {
  token: String,
  endpoint: String,
  size: SseDesktopSize,
  inner: Mutex<Option<Desk0Viewer>>,

  // texture
  texture_handle: Arc<TextureHandle>,
}

impl ViewerHandle {
  #[frb(sync)]
  pub fn new(token: String, endpoint: String, size: SseDesktopSize) -> ViewerHandle {
    ViewerHandle {
      size,
      token,
      endpoint,
      inner: Mutex::new(None),
      texture_handle: Arc::new(TextureHandle::new(size.width, size.height)),
    }
  }

  #[frb(sync)]
  pub fn create_texture(&self, engine_handle: i64) -> anyhow::Result<i64> {
    let id = self.texture_handle.create(engine_handle)?;
    Ok(id)
  }

  pub fn connect(&self, events: StreamSink<SseOutputEvent>) -> anyhow::Result<()> {
    init_tracing()?;
    let clipboard_events = events.clone();
    let static_channel_events = events.clone();
    let texture = self.texture_handle.clone();

    let viewer = Desk0Viewer::with_callback(
      Desk0ViewerConfig {
        token: &self.token,
        endpoint: &self.endpoint,
        desktop_size: Some(winit::dpi::PhysicalSize {
          width: self.size.width,
          height: self.size.height,
        }),
      },
      move |event| match event {
        RdpOutputEvent::DesktopUpdate(update) => {
          let width = update.width().get();
          let height = update.height().get();

          texture.apply_desktop_update(update);
          let _ = events.add(SseOutputEvent::Resize { width, height });
        }
        RdpOutputEvent::Image {
          buffer,
          width,
          height,
        } => {
          let width = width.get();
          let height = height.get();

          texture.apply_image(width, height, &buffer);
          let _ = events.add(SseOutputEvent::Resize { width, height });
        }
        event => {
          let _ = events.add(SseOutputEvent::from(event));
        }
      },
      move |event| {
        let _ = clipboard_events.add(SseOutputEvent::Clipboard(event.into()));
      },
      move |event| {
        let _ = static_channel_events.add(SseOutputEvent::StaticChannel(event.into()));
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

  pub fn send_static_channel_events(
    &mut self,
    events: Vec<SseStaticChannelInput>,
  ) -> anyhow::Result<()> {
    let mut inner = self
      .inner
      .lock()
      .map_err(|_| anyhow::anyhow!("viewer lock poisoned"))?;
    let viewer = inner
      .as_mut()
      .ok_or_else(|| anyhow::anyhow!("viewer is not connected"))?;

    for event in events {
      let message: pdu::static_channel::StaticChannelInput = event.into();
      let message: SvcMessage = message.into();

      let data = message
        .encode_unframed_pdu()
        .map_err(|error| anyhow::anyhow!("encoding error: {error}"))?;

      match viewer.input.try_send(RdpInputEvent::SendStaticChannelData {
        data,
        channel_name: ChannelName::from_static(b"desk0123"),
      }) {
        Ok(()) => {}
        Err(TrySendError::Full(_)) => {
          tracing::warn!("rdp input queue full");
        }
        Err(TrySendError::Closed(_)) => {
          tracing::error!("rdp input channel closed");
        }
      }
    }

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

    let event = RdpInputEvent::Resize {
      width,
      height,
      scale_factor,
      physical_size,
    };
    if let Err(error) = viewer.input.try_send(event) {
      tracing::error!(?error, "sending resize event failed");
    };

    Ok(())
  }
}
