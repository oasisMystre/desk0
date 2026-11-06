use axum::{
  Router,
  routing::{delete, get, patch, post},
};

use super::handlers;
use crate::{routes::applications::handlers as application_handlers, states::AppState};

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
    .route(
      "/{device}/applications",
      get(application_handlers::get_device_applications)
        .post(application_handlers::add_device_application),
    )
    .route(
      "/{device}/applications/{application}",
      delete(application_handlers::remove_device_application),
    )
}
