use axum::{
  Router,
  routing::{get, patch, post},
};

use super::handlers;
use crate::states::AppState;

pub fn router() -> Router<AppState> {
  Router::new()
    .route(
      "/",
      get(handlers::get_devices).post(handlers::create_device),
    )
    .route(
      "/challenge/{device}",
      post(handlers::create_challenge).get(handlers::create_challenge),
    )
    .route("/token/{device}", post(handlers::create_connection_token))
    .route(
      "/{device}",
      patch(handlers::update_device).delete(handlers::delete_device),
    )
}
