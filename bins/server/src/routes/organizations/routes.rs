use axum::{
  Router,
  routing::{get, post},
};

use super::handlers;
use crate::states::AppState;

pub fn router() -> Router<AppState> {
  Router::new()
    .route(
      "/",
      get(handlers::get_organizations).post(handlers::create_organization),
    )
    .route(
      "/token/{device_id}",
      post(handlers::create_connection_token),
    )
}
