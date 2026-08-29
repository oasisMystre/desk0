use ironrdp::server::DisplayUpdate;
use tokio::sync::mpsc::UnboundedSender;

pub trait Display {
  async fn capture(&self, sender: UnboundedSender<DisplayUpdate>) -> anyhow::Result<()>;
}
