use std::sync::Arc;

use ipc::{IpcSharedBufferField, encode};
use ipc_pdu::{
  Connector,
  outbound::{IpcDamageRect, IpcOutboundEvent, IpcPackedBitmapUpdate, IpcPackedDisplayEvent},
};
use tokio::{sync::mpsc::UnboundedReceiver, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::media::WrapVideoFrame;

pub struct IpcDisplayHandler {
  ipc: Arc<Connector>,

  handle: Option<JoinHandle<()>>,
  cancellation_token: CancellationToken,
}

impl IpcDisplayHandler {
  pub fn new(ipc: Arc<Connector>, cancellation_token: CancellationToken) -> Self {
    Self {
      ipc,
      handle: None,
      cancellation_token,
    }
  }

  pub fn capture(&mut self, mut video_frame_receiver: UnboundedReceiver<WrapVideoFrame>) {
    let ipc = self.ipc.clone();
    let cancellation_token = self.cancellation_token.clone();
    let handle = tokio::spawn(async move {
      'capture: loop {
        let frame = tokio::select! {
            _ = cancellation_token.cancelled() => break 'capture,
            result = video_frame_receiver.recv() => match result {
                Some(frame) => frame,
                None => break 'capture,
            },
        };

        let Some(pixels) = frame.inner.to_tight_bytes() else {
          tracing::trace!("dropping frame with unsupported pixel data");
          continue;
        };

        let mut damages: Option<Vec<IpcDamageRect>> = frame.inner.damage.as_deref().map(|rects| {
          rects
            .iter()
            .map(|rect| IpcDamageRect {
              x: rect.x,
              y: rect.y,
              width: rect.width,
              height: rect.height,
            })
            .collect()
        });

        let damages_bytes = match damages.take().map(encode).transpose() {
          Ok(damages) => damages,
          Err(error) => {
            tracing::warn!(?error, "Failed to encode damage rectangles");
            None
          }
        };

        let mut buffers = vec![pixels.as_slice()];

        if let Some(ref damages_bytes) = damages_bytes {
          buffers.push(damages_bytes.as_slice());
        }

        if let Err(error) = ipc
          .send_with_shared_buffers(
            IpcOutboundEvent::Display(IpcPackedDisplayEvent::BitmapUpdate(IpcPackedBitmapUpdate {
              x: frame.monitor.x,
              y: frame.monitor.y,
              width: frame.inner.width,
              height: frame.inner.height,
              source: frame.monitor.source.0,
              first_frame: frame.first_frame,
              pixels: IpcSharedBufferField(0),
              damage: damages_bytes.as_ref().map(|_| IpcSharedBufferField(1)),
            })),
            &buffers,
          )
          .await
        {
          tracing::error!(?error, "sending bitmap updates failed");
        }
      }
    });

    self.handle = Some(handle);
  }

  pub async fn stop(&mut self) {
    self.cancellation_token.cancel();

    if let Some(handle) = self.handle.take() {
      if let Err(error) = handle.await {
        tracing::error!(?error, "display handler task failed");
      }
    }
  }
}
