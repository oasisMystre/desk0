use tokio_util::sync::CancellationToken;

pub trait Server {
   fn run(&self, cancellation_token: CancellationToken);
}

#[async_trait::async_trait]
pub trait Tunnel {
  async fn run(&self, cancellation_token: CancellationToken) -> anyhow::Result<()>;
}
