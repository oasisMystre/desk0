use axum::{
  Router,
  routing::{get, patch},
};

use super::handlers;
use crate::states::AppState;

pub fn router() -> Router<AppState> {
  Router::new()
    .route(
      "/",
      get(handlers::get_attachments).post(handlers::create_attachment),
    )
    .route(
      "/{file_id}",
      patch(handlers::update_attachment).delete(handlers::delete_attachment),
    )
}
