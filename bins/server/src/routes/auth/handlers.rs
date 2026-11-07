use axum::{
  Json,
  extract::{Path, State},
  http::{HeaderMap, header},
};

use super::{
  dto::{
    EmailSignUpRequest, EmailSignUpResponse, RefreshTokenRequest, RefreshTokenResponse,
    RevokeSessionResponse, SessionResponse, VerifyOtpRequest, VerifyOtpResponse,
  },
  error::AuthError,
  service::AuthService,
};
use crate::{
  error::ErrorResponse,
  states::{AppState, AuthenticatedUser},
  utils::rate_limit::extract_ip,
};

#[utoipa::path(
  post,
  path = "/auth/email-signup",
  request_body = EmailSignUpRequest,
  responses(
    (status = 200, description = "Email signup OTP sent", body = EmailSignUpResponse),
    (status = 400, description = "Bad request", body = ErrorResponse),
    (status = 429, description = "Too many requests", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  tag = "auth"
)]
pub async fn email_signup(
  State(AppState { context, .. }): State<AppState>,
  Json(EmailSignUpRequest { email }): Json<EmailSignUpRequest>,
) -> Result<Json<EmailSignUpResponse>, AuthError> {
  let response = AuthService::email_signup(&context, email).await?;
  Ok(Json(response))
}

#[utoipa::path(
  post,
  path = "/auth/verify-otp",
  request_body = VerifyOtpRequest,
  responses(
    (status = 200, description = "OTP verified successfully", body = VerifyOtpResponse),
    (status = 400, description = "Invalid OTP or email", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  tag = "auth"
)]
pub async fn verify_otp(
  State(AppState { context, .. }): State<AppState>,
  headers: HeaderMap,
  Json(VerifyOtpRequest { otp, email }): Json<VerifyOtpRequest>,
) -> Result<Json<VerifyOtpResponse>, AuthError> {
  let user_agent = headers
    .get(header::USER_AGENT)
    .and_then(|value| value.to_str().ok())
    .map(|value| value.to_string());
  let ip_address = extract_ip(&headers);

  let response = AuthService::verify_otp(&context, email, otp, user_agent, ip_address).await?;
  Ok(Json(response))
}

#[utoipa::path(
  post,
  path = "/auth/refresh",
  request_body = RefreshTokenRequest,
  responses(
    (status = 200, description = "Token refreshed successfully", body = RefreshTokenResponse),
    (status = 401, description = "Invalid or expired refresh token", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  tag = "auth"
)]
pub async fn refresh_token(
  State(AppState { context, .. }): State<AppState>,
  Json(RefreshTokenRequest { refresh_token }): Json<RefreshTokenRequest>,
) -> Result<Json<RefreshTokenResponse>, AuthError> {
  let response = AuthService::refresh_token(&context, refresh_token).await?;
  Ok(Json(response))
}

#[utoipa::path(
  get,
  path = "/auth/sessions",
  responses(
    (status = 200, description = "List active sessions", body = Vec<SessionResponse>),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  security(
    ("bearer_auth" = [])
  ),
  tag = "auth"
)]
pub async fn get_sessions(
  State(AppState { context, .. }): State<AppState>,
  auth: AuthenticatedUser,
) -> Result<Json<Vec<SessionResponse>>, AuthError> {
  let sessions = AuthService::get_sessions(&context, auth.user, auth.session_id()).await?;
  Ok(Json(sessions))
}

#[utoipa::path(
  delete,
  path = "/auth/sessions/{session_id}",
  params(
    ("session_id" = uuid::Uuid, Path, description = "Session UUID")
  ),
  responses(
    (status = 200, description = "Session revoked successfully", body = RevokeSessionResponse),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  security(
    ("bearer_auth" = [])
  ),
  tag = "auth"
)]
pub async fn revoke_session(
  State(AppState { context, .. }): State<AppState>,
  auth: AuthenticatedUser,
  Path(session_id): Path<uuid::Uuid>,
) -> Result<Json<RevokeSessionResponse>, AuthError> {
  let response = AuthService::revoke_session(&context, auth.user, session_id).await?;
  Ok(Json(response))
}

#[utoipa::path(
  post,
  path = "/auth/logout",
  responses(
    (status = 200, description = "Logged out successfully", body = RevokeSessionResponse),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  security(
    ("bearer_auth" = [])
  ),
  tag = "auth"
)]
pub async fn logout(
  State(AppState { context, .. }): State<AppState>,
  auth: AuthenticatedUser,
) -> Result<Json<RevokeSessionResponse>, AuthError> {
  let response = AuthService::logout(
    &context,
    auth.user,
    auth
      .session_id()
      .ok_or_else(|| AuthError::SessionNotFound)?,
  )
  .await?;
  Ok(Json(response))
}
