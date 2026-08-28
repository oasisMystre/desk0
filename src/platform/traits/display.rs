use ironrdp::server::DisplayUpdate;
use tokio::sync::mpsc::Sender;

pub trait Display {
  async fn capture(width: u16, height: u16, sender: Sender<DisplayUpdate>);
}
