use clap::Parser;
use ironrdp::client::rdp::RdpOutputEvent;
use tokio::sync::mpsc::channel;
use winit::{dpi::PhysicalSize, event_loop::EventLoop};

use crate::{
  app::Desk0App,
  viewer::{Desk0Viewer, Desk0ViewerConfig},
};

mod app;
mod keymap;
mod viewer;

/// Desk0 RDP Viewer
#[derive(Parser, Debug)]
#[command(name = "desk0-viewer", version, about, long_about = None)]
struct Args {
  /// RDCleanPath / websocket endpoint to connect to
  #[arg(long)]
  endpoint: String,

  /// Auth token for the RDP session
  #[arg(long)]
  token: String,

  /// display width (must be passed together with --height)
  #[arg(long, requires = "height")]
  width: Option<u32>,

  /// display height (must be passed together with --width)
  #[arg(long, requires = "width")]
  height: Option<u32>,
}

fn main() -> anyhow::Result<()> {
  let args = Args::parse();

  let initial_size = match (args.width, args.height) {
    (Some(width), Some(height)) => Some(PhysicalSize::new(width, height)),
    (None, None) => None,
    _ => anyhow::bail!("--width and --height must be provided together"),
  };

  let (output_event_sender, mut output_event_receiver) = channel::<RdpOutputEvent>(64);

  let viewer = Desk0Viewer::with_channel(
    Desk0ViewerConfig {
      token: &args.token,
      endpoint: &args.endpoint,
    },
    output_event_sender,
  )?;

  let event_loop = EventLoop::<RdpOutputEvent>::with_user_event().build()?;
  let proxy = event_loop.create_proxy();
  let mut app = Desk0App::new(&event_loop, viewer, initial_size)?;

  let bridge_thread = std::thread::Builder::new()
    .name(Desk0Viewer::THREAD_NAME.into())
    .spawn(move || {
      let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
      {
        Ok(rt) => rt,
        Err(error) => {
          eprintln!("failed to build tokio runtime: {error}");
          return;
        }
      };

      rt.block_on(async move {
        while let Some(event) = output_event_receiver.recv().await {
          if proxy.send_event(event).is_err() {
            break;
          }
        }
      });
    })?;

  event_loop.run_app(&mut app)?;

  if let Err(error) = bridge_thread.join() {
    eprintln!("tokio bridge thread panicked: {error:?}");
  }

  Ok(())
}
