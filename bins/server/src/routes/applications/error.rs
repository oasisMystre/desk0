use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
};

use crate::error::{ErrorResponse, SharedError};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ApplicationError {
  #[error("Parsing multipart failed")]
  ParseMultipart,
  #[allow(dead_code)]
  #[error("Invalid {0} file")]
  InvalidFile(&'static str),
  #[error("Missing required field: {0}")]
  MissingField(&'static str),
  #[error("Invalid field: {0}")]
  InvalidField(&'static str),
  #[error("Application not found")]
  NotFound,
  #[error("Device not found")]
  DeviceNotFound,
  #[error("Failed to retrieve applications")]
  FetchFailed,
  #[error("Failed to create application")]
  CreateFailed,
  #[error("Failed to delete application")]
  DeleteFailed,
  #[error("Failed to upload file")]
  UploadFailed,
  #[error("Storage configuration error")]
  ConfigError,
  #[error(transparent)]
  Shared(#[from] SharedError),
}

impl ApplicationError {
  pub fn status(&self) -> StatusCode {
    match self {
      Self::NotFound | Self::DeviceNotFound => StatusCode::NOT_FOUND,
      Self::MissingField(_)
      | Self::InvalidField(_)
      | Self::InvalidFile(_)
      | Self::ParseMultipart => StatusCode::BAD_REQUEST,
      Self::FetchFailed
      | Self::CreateFailed
      | Self::DeleteFailed
      | Self::UploadFailed
      | Self::ConfigError => StatusCode::INTERNAL_SERVER_ERROR,
      Self::Shared(shared) => shared.status(),
    }
  }

  pub fn code(&self) -> &'static str {
    match self {
      Self::NotFound => "APPLICATION_NOT_FOUND",
      Self::DeviceNotFound => "DEVICE_NOT_FOUND",
      Self::ParseMultipart => "APPLICATION_PARSE_MULTIPART_ERROR",
      Self::InvalidFile(_) => "APPLICATION_INVALID_FILE",
      Self::MissingField(_) => "APPLICATION_MISSING_FIELD",
      Self::InvalidField(_) => "APPLICATION_INVALID_FIELD",
      Self::FetchFailed => "APPLICATION_FETCH_ERROR",
      Self::CreateFailed => "APPLICATION_CREATE_ERROR",
      Self::DeleteFailed => "APPLICATION_DELETE_ERROR",
      Self::UploadFailed => "APPLICATION_UPLOAD_ERROR",
      Self::ConfigError => "APPLICATION_CONFIG_ERROR",
      Self::Shared(shared) => shared.code(),
    }
  }
}

impl From<diesel::result::Error> for ApplicationError {
  fn from(value: diesel::result::Error) -> Self {
    match value {
      diesel::result::Error::NotFound => Self::NotFound,
      error => {
        tracing::error!(?error, "application database operation failed");
        SharedError::DatabaseOperation.into()
      }
    }
  }
}

impl IntoResponse for ApplicationError {
  fn into_response(self) -> Response {
    let status = self.status();
    let code = self.code();

    ErrorResponse::new(code, status, self.to_string()).into_response(status)
  }
}
