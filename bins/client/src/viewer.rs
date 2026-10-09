use std::{
  sync::{Arc, Mutex},
  thread,
};

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
use pdu::{io::IoOutputEvent, terminal::TerminalOutputEvent};
use tokio::sync::mpsc::{UnboundedSender, error::TrySendError};
use tokio_util::sync::CancellationToken;
use winit::dpi::PhysicalSize;

use crate::static_channel::{IoStaticChannel, TerminalStaticChannel};

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

#[derive(Clone)]
pub enum Desk0ViewerOutput {
  #[cfg(feature = "ffi")]
  Callback {
    rdp_output_event_callback:
      Arc<Mutex<Option<Arc<dyn Fn(RdpOutputEvent) + 'static + Sync + Send>>>>,
    clipboard_event_callback:
      Arc<Mutex<Option<Arc<dyn Fn(ClipboardEvent) + 'static + Sync + Send>>>>,
    io_output_event_callback:
      Arc<Mutex<Option<Arc<dyn Fn(IoOutputEvent) + Sync + Send + 'static>>>>,
    terminal_output_event_callback:
      Arc<Mutex<Option<Arc<dyn Fn(TerminalOutputEvent) + Sync + Send + 'static>>>>,
  },
  Channel {
    rdp_output_event_sender: OutputEventSender,
    clipboard_event_sender: Option<UnboundedSender<ClipboardEvent>>,
    io_output_event_sender: Option<UnboundedSender<IoOutputEvent>>,
    terminal_output_event_sender: Option<UnboundedSender<TerminalOutputEvent>>,
  },
}

pub struct Desk0ViewerConfig<'a> {
  pub token: &'a str,
  pub endpoint: &'a str,
  pub desktop_size: Option<PhysicalSize<u16>>,
}

pub struct Desk0Viewer {
  output: Desk0ViewerOutput,
  client: Option<RdpClient>,

  pub database: Database,
  pub input: RdpInputSender,
  pub thread: Option<thread::JoinHandle<()>>,
  _cancellation_token: CancellationToken,
}

impl Desk0ViewerOutput {
  #[cfg(feature = "ffi")]
  pub fn callback() -> Self {
    return Self::Callback {
      rdp_output_event_callback: Arc::new(Mutex::new(None)),
      clipboard_event_callback: Arc::new(Mutex::new(None)),
      io_output_event_callback: Arc::new(Mutex::new(None)),
      terminal_output_event_callback: Arc::new(Mutex::new(None)),
    };
  }

  pub fn send_rdp_output_event(&self, event: RdpOutputEvent) {
    match self {
      #[cfg(feature = "ffi")]
      Self::Callback {
        rdp_output_event_callback,
        ..
      } => {
        let callback = rdp_output_event_callback.lock().unwrap().clone();

        if let Some(callback) = callback {
          (callback)(event);
        }
      }
      Desk0ViewerOutput::Channel {
        rdp_output_event_sender,
        ..
      } => {
        let _ = rdp_output_event_sender.send(event);
      }
    }
  }

  pub fn send_clipboard_event(&self, event: ClipboardEvent) {
    match self {
      #[cfg(feature = "ffi")]
      Self::Callback {
        clipboard_event_callback,
        ..
      } => {
        let callback = clipboard_event_callback.lock().unwrap().clone();

        if let Some(callback) = callback {
          (callback)(event);
        }
      }
      Self::Channel {
        clipboard_event_sender,
        ..
      } => {
        if let Some(sender) = clipboard_event_sender {
          let _ = sender.send(event);
        };
      }
    }
  }

  pub fn send_io_output_event(&self, event: IoOutputEvent) {
    match self {
      #[cfg(feature = "ffi")]
      Self::Callback {
        io_output_event_callback,
        ..
      } => {
        let callback = io_output_event_callback.lock().unwrap().clone();

        if let Some(callback) = callback {
          (callback)(event);
        }
      }
      Self::Channel {
        io_output_event_sender,
        ..
      } => {
        if let Some(sender) = io_output_event_sender {
          let _ = sender.send(event);
        }
      }
    }
  }
  pub fn send_terminal_output_event(&self, event: TerminalOutputEvent) {
    match self {
      #[cfg(feature = "ffi")]
      Self::Callback {
        terminal_output_event_callback,
        ..
      } => {
        let callback = terminal_output_event_callback.lock().unwrap().clone();

        if let Some(callback) = callback {
          (callback)(event);
        }
      }
      Self::Channel {
        terminal_output_event_sender,
        ..
      } => {
        if let Some(sender) = terminal_output_event_sender {
          let _ = sender.send(event);
        }
      }
    }
  }

  #[cfg(feature = "ffi")]
  pub fn set_rdp_output_event_callback<F>(&self, callback: F)
  where
    F: Fn(RdpOutputEvent) + Send + Sync + 'static,
  {
    if let Desk0ViewerOutput::Callback {
      rdp_output_event_callback,
      ..
    } = self
    {
      *rdp_output_event_callback.lock().unwrap() = Some(Arc::new(callback));
    }
  }

  #[cfg(feature = "ffi")]
  pub fn set_clipboard_event_callback<F>(&self, callback: F)
  where
    F: Fn(ClipboardEvent) + Send + Sync + 'static,
  {
    if let Desk0ViewerOutput::Callback {
      clipboard_event_callback,
      ..
    } = self
    {
      *clipboard_event_callback.lock().unwrap() = Some(Arc::new(callback));
    }
  }

