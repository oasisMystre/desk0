use crate::{context::Context, routes::ws::context::WsContext};

#[derive(Clone)]
pub struct AppState {
  pub context: Context,
  pub ws_context: WsContext,
}

impl AppState {
  pub async fn init() -> anyhow::Result<Self> {
    let ws_context = WsContext::from_env()?;
    let context = Context::from_env().await?;

    Ok(Self {
      context,
      ws_context,
    })
  }
}
