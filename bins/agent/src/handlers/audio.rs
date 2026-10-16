use std::sync::{Arc, Mutex};

use audio::audio::from_audio_frame_to_pcm16;
use ironrdp::{
  rdpsnd::{
    pdu::AudioFormat,
    server::{NegotiatedFormat, RdpsndError},
  },
  server::{
    RdpsndServerHandler, RdpsndServerMessage, ServerEvent, ServerEventSender, SoundServerFactory,
  },
};
use pinray::AudioFrame;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;

use crate::media::{AudioDeviceSourceId, MediaHandler};

#[derive(thiserror::Error, Debug)]
pub enum AudioHandlerError {
  #[error("audio backend already initialized")]
  AlreadyInitialized,
  #[error("audio backend is not initialized")]
  NotInitialized,
}

#[derive(Clone, Debug)]
pub struct AudioHandler {
  media: Arc<MediaHandler>,
  sender: Option<UnboundedSender<ServerEvent>>,
  audio_frames_receiver: Arc<Mutex<Option<UnboundedReceiver<AudioFrame>>>>,
}

#[derive(Debug)]
pub struct AudioHandlerStaticChannelFactory {
  handler: AudioHandler,
  audio_formats: Vec<AudioFormat>,
  handle: Option<std::thread::JoinHandle<()>>,
  cancellation_token: Option<CancellationToken>,
}

impl AudioHandler {
  pub fn new(
    media: Arc<MediaHandler>,
    audio_frames_receiver: UnboundedReceiver<AudioFrame>,
  ) -> Self {
    Self {
      media,
      sender: None,
      audio_frames_receiver: Arc::new(Mutex::new(Some(audio_frames_receiver))),
    }
  }

  pub fn send(&self, message: RdpsndServerMessage) -> anyhow::Result<()> {
    match &self.sender {
      Some(sender) => sender.send(ServerEvent::Rdpsnd(message))?,
      None => return Err(AudioHandlerError::NotInitialized.into()),
    };

    Ok(())
  }
}

impl SoundServerFactory for AudioHandler {
  fn build_backend(&self) -> Box<dyn RdpsndServerHandler> {
    Box::new(AudioHandlerStaticChannelFactory {
      handle: None,
      handler: self.clone(),
      cancellation_token: None,
      audio_formats: self.media.audio_formats(),
    })
  }
}

impl ServerEventSender for AudioHandler {
  fn set_sender(&mut self, sender: UnboundedSender<ServerEvent>) {
    self.sender = Some(sender);
  }
}

impl RdpsndServerHandler for AudioHandlerStaticChannelFactory {
  fn get_formats(&self) -> &[AudioFormat] {
    &self.audio_formats
  }

  fn choose_format<'a>(&mut self, common: &'a [NegotiatedFormat]) -> Option<&'a NegotiatedFormat> {
    let audios = self.handler.media.audios();

    let audio = audios
      .iter()
      .find(|audio| audio.is_default && matches!(audio.source, AudioDeviceSourceId::SystemMix))?;

    common.iter().find(|negotiated| {
      audio
        .formats
        .iter()
        .any(|format| negotiated.format().matches_for_negotiation(format))
    })
  }

  fn start(&mut self, _f: &NegotiatedFormat) -> Result<(), Box<dyn RdpsndError>> {
    let mut receiver = self
      .handler
      .audio_frames_receiver
      .lock()
      .unwrap()
      .take()
      .ok_or_else(|| Box::new(AudioHandlerError::AlreadyInitialized) as Box<dyn RdpsndError>)?;

    let handler = self.handler.clone();
    let cancellation_token = CancellationToken::new();
    self.cancellation_token = Some(cancellation_token.clone());

    let thread = std::thread::spawn(move || {
      loop {
        if cancellation_token.is_cancelled() {
          break;
        }

        let frame = match receiver.blocking_recv() {
          Some(frame) => frame,
          None => break,
        };

        let (pcm16, timestamp_ms) = from_audio_frame_to_pcm16(&frame);
        if pcm16.is_empty() {
          continue;
        }

        if handler
          .send(RdpsndServerMessage::Wave(pcm16, timestamp_ms))
          .is_err()
        {
          break;
        }
      }
    });

    self.handle = Some(thread);

    Ok(())
  }

  fn stop(&mut self) {
    if let Some(cancellation_token) = self.cancellation_token.take() {
      cancellation_token.cancel();
    }

    if let Some(handle) = self.handle.take() {
      let _ = handle.join();
    }
  }
}

impl Drop for AudioHandlerStaticChannelFactory {
  fn drop(&mut self) {
    self.stop();
  }
}
