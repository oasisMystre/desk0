use axum::{
  Router,
  routing::{delete, get, patch, post},
};
use clap::Parser;
use shared::init::{init_tracing, load_env};
use tokio::net::TcpListener;

use crate::context::Context;

mod auth;
mod context;
mod env;
mod routes;

#[derive(Parser)]
struct Args {
  #[arg(long, default_value = "0.0.0.0")]
  host: String,

  #[arg(long, default_value = "8000")]
  port: String,
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
  load_env();
  init_tracing()?;

  let args = Args::parse();
  let context = Context::from_env().await?;

  let app = Router::new()
    .route("/users", post(routes::users::create_user))
    .route("/user", delete(routes::users::delete_user))
    .route(
      "/organizations",
      get(routes::organizations::get_organizations),
    )
    .route(
      "/organizations",
      post(routes::organizations::create_organization),
    )
    .route(
      "/organizations/token/{device_id}",
      post(routes::organizations::create_connection_token),
    )
    .route("/devices", get(routes::devices::get_devices))
    .route("/devices", post(routes::devices::create_device))
    .route(
      "/devices/{device_id}",
      patch(routes::devices::update_device),
    )
    .route(
      "/devices/{device_id}",
      delete(routes::devices::delete_device),
    )
    .route("/attachments", get(routes::attachments::get_attachments))
    .route("/attachments", post(routes::attachments::create_attachment))
    .route(
      "/attachments/{file_id}",
      patch(routes::attachments::update_attachment),
    )
    .route(
      "/attachments/{file_id}",
      delete(routes::attachments::delete_attachment),
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
