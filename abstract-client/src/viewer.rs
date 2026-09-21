use crate::handlers::cliprdr::ClipboardBridge;
use ironrdp::{
  cliprdr::Cliprdr,
  pdu::{
    input::{
      fast_path::{FastPathInputEvent, KeyboardFlags},
      mouse::{MousePdu, PointerFlags},
    },
    rdp::capability_sets::MajorPlatformType,
  },
};
use ironrdp_client::{
  config::{ClipboardType, ConfigBuilder, Destination, TransportKind},
  rdp::{RdpClient, RdpInputEvent, RdpOutputEvent},
};
use smallvec::smallvec;
use std::{num::NonZeroU16, sync::Arc, thread};
use tokio::sync::{
  Mutex,
  mpsc::{self, UnboundedSender},
};

pub type FrameCallback = extern "C" fn(*const u8, u16, u16);
pub type StatusCallback = extern "C" fn(i32, *const u8, usize);
pub use crate::handlers::cliprdr::ClipboardCallback;

#[derive(Debug)]
pub enum ViewerError {
  Endpoint(url::ParseError),
  Config(anyhow::Error),
  Thread(std::io::Error),
}

pub struct Viewer {
  input: UnboundedSender<RdpInputEvent>,
  thread: Option<thread::JoinHandle<()>>,
  clipboard: ClipboardBridge,
}

impl Viewer {
  pub fn start(
    endpoint: String,
    token: String,
    username: String,
    password: String,
    on_frame: Option<FrameCallback>,
    on_status: Option<StatusCallback>,
    on_clipboard: Option<ClipboardCallback>,
  ) -> Result<Self, ViewerError> {
    let mut url = url::Url::parse(&endpoint).map_err(ViewerError::Endpoint)?;
    if url.scheme() == "http" {
      let _ = url.set_scheme("ws");
    } else if url.scheme() == "https" {
      let _ = url.set_scheme("wss");
    }
    let bridge = ClipboardBridge::new(on_clipboard);
    let destination = Destination::new("").map_err(ViewerError::Config)?;
    let bridge_for_channel = bridge.clone();
    let config = ConfigBuilder::new()
      .with_destination(destination)
      .with_username(username)
      .with_password(password)
      .with_transport(TransportKind::RDCleanPath { url })
      .with_rdcleanpath_token(token)
      .with_platform(platform())
      .with_tls(false)
      .with_credssp(false)
      .with_desktop_width(1920)
      .with_desktop_height(1080)
      .with_clipboard(ClipboardType::Disable)
      .with_pointer_software_rendering(true)
      .with_client_name("Abstract Client")
      .with_static_channel(move |_| Some(Cliprdr::new(Box::new(bridge_for_channel.backend()))))
      .build()
      .map_err(ViewerError::Config)?;
    let (outputs, mut events) = mpsc::channel(64);
    let client = RdpClient::new(config, outputs);
    let input = client.input_sender();
    bridge.set_input(input.clone());
    let thread = thread::Builder::new().name("abstract-rdp-viewer".into()).spawn(move || {
            let Ok(runtime) = tokio::runtime::Builder::new_current_thread().enable_all().build() else { return };
            runtime.block_on(async move {
                let run = client.run(); tokio::pin!(run);
                loop { tokio::select! {
                    _ = &mut run => { status(on_status, -1, "terminated"); break; }
                    event = events.recv() => match event {
                        Some(RdpOutputEvent::Image { buffer, width, height }) => frame(on_frame, &buffer, width, height),
                        Some(RdpOutputEvent::ConnectionFailure(error)) => { status(on_status, -2, &error.to_string()); break; }
                        Some(RdpOutputEvent::Terminated(_)) => { status(on_status, 0, "terminated"); break; }
                        Some(_) => {}
                        None => break,
                    }
                }}
            });
        }).map_err(ViewerError::Thread)?;
    Ok(Self {
      input,
      thread: Some(thread),
      clipboard: bridge,
    })
  }
  pub fn mouse_move(&self, x: u16, y: u16) {
    self.send(FastPathInputEvent::MouseEvent(MousePdu {
      flags: PointerFlags::MOVE,
      number_of_wheel_rotation_units: 0,
      x_position: x,
      y_position: y,
    }));
  }
  pub fn mouse_button(&self, button: u8, pressed: bool, x: u16, y: u16) {
    let flag = match button {
      0 => PointerFlags::LEFT_BUTTON,
      1 => PointerFlags::MIDDLE_BUTTON_OR_WHEEL,
      2 => PointerFlags::RIGHT_BUTTON,
      _ => return,
    };
    let mut flags = flag;
    if pressed {
      flags |= PointerFlags::DOWN;
    }
    self.send(FastPathInputEvent::MouseEvent(MousePdu {
      flags,
      number_of_wheel_rotation_units: 0,
      x_position: x,
      y_position: y,
    }));
  }
  pub fn key(&self, scancode: u8, pressed: bool, extended: bool) {
    let mut flags = KeyboardFlags::empty();
    if !pressed {
      flags |= KeyboardFlags::RELEASE;
    }
    if extended {
      flags |= KeyboardFlags::EXTENDED;
    }
    self.send(FastPathInputEvent::KeyboardEvent(flags, scancode));
  }
  fn send(&self, event: FastPathInputEvent) {
    let _ = self.input.send(RdpInputEvent::FastPath(smallvec![event]));
  }
  pub fn set_clipboard_text(&self, text: String) -> Result<(), ()> {
    self.clipboard.set_text(text)
  }
}

impl Drop for Viewer {
  fn drop(&mut self) {
    let _ = self.input.send(RdpInputEvent::Close);
    if let Some(thread) = self.thread.take() {
      let _ = thread.join();
    }
  }
}

fn frame(callback: Option<FrameCallback>, buffer: &[u32], width: NonZeroU16, height: NonZeroU16) {
  if let Some(callback) = callback {
    callback(buffer.as_ptr().cast(), width.get(), height.get());
  }
}
fn status(callback: Option<StatusCallback>, code: i32, message: &str) {
  if let Some(callback) = callback {
    callback(code, message.as_ptr(), message.len());
  }
}
fn platform() -> MajorPlatformType {
  #[cfg(target_os = "windows")]
  {
    return MajorPlatformType::WINDOWS;
  }
  #[cfg(target_os = "macos")]
  {
    return MajorPlatformType::MACINTOSH;
  }
  #[cfg(target_os = "linux")]
  {
    return MajorPlatformType::UNIX;
  }
  #[allow(unreachable_code)]
  MajorPlatformType::UNIX
}
