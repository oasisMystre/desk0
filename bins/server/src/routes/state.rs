use std::sync::Arc;

use bullmq::{Queue, QueueOptions};

use crate::{context::Context, routes::ws::context::WsContext};

#[derive(Clone)]
pub struct AppState {
  pub queue: Queue,
  pub context: Arc<Context>,
  pub ws_context: WsContext,
}

impl AppState {
  pub async fn init() -> anyhow::Result<Self> {
    let context = Arc::new(Context::from_env().await?);
    let ws_context = WsContext::from_env(context.clone())?;

    let queue = Queue::with_options("desk0", QueueOptions::default()).await?;

    Ok(Self {
      queue,
      context,
      ws_context,
    })
  }
}
