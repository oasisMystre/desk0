use axum::{
  Router,
  routing::{delete, get},
};

use super::handlers;
use crate::states::AppState;

pub fn router() -> Router<AppState> {
  Router::new()
    .route(
      "/",
      get(handlers::get_applications).post(handlers::create_application),
    )
    .route(
      "/{id}",
      get(handlers::get_application).delete(handlers::delete_application),
    )
    .route(
      "/device/{device}",
      get(handlers::get_device_applications).post(handlers::add_device_application),
    )
    .route(
      "/device/{device}/{application}",
      delete(handlers::remove_device_application),
    )
}
