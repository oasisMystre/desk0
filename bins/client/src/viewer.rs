use std::{
  sync::{Arc, Mutex},
  thread,
};

use anyhow::Context;
use copypaste::{CliprdrHandler, cliprdr::types::clipboard_event::ClipboardEvent};
use ironrdp::{
  client::{
    config::{ClipboardType, ConfigBuilder, Destination, TransportKind},
    output_channel::{OutputEventReceiver, OutputEventSender, output_channel},
    rdp::{RdpClient, RdpInputEvent, RdpInputSender, RdpOutputEvent},
  },
  cliprdr::{Cliprdr, backend::CliprdrBackendFactory},
  input::{Database, Operation},
  pdu::rdp::capability_sets::MajorPlatformType,
};
use tokio_util::sync::CancellationToken;
use winit::dpi::PhysicalSize;

#[derive(Debug, thiserror::Error)]
pub enum Desk0ViewerError {
  #[error("input channel closed")]
  InputClosed,
  #[error("thread error: {0}")]
  Thread(#[from] std::io::Error),
  #[error("configuration error: {0}")]
  Configuration(#[from] anyhow::Error),
  #[error("invalid endpoint: {0}")]
  InvalidEndpoint(#[from] url::ParseError),
  #[error("display error: {0}")]
  Display(#[from] display_info::error::DIError),
}

pub struct Desk0ViewerConfig<'a> {
  pub token: &'a str,
  pub endpoint: &'a str,
  pub desktop_size: Option<PhysicalSize<u16>>,
}

pub struct Desk0Viewer {
  client: Option<RdpClient>,
  pub database: Database,
  pub input: RdpInputSender,
  pub thread: Option<thread::JoinHandle<()>>,
  _cancellation_token: CancellationToken,
}

impl Desk0Viewer {
  pub const CLIENT_NAME: &str = "Desk0 Viewer";
  pub const THREAD_NAME: &str = "desk0-rdp-viewer";

  #[allow(dead_code)]
  pub fn with_callback<'a, T, U>(
    config: Desk0ViewerConfig<'a>,
    rdp_output_event_callback: T,
    clipboard_event_callback: U,
  ) -> anyhow::Result<Self, Desk0ViewerError>
  where
    T: Fn(RdpOutputEvent) + 'static + Sync + Send,
    U: Fn(ClipboardEvent) + 'static + Sync + Send,
  {
    let rt = tokio::runtime::Builder::new_multi_thread()
      .enable_all()
      .build()
      .context("unable to create tokio runtime")?;

    let (output_event_sender, output_event_receiver) = output_channel(32);
    let mut this = Self::build(config, output_event_sender, clipboard_event_callback)?;
    this.run(rt, Some((rdp_output_event_callback, output_event_receiver)))?;

    Ok(this)
  }

  pub fn with_channel<'a, U>(
    config: Desk0ViewerConfig<'a>,
    output_event_sender: OutputEventSender,
    clipboard_event_callback: U,
  ) -> anyhow::Result<Self, Desk0ViewerError>
  where
    U: Fn(ClipboardEvent) + 'static + Sync + Send,
  {
    Self::build(config, output_event_sender, clipboard_event_callback)
  }

  fn build<'a, U>(
    config: Desk0ViewerConfig<'a>,
    output_event_sender: OutputEventSender,
    clipboard_event_callback: U,
  ) -> anyhow::Result<Self, Desk0ViewerError>
  where
    U: Fn(ClipboardEvent) + 'static + Sync + Send,
  {
    let mut endpoint =
      url::Url::parse(config.endpoint).map_err(Desk0ViewerError::InvalidEndpoint)?;
    if endpoint.scheme() == "http" {
      endpoint.set_scheme("ws").ok();
    } else if endpoint.scheme() == "https" {
      endpoint.set_scheme("wss").ok();
    }

    let clipboard_handler = CliprdrHandler::with_callback(clipboard_event_callback);
    let clipboard_handler = Arc::new(Mutex::new(clipboard_handler));

    let destination = Destination::new("127.0.0.1/0").unwrap();

    let mut config_builder = ConfigBuilder::new()
      .with_destination(destination)
      .with_username("username")
      .with_password("username")
      .with_transport(TransportKind::RDCleanPath { url: endpoint })
      .with_rdcleanpath_token(config.token)
      .with_platform(Self::platform())
      .with_tls(true)
      .with_credssp(false)
      .with_client_dir(
        std::env::current_exe()
          .unwrap()
          .to_string_lossy()
          .to_string(),
      )
      .with_autologon(true)
      .with_clipboard(ClipboardType::Enable)
      .with_pointer_software_rendering(false)
      .with_client_name(Self::client_name())
      .with_client_build(Self::client_build())
      .with_client_dir("C:\\Windows\\System32\\mstscax.dll")
      .with_static_channel({
        let clipboard_handler = clipboard_handler.clone();
        move |_| {
          let backend = {
            let handler = clipboard_handler.lock().unwrap();
            handler.build_cliprdr_backend()
          };
          Some(Cliprdr::new(backend))
        }
      });

    if let Some(desktop_size) = config.desktop_size {
      config_builder = config_builder
        .with_desktop_width(desktop_size.width)
        .with_desktop_height(desktop_size.height);
    }

    let config = config_builder
      .build()
      .map_err(Desk0ViewerError::Configuration)?;

    let client = RdpClient::new(config, output_event_sender).with_desktop_updates();
    let input = client.input_sender();

    let mut clipboard_handler = clipboard_handler.lock().unwrap();
    clipboard_handler.set_sender(input.clone());
    drop(clipboard_handler);

    Ok(Self {
      input,
      thread: None,
      client: Some(client),
      database: Database::new(),
      _cancellation_token: CancellationToken::new(),
    })
  }

