use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
};

use crate::error::{ErrorResponse, SharedError};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OrganizationError {
  #[allow(dead_code)]
  #[error("Organization not found")]
  NotFound,
  #[error("Failed to retrieve organizations")]
  FetchFailed,
  #[error("Failed to create organization")]
  CreateFailed,
  #[error("Failed to sign connection token")]
  TokenSignFailed,
  #[error(transparent)]
  Shared(#[from] SharedError),
}

impl OrganizationError {
  pub fn status(&self) -> StatusCode {
    match self {
      Self::NotFound => StatusCode::NOT_FOUND,
      Self::FetchFailed | Self::CreateFailed | Self::TokenSignFailed => {
        StatusCode::INTERNAL_SERVER_ERROR
      }
      Self::Shared(shared) => shared.status(),
    }
  }

  pub fn code(&self) -> &'static str {
    match self {
      Self::NotFound => "ORG_NOT_FOUND",
      Self::FetchFailed => "ORG_FETCH_ERROR",
      Self::CreateFailed => "ORG_CREATE_ERROR",
      Self::TokenSignFailed => "TOKEN_SIGN_ERROR",
      Self::Shared(shared) => shared.code(),
    }
  }
}

impl IntoResponse for OrganizationError {
  fn into_response(self) -> Response {
    let status = self.status();
    let code = self.code();

    ErrorResponse::new(code, status, self.to_string()).into_response(status)
  }
}
