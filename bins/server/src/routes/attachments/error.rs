use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
};

use crate::error::{ErrorResponse, SharedError};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AttachmentError {
  #[error("Attachment not found")]
  NotFound,
  #[error("Failed to retrieve attachments")]
  FetchFailed,
  #[error("Attachment storage service is misconfigured")]
  ConfigError,
  #[error("Failed to generate upload URL for attachment")]
  PresignFailed,
  #[error("Failed to save attachment record")]
  SaveFailed,
  #[error("Failed to update attachment")]
  UpdateFailed,
  #[error("Failed to delete attachment")]
  DeleteFailed,
  #[error(transparent)]
  Shared(#[from] SharedError),
}

impl AttachmentError {
  pub fn status(&self) -> StatusCode {
    match self {
      Self::NotFound => StatusCode::NOT_FOUND,
      Self::FetchFailed
      | Self::ConfigError
      | Self::PresignFailed
      | Self::SaveFailed
      | Self::UpdateFailed
      | Self::DeleteFailed => StatusCode::INTERNAL_SERVER_ERROR,
      Self::Shared(shared) => shared.status(),
    }
  }

  pub fn code(&self) -> &'static str {
    match self {
      Self::NotFound => "ATTACHMENT_NOT_FOUND",
      Self::SaveFailed => "ATTACHMENT_SAVE_ERROR",
      Self::FetchFailed => "ATTACHMENT_FETCH_ERROR",
      Self::ConfigError => "ATTACHMENT_CONFIG_ERROR",
      Self::UpdateFailed => "ATTACHMENT_UPDATE_ERROR",
      Self::DeleteFailed => "ATTACHMENT_DELETE_ERROR",
      Self::PresignFailed => "ATTACHMENT_PRESIGN_ERROR",
      Self::Shared(shared) => shared.code(),
    }
  }
}

impl IntoResponse for AttachmentError {
  fn into_response(self) -> Response {
    let status = self.status();
    let code = self.code();

    ErrorResponse::new(code, status, self.to_string()).into_response(status)
  }
}
