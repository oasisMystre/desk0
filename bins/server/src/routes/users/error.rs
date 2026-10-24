use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
};

use crate::error::{ErrorResponse, SharedError};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum UserError {
  #[error("User account not found")]
  NotFound,
  #[error("Failed to delete user account")]
  DeleteFailed,
  #[error(transparent)]
  Shared(#[from] SharedError),
}

impl UserError {
  pub fn status(&self) -> StatusCode {
    match self {
      Self::NotFound => StatusCode::NOT_FOUND,
      Self::DeleteFailed => StatusCode::INTERNAL_SERVER_ERROR,
      Self::Shared(shared) => shared.status(),
    }
  }

  pub fn code(&self) -> &'static str {
    match self {
      Self::NotFound => "USER_NOT_FOUND",
      Self::DeleteFailed => "USER_DELETE_ERROR",
      Self::Shared(shared) => shared.code(),
    }
  }
}

impl IntoResponse for UserError {
  fn into_response(self) -> Response {
    let status = self.status();
    let code = self.code();

    ErrorResponse::new(code, status, self.to_string()).into_response(status)
  }
}
