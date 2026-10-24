use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
};

use crate::error::{ErrorResponse, SharedError};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AgentError {
  #[error("Agent version allocation conflict. Please retry your build request.")]
  VersionConflict,
  #[error("Failed to allocate agent build version")]
  CreateFailed,
  #[error("Failed to serialize agent payload")]
  PayloadSerializationFailed,
  #[error("Failed to enqueue agent build job")]
  QueueFailed,
  #[error("Failed to retrieve agents")]
  FetchFailed,
  #[error("Failed to retrieve agent assets")]
  AssetsFetchFailed,
  #[error(transparent)]
  Shared(#[from] SharedError),
}

impl AgentError {
  pub fn status(&self) -> StatusCode {
    match self {
      Self::VersionConflict => StatusCode::CONFLICT,
      Self::CreateFailed
      | Self::PayloadSerializationFailed
      | Self::QueueFailed
      | Self::FetchFailed
      | Self::AssetsFetchFailed => StatusCode::INTERNAL_SERVER_ERROR,
      Self::Shared(shared) => shared.status(),
    }
  }

  pub fn code(&self) -> &'static str {
    match self {
      Self::QueueFailed => "AGENT_QUEUE_ERROR",
      Self::FetchFailed => "AGENT_FETCH_ERROR",
      Self::CreateFailed => "AGENT_CREATE_ERROR",
      Self::VersionConflict => "AGENT_VERSION_CONFLICT",
      Self::AssetsFetchFailed => "AGENT_ASSETS_FETCH_ERROR",
      Self::PayloadSerializationFailed => "AGENT_PAYLOAD_ERROR",
      Self::Shared(shared) => shared.code(),
    }
  }
}

impl IntoResponse for AgentError {
  fn into_response(self) -> Response {
    let status = self.status();
    let code = self.code();

    ErrorResponse::new(code, status, self.to_string()).into_response(status)
  }
}
