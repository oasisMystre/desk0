use axum::{
  Json,
  body::Body,
  http::StatusCode,
  response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub enum SharedErrorCode {
  #[serde(rename = "DATABASE_CONNECTION_ERROR")]
  DatabaseConnectionError,
  #[serde(rename = "DATABASE_OPERATION_ERROR")]
  DatabaseOperationError,
  #[serde(rename = "REDIS_CONNECTION_ERROR")]
  RedisConnectionError,
  #[serde(rename = "REDIS_OPERATION_ERROR")]
  RedisOperationError,
  #[serde(rename = "INTERNAL_ERROR")]
  InternalError,
  #[serde(rename = "BAD_REQUEST")]
  BadRequest,
  #[serde(rename = "UNAUTHORIZED")]
  Unauthorized,
  #[serde(rename = "FORBIDDEN")]
  Forbidden,
  #[serde(rename = "NOT_FOUND")]
  NotFound,
  #[serde(rename = "MISSING_AUTH_HEADER")]
  MissingAuthHeader,
  #[serde(rename = "INVALID_AUTH_HEADER")]
  InvalidAuthHeader,
  #[serde(rename = "INVALID_TOKEN")]
  InvalidToken,
  #[serde(rename = "MISSING_ROLE_CLAIM")]
  MissingRoleClaim,
  #[serde(rename = "MISSING_ORG_CLAIM")]
  MissingOrgClaim,
  #[serde(rename = "MISSING_USER_CLAIM")]
  MissingUserClaim,
  #[serde(rename = "MISSING_SUBJECT_CLAIM")]
  MissingSubjectClaim,
  #[serde(rename = "INVALID_CLAIM_ID")]
  InvalidClaimId,
  #[serde(rename = "INVALID_ORG_HEADER")]
  InvalidOrgHeader,
  #[serde(rename = "ORG_MEMBERSHIP_REQUIRED")]
  OrgMembershipRequired,
  #[serde(rename = "ORG_MEMBERSHIP_FORBIDDEN")]
  OrgMembershipForbidden,
  #[serde(rename = "UNSUPPORTED_ROLE")]
  UnsupportedRole,
  #[serde(rename = "WS_AUTH_FAILED")]
  WsAuthFailed,
  #[serde(rename = "WS_INVALID_ROLE")]
  WsInvalidRole,
  #[serde(rename = "ENDPOINT_NOT_FOUND")]
  EndpointNotFound,
}

impl SharedErrorCode {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::DatabaseConnectionError => "DATABASE_CONNECTION_ERROR",
      Self::DatabaseOperationError => "DATABASE_OPERATION_ERROR",
      Self::RedisConnectionError => "REDIS_CONNECTION_ERROR",
      Self::RedisOperationError => "REDIS_OPERATION_ERROR",
      Self::InternalError => "INTERNAL_ERROR",
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

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SharedErrorResponse {
  pub code: SharedErrorCode,
  pub message: String,
  pub error: Option<String>,
}

impl SharedErrorResponse {
  pub fn new(code: SharedErrorCode, status: StatusCode, message: impl Into<String>) -> Self {
    Self {
      code,
      message: message.into(),
      error: status.canonical_reason().map(|value| value.to_string()),
    }
  }

  pub fn into_response(self, status: StatusCode) -> Response {
    (status, Json(self)).into_response()
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
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

  pub fn into_body(self) -> Body {
    Body::from(serde_json::to_vec(&self).expect("Failed to serialize ErrorResponse"))
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

  pub fn error_code(&self) -> SharedErrorCode {
    match self {
      Self::DatabaseConnection => SharedErrorCode::DatabaseConnectionError,
      Self::DatabaseOperation => SharedErrorCode::DatabaseOperationError,
      Self::RedisConnection => SharedErrorCode::RedisConnectionError,
      Self::RedisOperation => SharedErrorCode::RedisOperationError,
      Self::Internal => SharedErrorCode::InternalError,
      Self::BadRequest => SharedErrorCode::BadRequest,
      Self::Unauthorized => SharedErrorCode::Unauthorized,
      Self::Forbidden => SharedErrorCode::Forbidden,
      Self::NotFound => SharedErrorCode::NotFound,
      Self::MissingAuthHeader => SharedErrorCode::MissingAuthHeader,
      Self::InvalidAuthHeader => SharedErrorCode::InvalidAuthHeader,
      Self::InvalidToken => SharedErrorCode::InvalidToken,
      Self::MissingRoleClaim => SharedErrorCode::MissingRoleClaim,
      Self::MissingOrgClaim => SharedErrorCode::MissingOrgClaim,
      Self::MissingUserClaim => SharedErrorCode::MissingUserClaim,
      Self::MissingSubjectClaim => SharedErrorCode::MissingSubjectClaim,
      Self::InvalidClaimId => SharedErrorCode::InvalidClaimId,
      Self::InvalidOrgHeader => SharedErrorCode::InvalidOrgHeader,
      Self::OrgMembershipRequired => SharedErrorCode::OrgMembershipRequired,
      Self::OrgMembershipForbidden => SharedErrorCode::OrgMembershipForbidden,
      Self::UnsupportedRole => SharedErrorCode::UnsupportedRole,
      Self::WsAuthFailed => SharedErrorCode::WsAuthFailed,
      Self::WsInvalidRole => SharedErrorCode::WsInvalidRole,
      Self::EndpointNotFound => SharedErrorCode::EndpointNotFound,
    }
  }

  pub fn code(&self) -> &'static str {
    self.error_code().as_str()
  }
}

impl IntoResponse for SharedError {
  fn into_response(self) -> Response {
    let status = self.status();
    let code = self.error_code();

    SharedErrorResponse::new(code, status, self.to_string()).into_response(status)
  }
}

pub type AppError = SharedError;
pub type AppResult<T> = Result<T, SharedError>;
