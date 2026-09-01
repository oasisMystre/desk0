use axum::{
  Router,
  routing::{delete, get, patch, post},
};
use clap::Parser;
use shared::init::load_env;
use tokio::net::TcpListener;

use crate::context::Context;

mod auth;
mod context;
mod env;
mod routes;

#[derive(Parser)]
struct Args {
  #[arg(long, default_value = "0.0.0,0")]
  host: String,

  #[arg(long, default_value = "8000")]
  port: String,
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
  load_env();
  let args = Args::parse();
  let context = Context::from_env().await?;

  let app = Router::new()
    .route(
      "/organizations",
      post(routes::organizations::create_organization),
    )
    .route(
      "/organizations/:device_id",
      post(routes::organizations::create_connection_token),
    )
    .route("/devices", get(routes::devices::get_devices))
    .route("/devices", post(routes::devices::create_device))
    .route("/devices/:device_id", patch(routes::devices::update_device))
    .route(
      "/devices/:device_id",
      delete(routes::devices::delete_device),
    )
    .with_state(context);

  let addr = format!("{}:{}", args.host, args.port);
  let listener = TcpListener::bind(&addr).await?;
  tracing::info!("listening on {}", addr);

  tokio::select! {
      _ = tokio::signal::ctrl_c() => {
          tracing::info!("server exitted");
    }
     result = axum::serve(listener, app) => {
        if let Err(error) = result {
            tracing::error!(?error, "server halt");
        }
     }
  }

  Ok(())
}
