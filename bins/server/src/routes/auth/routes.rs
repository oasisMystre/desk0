use axum::{
  Router,
  routing::{delete, get, post},
};

use super::handlers;
use crate::states::AppState;

pub fn router() -> Router<AppState> {
  Router::new()
    .route("/email-signup", post(handlers::email_signup))
    .route("/verify-otp", post(handlers::verify_otp))
    .route("/refresh", post(handlers::refresh_token))
    .route("/logout", post(handlers::logout))
    .route("/sessions", get(handlers::get_sessions))
    .route("/sessions/{session_id}", delete(handlers::revoke_session))
}