  pub fn run<T>(
    &mut self,
    rt: tokio::runtime::Runtime,
    callback: Option<(T, OutputEventReceiver)>,
  ) -> anyhow::Result<(), Desk0ViewerError>
  where
    T: Fn(RdpOutputEvent) + 'static + Send + Sync,
  {
    let client = self.client.take();
    if let Some(client) = client {
      let thread = if let Some(callback) = callback {
        thread::Builder::new()
          .name(Self::THREAD_NAME.into())
          .spawn({
            let input = self.input.clone();
            move || {
              rt.block_on(async move {
                let (callback, mut receiver) = callback;
                let runner = client.run();
                tokio::pin!(runner);

                loop {
                  tokio::select! {
                    biased;
                    event = receiver.recv() => {
                        match event {
                            Some(event) => callback(event),
                            None => break,
                        }
                    }
                    _ = &mut runner => {},
                  }
                }

                tracing::info!("stopping thread");

                drop(input);
              });
            }
          })
          .map_err(|error| {
            tracing::error!(?error, "input closed");
            Desk0ViewerError::InputClosed
          })?
      } else {
        let cancellation_token = self._cancellation_token.clone();
        thread::spawn(move || {
          rt.block_on(async {
            tokio::select! {
                result = client.run() => result,
                _ = cancellation_token.cancelled() => {
                    return;
                }
            }
          });
        })
      };

      self.thread = Some(thread);
    }

    Ok(())
  }

  fn platform() -> MajorPlatformType {
    #[cfg(target_os = "windows")]
    return MajorPlatformType::WINDOWS;

    #[cfg(target_os = "macos")]
    return MajorPlatformType::MACINTOSH;
  }

  pub fn client_name() -> String {
    whoami::hostname().unwrap_or_else(|_| Self::CLIENT_NAME.to_owned())
  }

  pub fn client_build() -> u32 {
    semver::Version::parse(env!("CARGO_PKG_VERSION"))
      .map_or(0, |value| {
        value.major * 100 + value.minor * 10 + value.patch
      })
      .try_into()
      .unwrap_or(0)
  }

  pub fn shutdown(&mut self) {
    self._cancellation_token.cancel();
    if let Some(thread) = self.thread.take() {
      if let Err(error) = thread.join() {
        tracing::error!(?error, "closing input thread error");
      }
    }
  }

  pub fn apply_and_send_fast_path_events(
    &mut self,
    operations: impl IntoIterator<Item = Operation>,
  ) {
    let events = self.database.apply(operations);

    if !events.is_empty() {
      if let Err(error) = self.input.try_send(RdpInputEvent::FastPath(events)) {
        tracing::error!(?error, "rdp input closed");
      }
    }
  }
}

impl Drop for Desk0Viewer {
  fn drop(&mut self) {
    self.shutdown();
  }
}
