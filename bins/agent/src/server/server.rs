use std::{net::SocketAddr, sync::Arc};

use copypaste::CliprdrHandler;

use ironrdp::server::{RdpServer, ServerError, ServerErrorKind, TransportTls};
use tokio::{
  io::{AsyncRead, AsyncWrite},
  net::TcpListener,
  sync::mpsc::{channel, unbounded_channel},
  task::JoinSet,
};
use tokio_rustls::{
  TlsAcceptor,
  rustls::{
    self,
    pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer},
  },
};
use tokio_util::sync::CancellationToken;

use crate::{
  server::traits::Server,
  static_channels::{IoStaticChannelFactory, TerminalStaticChannelFactory},
};

#[derive(Clone)]
pub struct AgentServer {
  config: AgentServerConfig,
}

#[derive(Clone, Default)]
pub struct AgentServerConfig {
  pub tunnel: bool,
  pub addr: Option<SocketAddr>,
}

impl AgentServer {
  pub fn new(addr: SocketAddr) -> Self {
    Self {
      config: AgentServerConfig {
        addr: Some(addr),
        tunnel: false,
      },
    }
  }
}

#[async_trait::async_trait(?Send)]
impl Server for AgentServer {
  async fn run(&self, cancellation_token: CancellationToken) -> anyhow::Result<()> {
    let local = tokio::task::LocalSet::new();
    local
      .run_until(async move {
        let addr = self
          .config
          .addr
          .ok_or_else(|| anyhow::anyhow!("Server bind address is not configured"))?;
        let listener = TcpListener::bind(addr).await?;
        tracing::info!(?addr, "RDP server listening");

        loop {
          tokio::select! {
            _ = cancellation_token.cancelled() => {
              tracing::debug!("RDP server shutting down...");
              break;
            }
            result = listener.accept() => {
              let (stream, peer_addr) = result?;
              tracing::info!(?peer_addr, "Accepted connection");

              let config = self.config.clone();
              let cancellation_token = cancellation_token.clone();

              tokio::task::spawn_local(async move {
                tracing::info!(%peer_addr, "RDP connection accepted");
                if let Err(error) = Self::handle(config, stream, cancellation_token).await {
                    tracing::error!(?error, ?peer_addr, "Connection handler error");
                }
                tracing::info!(%peer_addr, "RDP connection handler stopped");
              });
            }
          }
        }

        Ok::<_, anyhow::Error>(())
      })
      .await?;

    Ok(())
  }
}

impl AgentServer {
  fn is_expected_disconnect(error: &ServerError) -> bool {
    match error.kind() {
      ServerErrorKind::Io(error) => matches!(
        error.kind(),
        std::io::ErrorKind::UnexpectedEof
          | std::io::ErrorKind::ConnectionReset
          | std::io::ErrorKind::BrokenPipe
          | std::io::ErrorKind::NotConnected
      ),
      _ => false,
    }
  }

