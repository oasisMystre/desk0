use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::models::User;

#[derive(Deserialize, ToSchema)]
pub struct EmailSignUpRequest {
  pub email: String,
}

#[derive(Serialize, ToSchema)]
pub struct EmailSignUpResponse {
  pub message: String,
}

#[derive(Deserialize, ToSchema)]
pub struct VerifyOtpRequest {
  pub otp: String,
  pub email: String,
}

#[derive(Serialize, ToSchema)]
pub struct VerifyOtpResponse {
  pub user: User,
  pub token: String,
  pub refresh_token: String,
  pub session_id: uuid::Uuid,
}

#[derive(Deserialize, ToSchema)]
pub struct RefreshTokenRequest {
  pub refresh_token: String,
}

#[derive(Serialize, ToSchema)]
pub struct RefreshTokenResponse {
  pub token: String,
  pub refresh_token: String,
}

#[derive(Serialize, ToSchema)]
pub struct SessionResponse {
  pub id: uuid::Uuid,
  pub user_agent: Option<String>,
  pub ip_address: Option<String>,
  pub created_at: DateTime<Utc>,
  pub expires_at: DateTime<Utc>,
  pub last_used_at: Option<DateTime<Utc>>,
  pub is_current: bool,
}

#[derive(Serialize, ToSchema)]
pub struct RevokeSessionResponse {
  pub message: String,
}
