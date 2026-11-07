use std::{collections::VecDeque, sync::Arc};

use copypaste::ipc::clipboard_watcher::IpcClipboardWatcher;
use enigo::Settings;
use ipc::{Ipc, IpcConfig, IpcSharedBuffer, IpcSharedMemory};
use ipc_pdu::{
  inbound::{IpcAudioEvent, IpcClipboardEvent, IpcDisplayEvent, IpcInboundEvent},
  outbound::{
    IpcAudioSource, IpcDesktopSize, IpcDisplayInfo, IpcOutboundEvent, IpcPackedAudioEvent,
    IpcPackedDisplayEvent,
  },
};
use tokio::{
  sync::{
    Mutex,
    mpsc::{channel, unbounded_channel},
  },
  task::JoinSet,
};

use tokio_util::sync::CancellationToken;

use crate::{
  config::AgentConfig,
  ipc::connector::{audio::IpcAudioHandler, display::IpcDisplayHandler, input::IpcInputHandler},
  media::{AudioSource, MediaHandler, MonitorSource, Source},
};

#[derive(Clone)]
pub struct MediaSource {
  audio: Option<AudioSource>,
  monitor: Option<MonitorSource>,
}

pub struct IpcInboundEventHandler {
  cancellation_token: CancellationToken,
  media_source: Arc<Mutex<MediaSource>>,
}

impl MediaSource {
  pub fn set_audio(&mut self, audio: AudioSource) {
    self.audio = Some(audio);
  }

  pub fn set_monitor(&mut self, monitor: MonitorSource) {
    self.monitor = Some(monitor);
  }
}

impl IpcInboundEventHandler {
  pub fn new(cancellation_token: CancellationToken) -> Self {
    Self {
      cancellation_token,
      media_source: Arc::new(Mutex::new(MediaSource {
        audio: None,
        monitor: None,
      })),
    }
  }