  fn tls_acceptor() -> anyhow::Result<TlsAcceptor> {
    let identity = rcgen::generate_simple_self_signed(vec!["desk0-agent".to_owned()])?;
    let certificate = CertificateDer::from(identity.cert.der().to_vec());
    let private_key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
      identity.signing_key.serialize_der(),
    ));
    let config = rustls::ServerConfig::builder()
      .with_no_client_auth()
      .with_single_cert(vec![certificate], private_key)?;

    Ok(TlsAcceptor::from(Arc::new(config)))
  }

  #[cfg(not(feature = "ipc"))]
  pub async fn handle<S>(
    config: AgentServerConfig,
    stream: S,
    cancellation_token: CancellationToken,
  ) -> anyhow::Result<()>
  where
    S: AsyncRead + AsyncWrite + Sync + Send + Unpin + 'static,
  {
    use crate::{
      handlers::{AudioHandler, DisplayHandler, InputHandler},
      media::{MediaHandler, Source},
    };
    use enigo::Settings;
    use pinray::{CursorMode, PixelFormat};

    let (source_sender, source_receiver) = channel(1);
    let (video_frames_sender, video_frames_receiver) = unbounded_channel();
    let (audio_frames_sender, audioframes_receiver) = unbounded_channel();

    let media = Arc::new(MediaHandler::new(
      None,
      CursorMode::Hidden,
      PixelFormat::Bgra8888,
      video_frames_sender,
      Some(audio_frames_sender),
    ));

    media.discover()?;
    let audio = media.audios();
    source_sender.try_send(Source {
      monitors: media
        .monitors()
        .iter()
        .filter(|monitor| monitor.is_primary)
        .cloned()
        .collect(),
      audio: audio
        .iter()
        .find(|audio| audio.is_default)
        .or_else(|| audio.first())
        .cloned(),
    })?;

    let display_handler = DisplayHandler::new(Arc::clone(&media));
    let audio_handler = AudioHandler::new(Arc::clone(&media), audioframes_receiver);

    let input_handler = InputHandler::new(&Settings::default())?;

    let mut tasks = JoinSet::new();

    let addr = config
      .addr
      .unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 0)));

    let tls_acceptor = Self::tls_acceptor()?;
    let server = RdpServer::builder()
      .with_addr(addr)
      .with_tls(tls_acceptor)
      .with_input_handler(input_handler)
      .with_display_handler(display_handler.clone())
      .with_cliprdr_factory(Some(Box::new(CliprdrHandler::new())))
      .with_static_channel_factory(Box::new(IoStaticChannelFactory))
      .with_static_channel_factory(Box::new(TerminalStaticChannelFactory))
      .with_sound_factory(Some(Box::new(audio_handler)));

    let mut server = server.build();

    tasks.spawn_local({
      let cancellation_token = cancellation_token.clone();
      async move {
        match media.capture(source_receiver, cancellation_token).await {
          Ok(mut tasks) => {
            tasks.abort_all();
            while tasks.join_next().await.is_some() {}
          }
          Err(error) => {
            tracing::error!(?error, "media capture task error");
          }
        }
      }
    });

    tasks.spawn_local({
      let cancellation_token = cancellation_token.clone();
      async move {
        match display_handler.capture(video_frames_receiver, cancellation_token) {
          Ok(handle) => {
            if let Err(error) = handle.await {
              tracing::error!(?error, "capture task error");
            }
          }
          Err(error) => {
            tracing::error!(?error, "Display capture task error");
          }
        }
      }
    });

    tasks.spawn_local(async move {
      let future = if config.tunnel {
        server
          .run_connection_with(stream, TransportTls::AlreadyDone)
          .await
      } else {
        server.run_connection(stream).await
      };

      if let Err(error) = future {
        if Self::is_expected_disconnect(&error) {
          tracing::debug!(?error, "RDP connection closed");
        } else {
          tracing::error!(?error, "Connection dropped");
        }
      }
    });

    tokio::select! {
      _ = cancellation_token.cancelled() => {
        tasks.abort_all();
        while tasks.join_next().await.is_some() {
        }
      }
      _ = async {
        while tasks.join_next().await.is_some() {}
      } => {}
    }

    Ok(())
  }

  #[cfg(feature = "ipc")]
  pub async fn handle<S>(
    config: AgentServerConfig,
    stream: S,
    cancellation_token: CancellationToken,
  ) -> anyhow::Result<()>
  where
    S: AsyncRead + AsyncWrite + Sync + Send + Unpin + 'static,
  {
    use std::collections::VecDeque;

    use ipc::{Ipc, IpcConfig, IpcSharedBuffer, IpcSharedMemory};
    use ipc_pdu::{
      inbound::IpcInboundEvent,
      outbound::{IpcOutboundEvent, IpcUnpackedAudioEvent},
    };

    use crate::{
      config::AgentConfig,
      ipc::{AudioHandler, DisplayHandler, InputHandler},
    };

    let (sender, mut receiver) = channel(100);

    let (audio_frame_sender, audio_frame_receiver) = unbounded_channel();
    let (display_update_sender, display_update_receiver) = unbounded_channel();
    let (clipboard_event_sender, clipboard_event_receiver) = unbounded_channel();

    let shared_memory = Arc::new(IpcSharedMemory::<IpcOutboundEvent, IpcInboundEvent>::new(
      &AgentConfig::get_ipc_share_memory_path(),
      true,
      IpcConfig::default(),
    )?);
    let mut ipc = Ipc::new(shared_memory.clone());
    ipc
      .as_listener(AgentConfig::get_ipc_path()?, sender)
      .await?;

    let input_handler = InputHandler::new(shared_memory.clone())?;
    let audio_handler = AudioHandler::new(shared_memory.clone(), audio_frame_receiver);
    let display_handler = DisplayHandler::new(
      shared_memory.clone(),
      display_update_receiver,
      cancellation_token.child_token(),
    );

    display_handler.start();

    let addr = config
      .addr
      .unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 0)));

    let tls_acceptor = Self::tls_acceptor()?;
    let server = RdpServer::builder()
      .with_addr(addr)
      .with_tls(tls_acceptor)
      .with_input_handler(input_handler)
      .with_display_handler(display_handler.clone())
      .with_cliprdr_factory(Some(Box::new(CliprdrHandler::new(
        shared_memory.clone(),
        clipboard_event_receiver,
      ))))
      .with_sound_factory(Some(Box::new(audio_handler.clone())))
      .with_static_channel_factory(Box::new(IoStaticChannelFactory))
      .with_static_channel_factory(Box::new(TerminalStaticChannelFactory));

    let mut server = server.build();
    let mut tasks = JoinSet::new();

    tracing::info!("spawning ipc receiver");
    tasks.spawn_local({
      let audio_handler = audio_handler.clone();
      let cancellation_token = cancellation_token.clone();

      async move {
        loop {
          let result = tokio::select! {
              _ = cancellation_token.cancelled() => {break;}
              result = receiver.recv() => result
          };

          let Some(event) = result else {
            break;
          };

          if cancellation_token.is_cancelled() {
            break;
          }

          let Some(buffers) = event.buffers else {
            continue;
          };

          let mut buffers: VecDeque<IpcSharedBuffer> = buffers.into();

          tracing::info!(data=?event.data, "frame data");

          match event.data {
            IpcOutboundEvent::Audio(event) => {
              match event.unpack(&mut buffers) {
                Ok(event) => match event {
                  IpcUnpackedAudioEvent::Wave(wave) => {
                    if let Err(error) = audio_frame_sender.send(wave) {
                      tracing::error!(?error, "audio event channel closed");
                    }
                  }
                  IpcUnpackedAudioEvent::AudioSource(sources) => audio_handler.set_sources(sources),
                },
                Err(error) => {
                  tracing::error!(?error, "audio clipboard ipc event failed");
                }
              };
            }
            IpcOutboundEvent::Display(event) => {
              match event.unpack(&mut buffers) {
                Ok(event) => {
                  if let Err(error) = display_update_sender.send(event) {
                    tracing::error!(?error, "display event channel closed");
                  }
                }
                Err(error) => {
                  tracing::error!(?error, "unpacking display ipc event failed");
                }
              };
            }
            IpcOutboundEvent::Clipboard(event) => {
              match event.unpack(&mut buffers) {
                Ok(event) => {
                  if let Err(error) = clipboard_event_sender.send(event) {
                    tracing::error!(?error, "clipboard event channel closed");
                  }
                }
                Err(error) => {
                  tracing::error!(?error, "unpacking clipboard ipc event failed");
                }
              };
            }
          }
        }
      }
    });

    tasks.spawn_local(async move {
      let future = if config.tunnel {
        server
          .run_connection_with(stream, TransportTls::AlreadyDone)
          .await
      } else {
        server.run_connection(stream).await
      };

      if let Err(error) = future {
        if Self::is_expected_disconnect(&error) {
          tracing::debug!(?error, "RDP connection closed");
        } else {
          tracing::error!(?error, "Connection dropped");
        }
      }
    });

    tracing::info("running looppppp");

    loop {
      tokio::select! {
        _ = cancellation_token.cancelled() => break,
        result = tasks.join_next() => {
            match result {
                Some(Ok(())) => {
                    tracing::warn!("a supervised IPC/RDP task exited");
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
