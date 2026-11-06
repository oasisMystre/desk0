use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
};

use crate::error::{ErrorResponse, SharedError};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DeviceError {
  #[error("Device not found")]
  NotFound,
  #[error("Failed to retrieve devices")]
  FetchFailed,
  #[error("Failed to create device")]
  CreateFailed,
  #[error("Failed to update device")]
  UpdateFailed,
  #[error("Failed to delete device")]
  DeleteFailed,
  #[error("Failed to sign connection token")]
  TokenSignFailed,
  #[error("Invalid or expired challenge")]
  InvalidChallenge,
  #[error("Invalid challenge signature")]
  InvalidSignature,
  #[error("Invalid device public key")]
  InvalidPublicKey,
  #[error("Failed to store challenge")]
  ChallengeStorageFailed,
  #[error(transparent)]
  Shared(#[from] SharedError),
}

impl DeviceError {
  pub fn status(&self) -> StatusCode {
    match self {
      Self::NotFound => StatusCode::NOT_FOUND,
      Self::InvalidChallenge | Self::InvalidSignature => StatusCode::UNAUTHORIZED,
      Self::InvalidPublicKey => StatusCode::BAD_REQUEST,
      Self::FetchFailed
      | Self::CreateFailed
      | Self::UpdateFailed
      | Self::DeleteFailed
      | Self::TokenSignFailed
      | Self::ChallengeStorageFailed => StatusCode::INTERNAL_SERVER_ERROR,
      Self::Shared(shared) => shared.status(),
    }
  }

  pub fn code(&self) -> &'static str {
    match self {
      Self::NotFound => "DEVICE_NOT_FOUND",
      Self::InvalidChallenge => "INVALID_CHALLENGE",
      Self::InvalidSignature => "INVALID_SIGNATURE",
      Self::InvalidPublicKey => "INVALID_PUBLIC_KEY",
      Self::ChallengeStorageFailed => "CHALLENGE_STORAGE_ERROR",
      Self::FetchFailed => "DEVICE_FETCH_ERROR",
      Self::CreateFailed => "DEVICE_CREATE_ERROR",
      Self::UpdateFailed => "DEVICE_UPDATE_ERROR",
      Self::DeleteFailed => "DEVICE_DELETE_ERROR",
      Self::TokenSignFailed => "TOKEN_SIGN_ERROR",
      Self::Shared(shared) => shared.code(),
    }
  }
}

impl IntoResponse for DeviceError {
  fn into_response(self) -> Response {
    let status = self.status();
    let code = self.code();

    ErrorResponse::new(code, status, self.to_string()).into_response(status)
  }
}
