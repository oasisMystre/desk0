use ironrdp::server::{DisplayUpdate, ServerEvent};
use tokio::sync::mpsc::{Sender, UnboundedSender};

pub trait Display {
  async fn capture(
    width: u16,
    height: u16,
    sender: Sender<DisplayUpdate>,
    audio_sender: Option<UnboundedSender<ServerEvent>>,
  );
}
