use std::{fmt::Debug, sync::Arc};

use audio::audio::cpal_supported_stream_config_range_to_audio_format;
use cpal::traits::{DeviceTrait, HostTrait};
use display_info::DisplayInfo;
use ironrdp::{
  pdu::gcc::{Monitor, MonitorFlags},
  rdpsnd::pdu::AudioFormat,
};
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

#[derive(Clone, Debug)]
pub struct MonitorSource {
  pub x: u16,
  pub y: u16,
  pub width: u16,
  pub height: u16,
  #[allow(dead_code)]
  pub inner: Monitor,
  pub source: SourceId,
  pub is_primary: bool,
}

#[derive(Debug, Clone)]
pub enum AudioDeviceSourceId {
  SystemMix,
  MicroPhone(AudioDeviceSource),
}

#[derive(Debug, Clone)]
pub struct AudioSource {
  pub is_default: bool,
  pub formats: Vec<AudioFormat>,
  pub source: AudioDeviceSourceId,
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

  audios: Arc<std::sync::Mutex<Vec<AudioSource>>>,
  monitors: Arc<std::sync::Mutex<Vec<MonitorSource>>>,

  current_session_cancellation_token: Arc<Mutex<Option<CancellationToken>>>,
}

impl Debug for MediaHandler {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("MediaHandler")
      .field("frame_rate", &self.frame_rate)
      .field("cursor_mode", &self.cursor_mode)
      .field("pixel_format", &self.pixel_format)
      .field("video_sender", &self.video_sender)
      .field("audio_sender", &self.audio_sender)
      .field("audios", &self.audios)
      .field(
        "current_session_cancellation_token",
        &self.current_session_cancellation_token,
      )
      .finish()
  }
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

          if monitor.is_primary {
            if let Some(audio) = &source.audio {
              match &audio.source {
                AudioDeviceSourceId::SystemMix => {
                  builder = builder.audio(pinray::AudioCapture::SystemMix)
                }
                AudioDeviceSourceId::MicroPhone(source) => {
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

  pub fn audios(&self) -> Vec<AudioSource> {
    self.audios.lock().unwrap().clone()
  }

  pub fn audio_formats(&self) -> Vec<AudioFormat> {
    self
      .audios
      .lock()
      .unwrap()
      .iter()
      .flat_map(|audio| audio.formats.iter().cloned())
      .collect()
  }

  pub fn discover(&self) -> anyhow::Result<()> {
    let displays = DisplayInfo::all()?;
    let sources = pinray::enumerate_sources()?;

    let mut raw = Vec::with_capacity(displays.len());

    let mut audios = vec![];
    let mut audio_formats = vec![];

    let host = cpal::default_host();
    let default_audio_input = host.default_input_device();

    if let Some(device) = default_audio_input {
      if let Ok(supported_configs) = device.supported_input_configs() {
        for config in supported_configs {
          if let Some(format) = cpal_supported_stream_config_range_to_audio_format(config) {
            audio_formats.push(format);
          }
        }
      }
    }

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
        pinray::CaptureSource::SystemAudio(source) => {
          audios.push(AudioSource {
            is_default: source.is_default,
            formats: audio_formats.clone(),
            source: AudioDeviceSourceId::SystemMix,
          });
        }
        pinray::CaptureSource::Microphone(source) => {
          audios.push(AudioSource {
            is_default: source.is_default,
            formats: audio_formats.clone(),
            source: AudioDeviceSourceId::MicroPhone(source),
          });
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
        let y = (raw.y - origin_y) as u16;

        MonitorSource {
          x,
          y,
          width: raw.width,
          height: raw.height,
          is_primary: raw.primary,
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

    if audios.is_empty() {
      audios.push(AudioSource {
        is_default: true,
        formats: audio_formats,
        source: AudioDeviceSourceId::SystemMix,
      });
    }

    *self.audios.lock().unwrap() = audios;
    *self.monitors.lock().unwrap() = monitors;

    Ok(())
  }
}
