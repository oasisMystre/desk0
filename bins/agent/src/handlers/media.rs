use std::{sync::Arc, time::Duration};

use display_info::DisplayInfo;
use ironrdp::pdu::gcc::{Monitor, MonitorFlags};
use pinray::{
  AudioDeviceSource, AudioFrame, CaptureEvent, CaptureSession, CursorMode, PixelFormat, SourceId,
  VideoFrame,
};
use tokio::{
  sync::{
    Mutex,
    mpsc::{Receiver, UnboundedSender},
  },
  task::JoinSet,
};
use tokio_util::sync::CancellationToken;

struct RawDisplay {
  x: i32,
  y: i32,
  width: u16,
  height: u16,
  primary: bool,
  source: SourceId,
}

#[derive(Clone)]
pub struct MonitorSource {
  pub x: u16,
  pub y: u16,
  pub width: u16,
  pub height: u16,
  pub primary: bool,
  pub inner: Monitor,
  pub source: SourceId,
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
  pub inner: VideoFrame,
  pub first_frame: bool,
  pub monitor: MonitorSource,
}

#[derive(Clone)]
pub struct MediaHandler {
  frame_rate: Option<u32>,
  cursor_mode: CursorMode,
  pixel_format: PixelFormat,
  video_sender: UnboundedSender<WrapVideoFrame>,
  audio_sender: Option<UnboundedSender<AudioFrame>>,

  monitors: Arc<std::sync::Mutex<Vec<MonitorSource>>>,
  audios: Arc<std::sync::Mutex<Vec<AudioDeviceSource>>>,

  current_session_cancellation_token: Arc<Mutex<Option<CancellationToken>>>,
}

const CAPTURE_POLL_INTERVAL: Duration = Duration::from_millis(200);

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
      video_sender,
      audio_sender,
      audios: Arc::new(std::sync::Mutex::new(vec![])),
      monitors: Arc::new(std::sync::Mutex::new(vec![])),
      current_session_cancellation_token: Arc::new(Mutex::new(None)),
    }
  }

  pub async fn capture(
    &self,
    mut source_receiver: Receiver<Source>,
    cancellation_token: CancellationToken,
  ) -> anyhow::Result<JoinSet<()>> {
    let mut tasks = JoinSet::new();

    'source: loop {
      let source = tokio::select! {
          _ = cancellation_token.cancelled() => break 'source,
          source = source_receiver.recv() => {
              match source {
                  Some(source) => source,
                  None => {
                      if let Some(session_token) =
                          self.current_session_cancellation_token.lock().await.take()
                      {
                          session_token.cancel();
                      }
                      break 'source;
                  }
              }
          }
      };

      let session_cancellation_token = cancellation_token.child_token();
      if let Some(previous) = self
        .current_session_cancellation_token
        .lock()
        .await
        .replace(session_cancellation_token.clone())
      {
        previous.cancel();

        while let Some(result) = tasks.join_next().await {
          if let Err(error) = result {
            tracing::error!(?error, "previous capture task failed");
          }
        }
      };

      for monitor in source.monitors {
        let video_sender = self.video_sender.clone();
        let audio_sender = self.audio_sender.clone();
        let session_cancellation_token = session_cancellation_token.clone();

        let setup = (|| -> anyhow::Result<CaptureSession> {
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

          Ok(session)
        })();

        let mut session = match setup {
          Ok(session) => session,
          Err(error) => {
            session_cancellation_token.cancel();

            while let Some(result) = tasks.join_next().await {
              if let Err(join_error) = result {
                tracing::error!(?join_error, "capture task failed during cleanup");
              }
            }

            return Err(error);
          }
        };

        tasks.spawn(async move {
          let result = tokio::task::spawn_blocking(move || {
            Self::run(
              &mut session,
              session_cancellation_token,
              monitor,
              video_sender,
              audio_sender,
            )
          })
          .await;

          match result {
            Ok(Ok(())) => {}
            Ok(Err(error)) => tracing::error!(?error, "monitor capture session stopped"),
            Err(error) => tracing::error!(?error, "monitor capture thread panicked"),
          }
        });
      }
    }

    Ok(tasks)
  }

  fn run(
    session: &mut CaptureSession,
    cancellation_token: CancellationToken,
    monitor: MonitorSource,
    video_sender: UnboundedSender<WrapVideoFrame>,
    audio_sender: Option<UnboundedSender<AudioFrame>>,
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
            monitor: monitor.clone(),
          })?;

          first_frame = false;
        }
        Ok(CaptureEvent::Audio(frame)) => {
          if let Some(audio_sender) = &audio_sender {
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
    let mut raw = Vec::with_capacity(displays.len());

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

          let (Ok(width), Ok(height)) =
            (u16::try_from(display.width), u16::try_from(display.height))
          else {
            let name = &display.name;
            tracing::warn!(display = %name, "monitor dimensions out of RDP range; skipping");
            continue;
          };

          raw.push(RawDisplay {
            width,
            height,
            x: display.x,
            y: display.y,
            primary: display.is_primary,
            source: source.id.clone(),
          });
        }
        pinray::CaptureSource::SystemAudio(source) | pinray::CaptureSource::Microphone(source) => {
          audios.push(source);
        }
        pinray::CaptureSource::Window(_) => {}
      }
    }

    let origin_x = raw.iter().map(|raw| raw.x).min().unwrap_or(0);
    let origin_y = raw.iter().map(|raw| raw.y).min().unwrap_or(0);

    let monitors = raw
      .into_iter()
      .map(|raw| {
        let x = (raw.x - origin_x) as u16;
        let y = (raw.x - origin_y) as u16;

        MonitorSource {
          x,
          y,
          width: raw.width,
          height: raw.height,
          primary: raw.primary,
          source: raw.source.clone(),
          inner: Monitor {
            left: x as i32,
            top: y as i32,
            right: x as i32 + raw.width as i32,
            bottom: y as i32 + raw.height as i32,
            flags: if raw.primary {
              MonitorFlags::PRIMARY
            } else {
              MonitorFlags::empty()
            },
          },
        }
      })
      .collect();

    *self.audios.lock().unwrap() = audios;
    *self.monitors.lock().unwrap() = monitors;

    Ok(())
  }
}
