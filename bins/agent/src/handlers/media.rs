use std::{ops::Deref, sync::Arc};

use display_info::DisplayInfo;
use ironrdp::pdu::gcc::{Monitor, MonitorFlags};
use pinray::{
  AudioDeviceSource, AudioFrame, CaptureEvent, CaptureSession, CaptureSource, CursorMode,
  PixelFormat, SourceId, VideoFrame,
};
use tokio::{
  sync::{
    Mutex,
    mpsc::{Receiver, Sender, UnboundedSender},
  },
  task::{JoinHandle, JoinSet},
};
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub struct MonitorSource {
  pub width: u16,
  pub height: u16,
  pub inner: Monitor,
  pub source: SourceId,
  pub primary: bool,
}

pub enum AudioSource {
  SystemMix,
  MicroPhone(AudioDeviceSource),
}

pub struct Source {
  pub audio: Option<AudioSource>,
  pub monitors: Vec<MonitorSource>,
}

pub struct WrapVideoFrame {
  pub monitor: Monitor,
  pub inner: VideoFrame,
  pub first_frame: bool,
}

#[derive(Clone)]
pub struct MediaHandler {
  frame_rate: Option<u32>,
  cursor_mode: CursorMode,
  pixel_format: PixelFormat,
  video_sender: Arc<UnboundedSender<WrapVideoFrame>>,
  audio_sender: Arc<Option<UnboundedSender<AudioFrame>>>,

  monitors: Arc<std::sync::Mutex<Vec<MonitorSource>>>,
  audios: Arc<std::sync::Mutex<Vec<AudioDeviceSource>>>,

  current_session_cancellation_token: Arc<Mutex<Option<CancellationToken>>>,
}

impl MediaHandler {
  pub fn new(
    frame_rate: Option<u32>,
    cursor_mode: CursorMode,
    pixel_format: PixelFormat,
    video_sender: UnboundedSender<WrapVideoFrame>,
    audio_sender: Option<UnboundedSender<AudioFrame>>,
  ) -> Self {
    Self {
      frame_rate,
      cursor_mode,
      pixel_format,
      video_sender: Arc::new(video_sender),
      audio_sender: Arc::new(audio_sender),
      audios: Arc::new(std::sync::Mutex::new(vec![])),
      monitors: Arc::new(std::sync::Mutex::new(vec![])),
      current_session_cancellation_token: Arc::new(Mutex::new(None)),
    }
  }

  pub async fn capture(
    &self,
    mut source_receiver: Receiver<Source>,
    cancellation_token: CancellationToken,
  ) -> anyhow::Result<()> {
    let mut tasks = JoinSet::new();

    let result: anyhow::Result<()> = 'source: loop {
      tokio::select! {
         _ = cancellation_token.cancelled() => {
             tracing::info!("omo shutting down");
             tasks.abort_all();
             while tasks.join_next().await.is_some() {}
             break 'source Ok(());
         },
         source = source_receiver.recv() => {
             let Some(source)  = source  else { break 'source Ok(());};
             let session_cancellation_token = CancellationToken::new();
             if let Some(previous) = self
               .current_session_cancellation_token
               .lock()
               .await
               .replace(session_cancellation_token.clone())
             {
               previous.cancel();
             };

             for monitor in source.monitors {
               let video_sender = self.video_sender.clone();
               let audio_sender = self.audio_sender.clone();

               let session_cancellation_token = session_cancellation_token.clone();
               let mut builder = CaptureSession::builder()
                 .video_target(pinray::VideoCaptureTarget::Display(monitor.source.clone()));

               if monitor.primary {
                 if let Some(audio) = &source.audio {
                   match audio {
                     AudioSource::SystemMix => builder = builder.audio(pinray::AudioCapture::SystemMix),
                     AudioSource::MicroPhone(source) => {
                       builder = builder.audio(pinray::AudioCapture::Microphone(source.id.clone()))
                     }
                   }
                 }
               }

               let mut session = builder
                 .pixel_format(self.pixel_format)
                 .cursor_mode(self.cursor_mode)
                 .frame_rate(self.frame_rate)
                 .build()?;

               session.start()?;
               tasks.spawn(
                 async move {
                   if let Err(error) = Self::run(
                     &mut session,
                     session_cancellation_token,
                     monitor,
                     video_sender,
                     audio_sender,
                   )
                   .await
                   {
                     tracing::error!(?error, "monitor capture session stopped");
                   }
                 }
               );
             }
         }
      }
    };

    result
  }

  async fn run(
    session: &mut CaptureSession,
    cancellation_token: CancellationToken,
    monitor: MonitorSource,
    video_sender: Arc<UnboundedSender<WrapVideoFrame>>,
    audio_sender: Arc<Option<UnboundedSender<AudioFrame>>>,
  ) -> anyhow::Result<()> {
    let mut first_frame = true;

    let result: anyhow::Result<()> = 'capture: loop {
      if cancellation_token.is_cancelled() {
        break 'capture Ok(());
      }

      match session.next_event(None) {
        Ok(CaptureEvent::Video(frame)) => {
          video_sender.send(WrapVideoFrame {
            first_frame,
            inner: frame,
            monitor: monitor.inner.clone(),
          })?;

          first_frame = false;
        }
        Ok(CaptureEvent::Audio(frame)) => {
          if let Some(audio_sender) = audio_sender.deref() {
            audio_sender.send(frame)?;
          }
        }
        Ok(CaptureEvent::Gap(gap)) => {
          tracing::debug!(reason = ? gap.reason, "capture gap; forcing a full-frame refresh");
          first_frame = true;
        }
        Ok(CaptureEvent::End) => break 'capture Ok(()),
        Err(error) => {
          break 'capture Err(error.into());
        }
      }
    };

    if let Err(error) = session.stop() {
      tracing::warn!(?error, monitor=?monitor.source, "failed to stop screen capture cleanly");
    }

    result
  }

  pub async fn stop(&self) {
    self
      .current_session_cancellation_token
      .lock()
      .await
      .as_ref()
      .map(|token| token.cancel());
  }

  pub fn monitors(&self) -> Vec<MonitorSource> {
    self.monitors.lock().unwrap().clone()
  }

  pub fn monitor_count(&self) -> usize {
    self.monitors().len()
  }

  pub fn discover(&self) -> anyhow::Result<()> {
    let displays = DisplayInfo::all()?;
    let sources = pinray::enumerate_sources()?;

    let mut audios = vec![];
    let mut monitors = Vec::with_capacity(displays.len());

    for source in sources {
      match source {
        pinray::CaptureSource::Display(source) => {
          let display = displays.iter().find(|display| display.name == source.name);

          let Some(display) = display else {
            tracing::warn!(
                source = ?source,
                "could not match Pinray display source to display topology"
            );
            continue;
          };

          monitors.push(MonitorSource {
            source: source.id,
            primary: display.is_primary,
            width: u16::try_from(display.width)?,
            height: u16::try_from(display.height)?,
            inner: ironrdp::pdu::gcc::Monitor {
              top: display.y,
              left: display.x,
              flags: MonitorFlags::empty(),
              right: display.x + display.width as i32,
              bottom: display.y + display.height as i32,
            },
          });
        }
        pinray::CaptureSource::SystemAudio(source) | pinray::CaptureSource::Microphone(source) => {
          audios.push(source);
        }
        pinray::CaptureSource::Window(_) => {}
      }
    }

    *self.audios.lock().unwrap() = audios;
    *self.monitors.lock().unwrap() = monitors;

    Ok(())
  }
}
