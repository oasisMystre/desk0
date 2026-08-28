use ironrdp::server::{DisplayUpdate, ServerEvent};
use tokio::sync::mpsc::UnboundedSender;

pub trait Display {
  async fn capture(
    width: u16,
    height: u16,
    sender: UnboundedSender<DisplayUpdate>,
    audio_sender: Option<UnboundedSender<ServerEvent>>,
  );
}
