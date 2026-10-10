use desk0_client::viewer::{Desk0Viewer, Desk0ViewerConfig, Desk0ViewerOutput};
use flutter_rust_bridge::frb;
use ironrdp::{
  client::rdp::{RdpInputEvent, RdpOutputEvent},
  input::Operation,
  svc::SvcMessage,
};
use pdu::terminal::TERMINAL_STATIC_CHANNEL_NAME;
use shared::init::init_tracing;
use std::sync::{
  Arc, Mutex,
  atomic::{AtomicBool, Ordering},
};
use tokio::sync::mpsc::error::TrySendError;

use crate::{
  SseClipboardEvent, SseInputEvent, SseIoOutputEvent, SseStaticChannelInputEvent,
  SseTerminalOutputEvent,
  frame::TextureHandle,
  frb_generated::StreamSink,
  types::{SseDesktopSize, SseOutputEvent},
};

#[frb(opaque)]
pub struct ViewerHandle {
  token: String,
  endpoint: String,
  size: SseDesktopSize,
  initialized: AtomicBool,
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
      initialized: AtomicBool::new(false),
      texture_handle: Arc::new(TextureHandle::new(size.width, size.height)),
    }
  }

  #[frb(sync)]
  pub fn create_texture(&self, engine_handle: i64) -> anyhow::Result<i64> {
    let id = self.texture_handle.create(engine_handle)?;
    Ok(id)
  }

  #[frb(sync)]
  pub fn connect(&self, events: Option<StreamSink<SseOutputEvent>>) -> anyhow::Result<()> {
    if self.initialized.load(Ordering::Acquire) {
      return Ok(());
    }

    let mut inner = self
      .inner
      .lock()
      .map_err(|_| anyhow::anyhow!("viewer lock poisoned"))?;

    if inner.is_some() {
      self.initialized.store(true, Ordering::Release);
      return Ok(());
    }

    let _ = init_tracing();

    let rt = tokio::runtime::Builder::new_multi_thread()
      .enable_all()
      .build()?;

    let viewer = Desk0Viewer::new(
      rt,
      Desk0ViewerConfig {
        token: &self.token,
        endpoint: &self.endpoint,
        desktop_size: Some(self.size.into()),
      },
      Desk0ViewerOutput::callback(),
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))?;

    let texture = self.texture_handle.clone();
    viewer.set_rdp_output_event_callback(move |event| match event {
      RdpOutputEvent::DesktopUpdate(update) => {
        let width = update.width().get();
        let height = update.height().get();

        texture.apply_desktop_update(update);
        if let Some(events) = &events {
          let _ = events.add(SseOutputEvent::Resize { width, height });
        }
      }
      RdpOutputEvent::Image {
        buffer,
        width,
        height,
      } => {
        let width = width.get();
        let height = height.get();

        texture.apply_image(width, height, &buffer);
        if let Some(events) = &events {
          let _ = events.add(SseOutputEvent::Resize { width, height });
        }
      }
      event => {
        if let Some(events) = &events {
          let _ = events.add(SseOutputEvent::from(event));
        }
      }
    });

    *inner = Some(viewer);
    self.initialized.store(true, Ordering::Release);

    Ok(())
  }

  pub fn on_clipboard_event(&self, events: StreamSink<SseClipboardEvent>) -> anyhow::Result<()> {
    let mut inner = self
      .inner
      .lock()
      .map_err(|_| anyhow::anyhow!("viewer lock poisoned"))?;
    let viewer = inner
      .as_mut()
      .ok_or_else(|| anyhow::anyhow!("viewer is not connected"))?;

    viewer.set_clipboard_event_callback(move |event| {
      let _ = events.add(event.into());
    });

    Ok(())
  }

  pub fn on_io_output_event_event(
    &self,
    events: StreamSink<SseIoOutputEvent>,
  ) -> anyhow::Result<()> {
    let mut inner = self
      .inner
      .lock()
      .map_err(|_| anyhow::anyhow!("viewer lock poisoned"))?;
    let viewer = inner
      .as_mut()
      .ok_or_else(|| anyhow::anyhow!("viewer is not connected"))?;

    viewer.set_io_output_event_callback(move |event| {
      let _ = events.add(event.into());
    });

    Ok(())
  }

  pub fn on_terminal_output_event_event(
    &self,
    events: StreamSink<SseTerminalOutputEvent>,
  ) -> anyhow::Result<()> {
    let mut inner = self
      .inner
      .lock()
      .map_err(|_| anyhow::anyhow!("viewer lock poisoned"))?;
    let viewer = inner
      .as_mut()
      .ok_or_else(|| anyhow::anyhow!("viewer is not connected"))?;

    viewer.set_terminal_output_event_callback(move |event| {
      let _ = events.add(event.into());
    });

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

  pub fn send_static_channel_input_events(
    &self,
    events: Vec<SseStaticChannelInputEvent>,
  ) -> anyhow::Result<()> {
    let mut inner = self
      .inner
      .lock()
      .map_err(|_| anyhow::anyhow!("viewer lock poisoned"))?;
    let viewer = inner
      .as_mut()
      .ok_or_else(|| anyhow::anyhow!("viewer is not connected"))?;

    for event in events {
      let message: SvcMessage = event.into();

      let data = message
        .encode_unframed_pdu()
        .map_err(|error| anyhow::anyhow!("encoding error: {error}"))?;

      match viewer.input.try_send(RdpInputEvent::SendStaticChannelData {
        data,
        channel_name: TERMINAL_STATIC_CHANNEL_NAME,
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