  pub async fn start(&mut self) -> anyhow::Result<()> {
    let mut tasks = JoinSet::new();
    let cancellation_token = self.cancellation_token.clone();

    let (sender, mut receiver) = channel(100);
    let (source_sender, source_receiver) = channel(1);

    let (clipboard_sender, clipboard_receiver) = unbounded_channel();
    let (input_event_sender, input_event_receiver) = unbounded_channel();
    let (audio_frame_sender, audio_frame_receiver) = unbounded_channel();
    let (video_frame_sender, video_frame_receiver) = unbounded_channel();

    let media = Arc::new(MediaHandler::new(
      Some(60),
      pinray::CursorMode::Hidden,
      pinray::PixelFormat::Bgra8888,
      video_frame_sender,
      Some(audio_frame_sender),
    ));

    media.discover()?;

    let shared_memory = Arc::new(IpcSharedMemory::<IpcInboundEvent, IpcOutboundEvent>::new(
      &AgentConfig::get_ipc_share_memory_path(),
      false,
      IpcConfig::default(),
    )?);
    let mut ipc = Ipc::new(shared_memory.clone());
    ipc
      .as_connector(AgentConfig::get_ipc_path()?, sender)
      .await?;

    let mut input_handler = IpcInputHandler::new(Settings::default(), cancellation_token.clone());
    let mut clipboard_handler = IpcClipboardWatcher::new(shared_memory.clone(), clipboard_receiver);
    let mut audio_handler =
      IpcAudioHandler::new(shared_memory.clone(), cancellation_token.child_token());
    let mut display_handler =
      IpcDisplayHandler::new(shared_memory.clone(), cancellation_token.child_token());

    input_handler.start(input_event_receiver)?;
    audio_handler.capture(audio_frame_receiver);
    display_handler.capture(video_frame_receiver);

    let monitors = media.monitors();
    let width = monitors
      .iter()
      .map(|monitor| monitor.x as u32 + monitor.width as u32)
      .max()
      .unwrap_or(1282);

    let height = monitors
      .iter()
      .map(|monitor| monitor.y as u32 + monitor.height as u32)
      .max()
      .unwrap_or(720);

    shared_memory
      .send(IpcOutboundEvent::Display(
        IpcPackedDisplayEvent::DisplayInfo(IpcDisplayInfo {
          monitors: monitors.len() as u32,
          size: IpcDesktopSize {
            width: width.clamp(1, u16::MAX as u32) as u16,
            height: height.clamp(1, u16::MAX as u32) as u16,
          },
        }),
      ))
      .await?;

    shared_memory
      .send(IpcOutboundEvent::Audio(IpcPackedAudioEvent::AudioSource(
        media
          .audios()
          .into_iter()
          .map(|audio| IpcAudioSource {
            is_default: audio.is_default,
            formats: audio
              .formats
              .into_iter()
              .map(|format| format.into())
              .collect(),
          })
          .collect(),
      )))
      .await?;

    let (audio, monitor) = {
      let source = self.media_source.lock().await;
      (source.audio.clone(), source.monitor.clone())
    };
    source_sender.send(Source { audio, monitor }).await?;

    tasks.spawn({
      let audios = media.audios();
      let monitors = media.monitors();
      let media_source = Arc::clone(&self.media_source);
      let cancellation_token = cancellation_token.clone();

      async move {
        loop {
          let message = tokio::select! {
              _ = cancellation_token.cancelled() => break,
              message = receiver.recv() => message,
          };

          let Some(message) = message else {
            break;
          };

          if cancellation_token.is_cancelled() {
            break;
          }

          tracing::info!(data=?message.data, "data");

          match message.data {
            IpcInboundEvent::Input(event) => {
              if let Err(error) = input_event_sender.send(event) {
                tracing::error!(?error, "input channel closed");
              }
            }
            IpcInboundEvent::Audio(event) => match event {
              IpcAudioEvent::Start(index) => {
                if index < audios.len() {
                  let audio = audios.get(index);
                  let Some(audio) = audio else { continue };
                  let monitor = {
                    let source = media_source.lock().await;
                    source.monitor.clone()
                  };

                  if let Err(error) = source_sender
                    .send(Source {
                      monitor,
                      audio: Some(audio.clone()),
                    })
                    .await
                  {
                    tracing::error!(?error, "setting monitor failed");
                  }

                  media_source.lock().await.set_audio(audio.clone());
                }
              }
              IpcAudioEvent::Stop => {
                audio_handler.stop().await;
              }
            },
            IpcInboundEvent::Display(event) => match event {
              IpcDisplayEvent::Start(index) => {
                if index < monitors.len() {
                  let monitor = monitors.get(index);
                  tracing::info!(?monitor, "upadating monitor monitor index");

                  let Some(monitor) = monitor else {
                    continue;
                  };
                  let audio = {
                    let source = media_source.lock().await;
                    source.audio.clone()
                  };

                  tracing::info!(?index, "upadating monitor monitor index");
                  if let Err(error) = source_sender
                    .send(Source {
                      audio,
                      monitor: Some(monitor.clone()),
                    })
                    .await
                  {
                    tracing::error!(?error, "setting monitor failed");
                  }

                  tracing::info!(?index, "monitor index");

                  {
                    let mut source = media_source.lock().await;
                    source.set_monitor(monitor.clone());
                  }

                  tracing::info!(?index, "monitor index");
                }
              }
              IpcDisplayEvent::Stop => {
                display_handler.stop().await;
              }
            },
            IpcInboundEvent::Clipboard(event) => match event {
              IpcClipboardEvent::Start => {
                clipboard_handler.start();
              }
              IpcClipboardEvent::Stop => {
                clipboard_handler.stop().await;
              }
              IpcClipboardEvent::Data(data) => {
                if let Some(buffers) = message.buffers {
                  let mut buffers: VecDeque<IpcSharedBuffer> = buffers.into();
                  match data.unpack(&mut buffers) {
                    Ok(event) => {
                      if let Err(error) = clipboard_sender.send(event) {
                        tracing::error!(?error, "clipboard channel closed");
                      }
                    }
                    Err(error) => {
                      if let Err(error) = shared_memory
                        .ack_shared_buffers(
                          buffers.into_iter().map(|buffer| buffer.into()).collect(),
                        )
                        .await
                      {
                        tracing::error!(?error, "acking shared buffer for clipboard data failed");
                      };
                      tracing::error!(?error, "unpacking clipboard event failed")
                    }
                  }
                }
              }
            },
          }
        }
      }
    });

    tasks.spawn({
      let cancellation_token = cancellation_token.clone();

      async move {
        match media.capture(source_receiver, cancellation_token).await {
          Ok(mut tasks) => {
            while let Some(result) = tasks.join_next().await {
              if let Err(error) = result {
                if !error.is_cancelled() {
                  tracing::error!(?error, "media task failed during shutdown");
                }
              }
            }
          }
          Err(error) => {
            tracing::error!(?error, "media capture task error");
          }
        }
      }
    });

    loop {
      tokio::select! {
        _ = cancellation_token.cancelled() => break,
        result = tasks.join_next() => {
            match result {
                Some(Ok(())) => {
                    tracing::warn!("a supervised IPC/media task exited");
                },
                Some(Err(error)) if error.is_cancelled() =>{
                    tracing::debug!("a supervised task was cancelled");
                },
                Some(Err(error))  => {
                    tracing::error!(?error, "a supervised task failed");
                },
                None => {
                    tracing::warn!("all supervised tasks exited");
                },
            }

            break;
        }
      }
    }

    cancellation_token.cancel();

    if let Err(error) = ipc.stop().await {
      tracing::error!(?error, "stopping ipc task failed");
    }

    tasks.abort_all();

    while let Some(result) = tasks.join_next().await {
      if let Err(error) = result {
        if !error.is_cancelled() {
          tracing::error!(?error, "task failed during shutdown");
        }
      }
    }

    Ok(())
  }
}
