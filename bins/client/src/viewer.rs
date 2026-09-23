use std::thread;

use anyhow::Context;
use copypaste::CliprdrHandler;
use display_info::DisplayInfo;
use ironrdp::{
  client::{
    config::{ClipboardType, ConfigBuilder, Destination, TransportKind},
    rdp::{RdpClient, RdpInputEvent, RdpOutputEvent},
  },
  cliprdr::{Cliprdr, backend::CliprdrBackendFactory},
  pdu::rdp::capability_sets::MajorPlatformType,
};
use tokio::sync::mpsc::{Receiver, Sender, UnboundedSender, channel};
use tokio_util::sync::CancellationToken;

pub type RdpOutputEventCallback = fn(event: RdpOutputEvent);

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
}

pub struct Desk0Viewer {
  client: Option<RdpClient>,
  pub clipboard: CliprdrHandler,
  pub thread: Option<thread::JoinHandle<()>>,
  pub input: UnboundedSender<RdpInputEvent>,
  _cancellation_token: CancellationToken,
}

impl Desk0Viewer {
  pub const CLIENT_NAME: &str = "Desk0 Viewer";
  pub const THREAD_NAME: &str = "desk0-rdp-viewer";

  #[allow(dead_code)]
  pub fn with_callback<'a>(
    config: Desk0ViewerConfig<'a>,
    callback: RdpOutputEventCallback,
  ) -> anyhow::Result<Self, Desk0ViewerError> {
    let rt = tokio::runtime::Builder::new_multi_thread()
      .enable_all()
      .build()
      .context("unable to create tokio runtime")?;

    let (output_event_sender, output_event_receiver) = channel(32);
    let mut this = Self::build(config, output_event_sender)?;
    this.run(rt, Some((callback, output_event_receiver)))?;

    Ok(this)
  }

  pub fn with_channel<'a>(
    config: Desk0ViewerConfig<'a>,
    output_event_sender: Sender<RdpOutputEvent>,
  ) -> anyhow::Result<Self, Desk0ViewerError> {
    Self::build(config, output_event_sender)
  }

  fn build<'a>(
    config: Desk0ViewerConfig<'a>,
    output_event_sender: Sender<RdpOutputEvent>,
  ) -> anyhow::Result<Self, Desk0ViewerError> {
    let mut endpoint =
      url::Url::parse(config.endpoint).map_err(Desk0ViewerError::InvalidEndpoint)?;
    if endpoint.scheme() == "http" {
      endpoint.set_scheme("ws").ok();
    } else if endpoint.scheme() == "https" {
      endpoint.set_scheme("wss").ok();
    }

    let mut clipboard_handler = CliprdrHandler::new();

    let displays = DisplayInfo::all().map_err(Desk0ViewerError::Display)?;

    let to_u16 =
      |value: u32| -> Option<u16> { u16::try_from(value).ok().filter(|&value| value > 0) };

    let (width, height) = displays
      .iter()
      .find(|display| display.is_primary)
      .or_else(|| displays.first())
      .and_then(|display| Some((to_u16(display.width)?, to_u16(display.height)?)))
      .unwrap_or((1920, 1080));

    let destination = Destination::new("").unwrap();

    let config = ConfigBuilder::new()
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
      .with_desktop_width(width)
      .with_desktop_height(height)
      .with_clipboard(ClipboardType::Enable)
      .with_pointer_software_rendering(true)
      .with_client_name(Self::client_name())
      .with_client_build(Self::client_build())
      .with_client_dir("C:\\Windows\\System32\\mstscax.dll")
      .with_static_channel({
        let clipboard_handler = clipboard_handler.clone();
        move |_| Some(Cliprdr::new(clipboard_handler.build_cliprdr_backend()))
      })
      .build()
      .map_err(Desk0ViewerError::Configuration)?;

    let client = RdpClient::new(config, output_event_sender);
    let input = client.input_sender();

    clipboard_handler.set_sender(input.clone());

    Ok(Self {
      input,
      thread: None,
      client: Some(client),
      clipboard: clipboard_handler,
      _cancellation_token: CancellationToken::new(),
    })
  }

  pub fn run(
    &mut self,
    rt: tokio::runtime::Runtime,
    callback: Option<(RdpOutputEventCallback, Receiver<RdpOutputEvent>)>,
  ) -> anyhow::Result<(), Desk0ViewerError> {
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
                    _ = &mut runner => break,
                    event = receiver.recv() => {
                        match event {
                            Some(event) => callback(event),
                            None => break,
                        }
                    }
                  }
                }

                drop(input);
              });
            }
          })
          .map_err(|error| {
            tracing::error!(?error, "input closed");
            Desk0ViewerError::InputClosed
          })?
      } else {
        let input = self.input.clone();
        let cancellation_token = self._cancellation_token.clone();
        thread::spawn(move || {
          rt.block_on(async {
            tokio::select! {
                result = client.run() => result,
                _ = cancellation_token.cancelled() => {
                    if let Err(error) = input.send(RdpInputEvent::Close) {
                       tracing::error!(?error, "failed to gracefully close client");
                    }
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
    if let Err(error) = self.input.send(RdpInputEvent::Close) {
      tracing::debug!(?error, "input channel closed");
    }

    if let Some(thread) = self.thread.take() {
      if let Err(error) = thread.join() {
        tracing::error!(?error, "closing input thread error");
      }
    }
  }
}

impl Drop for Desk0Viewer {
  fn drop(&mut self) {
    self.shutdown();
  }
}
