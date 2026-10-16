use std::sync::{Arc, Condvar, Mutex, OnceLock};

use ipc_pdu::{
  Listener,
  inbound::{IpcAudioEvent, IpcInboundEvent},
  outbound::{IpcAudioSource, IpcUnpackedAudioWave},
};
use ironrdp::{
  rdpsnd::{
    pdu::AudioFormat,
    server::{NegotiatedFormat, RdpsndError},
  },
  server::{
    RdpsndServerHandler, RdpsndServerMessage, ServerEvent, ServerEventSender, SoundServerFactory,
  },
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;

#[derive(thiserror::Error, Debug)]
pub enum AudioHandlerError {
  #[error("audio failed to send start event {0}")]
  StartError(anyhow::Error),
  #[error("audio backend already initialized")]
  AlreadyInitialized,
  #[error("audio backend is not initialized")]
  NotInitialized,
}

#[derive(Debug)]
pub struct AudioState {
  lock: Mutex<()>,
  ready: Condvar,
  formats: OnceLock<Vec<AudioFormat>>,
  sources: OnceLock<Vec<IpcAudioSource>>,
}

#[derive(Clone, Debug)]
pub struct AudioHandler {
  ipc: Arc<Listener>,
  sources: Arc<AudioState>,
  sender: Option<UnboundedSender<ServerEvent>>,
  inbound: Arc<Mutex<Option<UnboundedReceiver<IpcUnpackedAudioWave>>>>,
}

#[derive(Debug)]
pub struct AudioHandlerStaticChannelFactory {
  handler: AudioHandler,

  handle: Option<std::thread::JoinHandle<()>>,
  cancellation_token: Option<CancellationToken>,
}

impl AudioState {
  pub fn new() -> Self {
    Self {
      lock: Mutex::new(()),
      ready: Condvar::new(),
      formats: OnceLock::new(),
      sources: OnceLock::new(),
    }
  }

  pub fn set_sources(&self, sources: Vec<IpcAudioSource>) {
    let formats = sources
      .iter()
      .flat_map(|source| source.formats.iter().cloned().map(Into::into))
      .collect();

    self
      .sources
      .set(sources)
      .expect("audio sources already initialized");
    self
      .formats
      .set(formats)
      .expect("audio formats already initialized");
    self.ready.notify_all();
  }

  fn wait(&self) {
    let mut guard = self.lock.lock().unwrap();

    while self.formats.get().is_none() {
      guard = self.ready.wait(guard).unwrap();
    }
  }

  pub fn formats(&self) -> &[AudioFormat] {
    self.wait();
    self.formats.get().unwrap()
  }

  pub fn sources(&self) -> &[IpcAudioSource] {
    self.wait();
    self.sources.get().unwrap()
  }
}

impl AudioHandler {
  pub fn new(ipc: Arc<Listener>, inbound: UnboundedReceiver<IpcUnpackedAudioWave>) -> Self {
    Self {
      ipc,
      sender: None,
      sources: Arc::new(AudioState::new()),
      inbound: Arc::new(Mutex::new(Some(inbound))),
    }
  }

  pub fn set_sources(&self, sources: Vec<IpcAudioSource>) {
    self.sources.set_sources(sources);
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
    self.handler.sources.formats()
  }

  fn choose_format<'a>(&mut self, common: &'a [NegotiatedFormat]) -> Option<&'a NegotiatedFormat> {
    let audio = self
      .handler
      .sources
      .sources()
      .iter()
      .find(|audio| audio.is_default)?;

    common.iter().find(|negotiated| {
      audio.formats.iter().any(|format| {
        let format: AudioFormat = format.clone().into();
        negotiated.format().matches_for_negotiation(&format)
      })
    })
  }

  fn start(&mut self, _f: &NegotiatedFormat) -> Result<(), Box<dyn RdpsndError>> {
    self
      .handler
      .ipc
      .blocking_send(IpcInboundEvent::Audio(IpcAudioEvent::Start(0)))
      .map_err(|error| Box::new(AudioHandlerError::StartError(error)) as Box<dyn RdpsndError>)?;

    let mut receiver = self
      .handler
      .inbound
      .lock()
      .unwrap()
      .take()
      .ok_or_else(|| Box::new(AudioHandlerError::AlreadyInitialized) as Box<dyn RdpsndError>)?;

    let handler = self.handler.clone();
    let cancellation_token = CancellationToken::new();
    self.cancellation_token = Some(cancellation_token.clone());

    self.handle = Some(std::thread::spawn(move || {
      loop {
        if cancellation_token.is_cancelled() {
          break;
        }

        let frame = match receiver.blocking_recv() {
          Some(frame) => frame,
          None => break,
        };

        match handler.ipc.blocking_copy_from_shared_buffer(frame.pcm16) {
          Ok(pcm16) => {
            if handler
              .send(RdpsndServerMessage::Wave(pcm16, frame.timestamp_ms))
              .is_err()
            {
              break;
            }
          }
          Err(error) => {
            tracing::error!(?error, "failed to copy audio frame from shared buffer");
          }
        };
      }
    }));

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
