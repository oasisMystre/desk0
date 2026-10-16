use std::sync::Arc;

use audio::audio::from_audio_frame_to_pcm16;
use ipc::IpcSharedBufferField;
use ipc_pdu::{
  Connector,
  outbound::{IpcOutboundEvent, IpcPackedAudioEvent, IpcPackedAudioWave},
};
use pinray::AudioFrame;
use tokio::{sync::mpsc::UnboundedReceiver, task::JoinHandle};
use tokio_util::sync::CancellationToken;

pub struct IpcAudioHandler {
  ipc: Arc<Connector>,

  handle: Option<JoinHandle<()>>,
  cancellation_token: CancellationToken,
}

impl IpcAudioHandler {
  pub fn new(ipc: Arc<Connector>, cancellation_token: CancellationToken) -> Self {
    Self {
      ipc,
      handle: None,
      cancellation_token,
    }
  }
  pub fn capture(&mut self, mut audio_frame_receiver: UnboundedReceiver<AudioFrame>) {
    let ipc = self.ipc.clone();
    let cancellation_token = self.cancellation_token.clone();

    let handle = tokio::spawn(async move {
      'capture: loop {
        let frame = tokio::select! {
            _ = cancellation_token.cancelled() => break 'capture,
            result = audio_frame_receiver.recv() => match result {
                Some(frame) => frame,
                None => break 'capture,
            },
        };

        let (pcm16, timestamp_ms) = from_audio_frame_to_pcm16(&frame);
        if pcm16.is_empty() {
          continue;
        }

        if let Err(error) = ipc
          .send_with_shared_buffers(
            IpcOutboundEvent::Audio(IpcPackedAudioEvent::Wave(IpcPackedAudioWave {
              timestamp_ms,
              pcm16: IpcSharedBufferField(0),
            })),
            &[pcm16],
          )
          .await
        {
          tracing::error!(?error, "sending audio frame failed");
        }
      }
    });

    self.handle = Some(handle);
  }

  pub async fn stop(&mut self) {
    self.cancellation_token.cancel();

    if let Some(handle) = self.handle.take() {
      if let Err(error) = handle.await {
        tracing::error!(?error, "audio handler task failed");
      }
    }
  }
}
