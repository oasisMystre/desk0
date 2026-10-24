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
use crate::states::{AppState, AuthenticatedUser};

fn extract_ip(headers: &HeaderMap) -> Option<ipnet::IpNet> {
  let ip = headers
    .get("x-forwarded-for")
    .and_then(|header| header.to_str().ok())
    .and_then(|value| value.split(',').next())
    .map(|value| value.trim())
    .or_else(|| {
      headers
        .get("x-real-ip")
        .and_then(|header| header.to_str().ok())
        .map(|value| value.trim())
    });

  ip.and_then(|value| value.parse::<std::net::IpAddr>().ok())
    .map(|ip| ip.into())
}

pub async fn email_signup(
  State(AppState { context, .. }): State<AppState>,
  Json(EmailSignUpRequest { email }): Json<EmailSignUpRequest>,
) -> Result<Json<EmailSignUpResponse>, AuthError> {
  let response = AuthService::email_signup(&context, email).await?;
  Ok(Json(response))
}

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

pub async fn refresh_token(
  State(AppState { context, .. }): State<AppState>,
  Json(RefreshTokenRequest { refresh_token }): Json<RefreshTokenRequest>,
) -> Result<Json<RefreshTokenResponse>, AuthError> {
  let response = AuthService::refresh_token(&context, refresh_token).await?;
  Ok(Json(response))
}

pub async fn get_sessions(
  State(AppState { context, .. }): State<AppState>,
  auth: AuthenticatedUser,
) -> Result<Json<Vec<SessionResponse>>, AuthError> {
  let sessions = AuthService::get_sessions(&context, auth.user, auth.session_id()).await?;
  Ok(Json(sessions))
}

pub async fn revoke_session(
  State(AppState { context, .. }): State<AppState>,
  auth: AuthenticatedUser,
  Path(session_id): Path<uuid::Uuid>,
) -> Result<Json<RevokeSessionResponse>, AuthError> {
  let response = AuthService::revoke_session(&context, auth.user, session_id).await?;
  Ok(Json(response))
}

pub async fn logout(
  State(AppState { context, .. }): State<AppState>,
  auth: AuthenticatedUser,
) -> Result<Json<RevokeSessionResponse>, AuthError> {
  let response = AuthService::logout(&context, auth.user, auth.session_id()).await?;
  Ok(Json(response))
}
