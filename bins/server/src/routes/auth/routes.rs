use axum::{
  Router,
  routing::{delete, get, post},
};

use super::handlers;
use crate::{states::AppState, utils::rate_limit::create_governor_rate_limit};

pub fn router() -> Router<AppState> {
  Router::new()
    .route(
      "/email-signup",
      post(handlers::email_signup).layer(create_governor_rate_limit(1, 2)),
    )
    .route(
      "/verify-otp",
      post(handlers::verify_otp).layer(create_governor_rate_limit(1, 3)),
    )
    .route("/refresh", post(handlers::refresh_token))
    .route("/logout", post(handlers::logout))
    .route("/sessions", get(handlers::get_sessions))
    .route("/sessions/{session_id}", delete(handlers::revoke_session))
}
