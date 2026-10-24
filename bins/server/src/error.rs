use axum::{
  Json,
  http::StatusCode,
  response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
  pub code: String,
  pub message: String,
  pub error: Option<String>,
}

impl ErrorResponse {
  pub fn new(code: impl Into<String>, status: StatusCode, message: impl Into<String>) -> Self {
    Self {
      code: code.into(),
      message: message.into(),
      error: status.canonical_reason().map(|value| value.to_string()),
    }
  }

  pub fn into_response(self, status: StatusCode) -> Response {
    (status, Json(self)).into_response()
  }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SharedError {
  #[error("Failed to connect to database")]
  DatabaseConnection,
  #[error("Database query execution failed")]
  DatabaseOperation,
  #[error("Failed to connect to Redis cache")]
  RedisConnection,
  #[error("Redis cache operation failed")]
  RedisOperation,
  #[error("An internal server error occurred")]
  Internal,
  #[error("Bad request")]
  BadRequest,
  #[error("Unauthorized")]
  Unauthorized,
  #[error("Forbidden")]
  Forbidden,
  #[error("Resource not found")]
  NotFound,
  #[error("Missing authorization header")]
  MissingAuthHeader,
  #[error("Invalid authorization header format. Expected 'Bearer <token>'")]
  InvalidAuthHeader,
  #[error("Invalid or expired authorization token")]
  InvalidToken,
  #[error("Token is missing role claim")]
  MissingRoleClaim,
  #[error("Token is missing organization claim")]
  MissingOrgClaim,
  #[error("Token is missing user claim")]
  MissingUserClaim,
  #[error("Token is missing subject claim")]
  MissingSubjectClaim,
  #[error("Invalid ID in token claims")]
  InvalidClaimId,
  #[error("Invalid x-org-id header format")]
  InvalidOrgHeader,
  #[error("Organization ID header (x-org-id) or claim is required")]
  OrgMembershipRequired,
  #[error("User is not a member of the specified organization")]
  OrgMembershipForbidden,
  #[error("Unsupported or unknown role in token")]
  UnsupportedRole,
  #[error("WebSocket authentication failed")]
  WsAuthFailed,
  #[error("Invalid role for WebSocket connection")]
  WsInvalidRole,
  #[error("Endpoint not found")]
  EndpointNotFound,
}

impl SharedError {
  pub fn status(&self) -> StatusCode {
    match self {
      Self::MissingAuthHeader
      | Self::InvalidToken
      | Self::MissingRoleClaim
      | Self::MissingOrgClaim
      | Self::MissingUserClaim
      | Self::MissingSubjectClaim
      | Self::UnsupportedRole
      | Self::OrgMembershipRequired
      | Self::WsAuthFailed
      | Self::WsInvalidRole
      | Self::Unauthorized => StatusCode::UNAUTHORIZED,

      Self::InvalidAuthHeader
      | Self::InvalidClaimId
      | Self::InvalidOrgHeader
      | Self::BadRequest => StatusCode::BAD_REQUEST,

      Self::OrgMembershipForbidden | Self::Forbidden => StatusCode::FORBIDDEN,

      Self::EndpointNotFound | Self::NotFound => StatusCode::NOT_FOUND,

      Self::DatabaseConnection
      | Self::DatabaseOperation
      | Self::RedisConnection
      | Self::RedisOperation
      | Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
    }
  }

  pub fn code(&self) -> &'static str {
    match self {
      Self::DatabaseConnection => "DATABASE_CONNECTION_ERROR",
      Self::DatabaseOperation => "DATABASE_OPERATION_ERROR",
      Self::RedisConnection => "REDIS_CONNECTION_ERROR",
      Self::RedisOperation => "REDIS_OPERATION_ERROR",
      Self::Internal => "INTERNAL_ERROR",
      Self::BadRequest => "BAD_REQUEST",
      Self::Unauthorized => "UNAUTHORIZED",
      Self::Forbidden => "FORBIDDEN",
      Self::NotFound => "NOT_FOUND",
      Self::MissingAuthHeader => "MISSING_AUTH_HEADER",
      Self::InvalidAuthHeader => "INVALID_AUTH_HEADER",
      Self::InvalidToken => "INVALID_TOKEN",
      Self::MissingRoleClaim => "MISSING_ROLE_CLAIM",
      Self::MissingOrgClaim => "MISSING_ORG_CLAIM",
      Self::MissingUserClaim => "MISSING_USER_CLAIM",
      Self::MissingSubjectClaim => "MISSING_SUBJECT_CLAIM",
      Self::InvalidClaimId => "INVALID_CLAIM_ID",
      Self::InvalidOrgHeader => "INVALID_ORG_HEADER",
      Self::OrgMembershipRequired => "ORG_MEMBERSHIP_REQUIRED",
      Self::OrgMembershipForbidden => "ORG_MEMBERSHIP_FORBIDDEN",
      Self::UnsupportedRole => "UNSUPPORTED_ROLE",
      Self::WsAuthFailed => "WS_AUTH_FAILED",
      Self::WsInvalidRole => "WS_INVALID_ROLE",
      Self::EndpointNotFound => "ENDPOINT_NOT_FOUND",
    }
  }
}

impl IntoResponse for SharedError {
  fn into_response(self) -> Response {
    let status = self.status();
    let code = self.code();

    ErrorResponse::new(code, status, self.to_string()).into_response(status)
  }
}

pub type AppError = SharedError;
pub type AppResult<T> = Result<T, SharedError>;
