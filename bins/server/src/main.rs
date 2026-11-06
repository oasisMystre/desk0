use std::time::Duration;

use axum::{Json, Router, routing::get};
use clap::Parser;
use shared::init::{init_tracing, load_env};
use tokio::{net::TcpListener, time::sleep};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::states::AppState;

pub mod error;
pub mod openapi;
pub use error::{AppError, AppResult, SharedError};

mod context;
mod env;
mod routes;
mod states;

#[derive(Parser, Debug)]
#[command(name = "desk0-server", about = "Desk0 Relay Server & Device Broker")]
struct Args {
  #[arg(long, default_value = "0.0.0.0")]
  bind_addr: String,

  #[arg(long, default_value = "8080")]
  port: u16,

  #[arg(long)]
  export_openapi: Option<std::path::PathBuf>,
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
  load_env();
  init_tracing()?;

  let args = Args::parse();

  if let Some(path) = args.export_openapi {
    let spec = openapi::ApiDoc::openapi().to_pretty_json()?;
    if path.to_string_lossy() == "-" {
      println!("{spec}");
    } else {
      std::fs::write(&path, spec)?;
      println!("Exported OpenAPI spec to {}", path.display());
    }
    return Ok(());
  }

  let state = AppState::init().await?;

  let address = format!("{}:{}", args.bind_addr, args.port);
  let listener = TcpListener::bind(&address).await?;
  tracing::info!(%address, "server listening");

  let openapi_doc = openapi::ApiDoc::openapi();

  let app = Router::new()
    .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi_doc.clone()))
    .route(
      "/openapi.json",
      get({
        let openapi_doc = openapi_doc.clone();
        move || async move { Json(openapi_doc) }
      }),
    )
    .nest("/auth", routes::auth::routes::router())
    .merge(routes::users::routes::router())
    .nest("/organizations", routes::organizations::routes::router())
    .nest("/devices", routes::devices::routes::router())
    .nest("/agents", routes::agent::routes::router())
    .nest("/attachments", routes::attachments::routes::router())
    .nest("/applications", routes::applications::routes::router())
    .route("/ws", get(routes::ws::websocket::ws_route))
    .fallback(|| async { SharedError::EndpointNotFound })
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
