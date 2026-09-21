use tokio_util::sync::CancellationToken;

#[async_trait::async_trait(?Send)]
pub trait Server {
  async fn run(&self, cancellation_token: CancellationToken) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
pub trait Tunnel {
  async fn run(&self, cancellation_token: CancellationToken) -> anyhow::Result<()>;
}
