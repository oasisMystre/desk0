use std::time::Duration;

use axum::{
  Router,
  routing::{delete, get, patch, post},
};
use clap::Parser;
use shared::init::{init_tracing, load_env};
use tokio::{net::TcpListener, time::sleep};
use tower_http::trace::TraceLayer;

use crate::routes::state::AppState;

mod context;
mod env;
mod routes;

#[derive(Parser, Debug)]
#[command(name = "desk0-server", about = "Desk0 Relay Server & Device Broker")]
struct Args {
  #[arg(long, default_value = "0.0.0.0")]
  bind_addr: String,

  #[arg(long, default_value = "8080")]
  port: u16,
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
  load_env();
  init_tracing()?;

  let args = Args::parse();
  let state = AppState::init().await?;

  let address = format!("{}:{}", args.bind_addr, args.port);
  let listener = TcpListener::bind(&address).await?;
  tracing::info!(%address, "server listening");

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
      "/devices/token/{device_id}",
      post(routes::devices::create_connection_token),
    )
    .route(
      "/devices/{device_id}",
      patch(routes::devices::update_device),
    )
    .route(
      "/devices/{device_id}",
      delete(routes::devices::delete_device),
    )
    .route("/agents", post(routes::agent::build_agent))
    .route("/agents", get(routes::agent::get_agents))
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
    .route("/ws", get(routes::ws::websocket::ws_route))
    .layer(TraceLayer::new_for_http())
    .with_state(state.clone());

  let handle = tokio::spawn({
    let state = state.clone();

    async move {
      loop {
        match state.ws_context.hub.connect().await {
          Ok(mut handle) => {
            if let Err(error) = handle.run().await {
              tracing::warn!(?error, "ws hub connection closed, reconnecting");
            };
          }

          Err(error) => {
            tracing::warn!(?error, "ws hub connection failed, reconnecting");
          }
        }

        sleep(Duration::from_secs(1)).await;
      }
    }
  });

  tokio::select! {
    result = tokio::signal::ctrl_c() => result?,
    result = handle => result?,
    result = axum::serve(listener, app) => result?
  }

  Ok(())
}
