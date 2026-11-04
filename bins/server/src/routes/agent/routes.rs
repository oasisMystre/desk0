use axum::{Router, routing::get};

use super::handlers;
use crate::states::AppState;

pub fn router() -> Router<AppState> {
  Router::new()
    .route("/", get(handlers::get_agents).post(handlers::build_agent))
    .route("/status", get(handlers::get_build_agent_status))
}
