use std::sync::Arc;

use crate::{context::Context, routes::ws::context::WsContext};

#[derive(Clone)]
pub struct AppState {
  pub context: Context,
  pub ws_context: WsContext,
}

impl AppState {
  pub async fn init() -> anyhow::Result<Self> {
    let context = Arc::new(Context::from_env().await?);
    let ws_context = WsContext::from_env(context.clone())?;

    Ok(Self {
      ws_context,
      context: Arc::unwrap_or_clone(context),
    })
  }
}