  #[cfg(feature = "ffi")]
  pub fn set_io_output_event_callback<F>(&self, callback: F)
  where
    F: Fn(IoOutputEvent) + Send + Sync + 'static,
  {
    if let Desk0ViewerOutput::Callback {
      io_output_event_callback,
      ..
    } = self
    {
      *io_output_event_callback.lock().unwrap() = Some(Arc::new(callback));
    }
  }

  #[cfg(feature = "ffi")]
  pub fn set_terminal_output_event_callback<F>(&self, callback: F)
  where
    F: Fn(TerminalOutputEvent) + Send + Sync + 'static,
  {
    if let Desk0ViewerOutput::Callback {
      terminal_output_event_callback,
      ..
    } = self
    {
      *terminal_output_event_callback.lock().unwrap() = Some(Arc::new(callback));
    }
  }
}

impl Desk0Viewer {
  pub const CLIENT_NAME: &str = "Desk0 Viewer";
  pub const THREAD_NAME: &str = "desk0-rdp-viewer";

  pub fn new<'a>(
    rt: tokio::runtime::Runtime,
    config: Desk0ViewerConfig<'a>,
    output: Desk0ViewerOutput,
  ) -> anyhow::Result<Self, Desk0ViewerError> {
    let (output_event_sender, output_event_receiver) = output_channel(32);
    let mut this = Self::new_inner(config, output, output_event_sender)?;
    this.run(rt, output_event_receiver)?;

    Ok(this)
  }

  fn new_inner<'a>(
    config: Desk0ViewerConfig<'a>,
    output: Desk0ViewerOutput,
    rdp_output_event_sender: OutputEventSender,
  ) -> anyhow::Result<Self, Desk0ViewerError> {
    let mut endpoint =
      url::Url::parse(config.endpoint).map_err(Desk0ViewerError::InvalidEndpoint)?;
    if endpoint.scheme() == "http" {
      endpoint.set_scheme("ws").ok();
    } else if endpoint.scheme() == "https" {
      endpoint.set_scheme("wss").ok();
    }

    let clipboard_handler = CliprdrHandler::with_callback({
      let output = output.clone();

      move |event| {
        output.send_clipboard_event(event);
      }
    });

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
      })
      .with_static_channel({
        let output = output.clone();
        move |_| {
          let output = output.clone();

          Some(IoStaticChannel::new(move |event| {
            output.send_io_output_event(event);
          }))
        }
      })
      .with_static_channel({
        let output = output.clone();
        move |_| {
          let output = output.clone();

          Some(TerminalStaticChannel::new(move |event| {
            output.send_terminal_output_event(event);
          }))
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

    let client = RdpClient::new(config, rdp_output_event_sender).with_desktop_updates();
    let input = client.input_sender();

    let mut clipboard_handler = clipboard_handler.lock().unwrap();
    clipboard_handler.set_sender(input.clone());
    drop(clipboard_handler);

    Ok(Self {
      input,
      output,
      thread: None,
      client: Some(client),
      database: Database::new(),
      _cancellation_token: CancellationToken::new(),
    })
  }

  pub fn run(
    &mut self,
    rt: tokio::runtime::Runtime,
    mut rdp_output_event_receiver: OutputEventReceiver,
  ) -> anyhow::Result<(), Desk0ViewerError> {
    let client = self.client.take();
    if let Some(client) = client {
      let thread = thread::Builder::new()
        .name(Self::THREAD_NAME.into())
        .spawn({
          let input = self.input.clone();
          let output = self.output.clone();

          move || {
            rt.block_on(async move {
              let runner = client.run();
              tokio::pin!(runner);

              loop {
                tokio::select! {
                  biased;
                  event = rdp_output_event_receiver.recv() => {
                      match event {
                          Some(event) => output.send_rdp_output_event(event),
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
        })?;

      self.thread = Some(thread);
    }

    Ok(())
  }

  #[cfg(feature = "ffi")]
  pub fn set_rdp_output_event_callback<F>(&self, callback: F)
  where
    F: Fn(RdpOutputEvent) + Send + Sync + 'static,
  {
    self.output.set_rdp_output_event_callback(callback);
  }

  #[cfg(feature = "ffi")]
  pub fn set_clipboard_event_callback<F>(&self, callback: F)
  where
    F: Fn(ClipboardEvent) + Send + Sync + 'static,
  {
    self.output.set_clipboard_event_callback(callback);
  }

  #[cfg(feature = "ffi")]
  pub fn set_io_output_event_callback<F>(&self, callback: F)
  where
    F: Fn(IoOutputEvent) + Send + Sync + 'static,
  {
    self.output.set_io_output_event_callback(callback);
  }

  #[cfg(feature = "ffi")]
  pub fn set_terminal_output_event_callback<F>(&self, callback: F)
  where
    F: Fn(TerminalOutputEvent) + Send + Sync + 'static,
  {
    self.output.set_terminal_output_event_callback(callback);
  }

  fn platform() -> MajorPlatformType {
    #[cfg(target_os = "windows")]
    return MajorPlatformType::WINDOWS;

    #[cfg(target_os = "macos")]
    return MajorPlatformType::MACINTOSH;

    #[cfg(target_os = "linux")]
    return MajorPlatformType::UNIX;

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    return MajorPlatformType::UNSPECIFIED;
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
      match self.input.try_send(RdpInputEvent::FastPath(events)) {
        Ok(()) => {}
        Err(TrySendError::Full(_)) => {
          tracing::warn!("rdp input queue full");
        }
        Err(TrySendError::Closed(_)) => {
          tracing::error!("rdp input channel closed");
        }
      }
    }
  }
}

impl Drop for Desk0Viewer {
  fn drop(&mut self) {
    self.shutdown();
  }
}
