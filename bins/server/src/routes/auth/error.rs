use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
};

use crate::error::{ErrorResponse, SharedError};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AuthError {
  #[error("Invalid or expired OTP code")]
  InvalidOtp,
  #[error("Failed to send verification email")]
  OtpSendFailed,
  #[error("Session has been revoked")]
  SessionRevoked,
  #[error("Session not found")]
  SessionNotFound,
  #[error("Failed to sign authorization token")]
  TokenSignFailed,
  #[error("Failed to retrieve or create user account")]
  UserCreateFailed,
  #[error("Failed to retrieve or create organization for user")]
  OrganizationCreateFailed,
  #[error("Failed to store OTP in cache")]
  OtpStorageFailed,
  #[error("Too many failed OTP attempts. Please request a new code.")]
  TooManyOtpAttempts,
  #[error("Invalid, expired, or revoked refresh token")]
  InvalidRefreshToken,
  #[error("Failed to create user session")]
  SessionCreateFailed,
  #[error("Failed to retrieve user sessions")]
  SessionFetchFailed,
  #[error("Failed to revoke user session")]
  SessionRevokeFailed,
  #[error(transparent)]
  Shared(#[from] SharedError),
}

impl AuthError {
  pub fn status(&self) -> StatusCode {
    match self {
      Self::SessionNotFound => StatusCode::NOT_FOUND,
      Self::TooManyOtpAttempts => StatusCode::TOO_MANY_REQUESTS,
      Self::InvalidOtp | Self::InvalidRefreshToken | Self::SessionRevoked => {
        StatusCode::UNAUTHORIZED
      }
      Self::OtpStorageFailed
      | Self::OtpSendFailed
      | Self::TokenSignFailed
      | Self::UserCreateFailed
      | Self::OrganizationCreateFailed
      | Self::SessionCreateFailed
      | Self::SessionFetchFailed
      | Self::SessionRevokeFailed => StatusCode::INTERNAL_SERVER_ERROR,

      Self::Shared(shared) => shared.status(),
    }
  }

  pub fn code(&self) -> &'static str {
    match self {
      Self::InvalidOtp => "INVALID_OTP",
      Self::OtpSendFailed => "OTP_SEND_ERROR",
      Self::SessionRevoked => "SESSION_REVOKED",
      Self::TokenSignFailed => "TOKEN_SIGN_ERROR",
      Self::SessionNotFound => "SESSION_NOT_FOUND",
      Self::OtpStorageFailed => "OTP_STORAGE_ERROR",
      Self::UserCreateFailed => "USER_CREATE_ERROR",
      Self::SessionFetchFailed => "SESSION_FETCH_ERROR",
      Self::InvalidRefreshToken => "INVALID_REFRESH_TOKEN",
      Self::TooManyOtpAttempts => "TOO_MANY_OTP_ATTEMPTS",
      Self::SessionCreateFailed => "SESSION_CREATE_ERROR",
      Self::SessionRevokeFailed => "SESSION_REVOKE_ERROR",
      Self::OrganizationCreateFailed => "ORGANIZATION_CREATE_FAILED",
      Self::Shared(shared) => shared.code(),
    }
  }
}

impl From<diesel::result::Error> for AuthError {
  fn from(value: diesel::result::Error) -> Self {
    match value {
      error => {
        tracing::error!(?error, "application database operation failed");
        SharedError::DatabaseOperation.into()
      }
    }
  }
}

impl IntoResponse for AuthError {
  fn into_response(self) -> Response {
    let status = self.status();
    let code = self.code();

    ErrorResponse::new(code, status, self.to_string()).into_response(status)
  }
}
