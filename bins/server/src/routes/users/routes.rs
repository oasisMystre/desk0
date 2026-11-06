use axum::{Router, routing::delete};

use super::handlers;
use crate::states::AppState;

pub fn router() -> Router<AppState> {
  Router::new().route("/user", delete(handlers::delete_user))
}
