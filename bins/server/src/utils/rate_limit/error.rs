use axum::{
  http::{HeaderValue, StatusCode, header},
  response::{IntoResponse, Response},
};

use crate::{SharedError, error::ErrorResponse};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RateLimitError {
  #[error("Rate limiting unavailable")]
  Unavailable,
  #[error("Too many requests")]
  TooManyRequests(u64),
  #[error(transparent)]
  Shared(#[from] SharedError),
}

impl RateLimitError {
  pub fn status(&self) -> StatusCode {
    match self {
      RateLimitError::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
      RateLimitError::TooManyRequests(_) => StatusCode::TOO_MANY_REQUESTS,
      RateLimitError::Shared(shared) => shared.status(),
    }
  }

  pub fn code(&self) -> &'static str {
    match self {
      RateLimitError::Unavailable => "UNAVAILABLE",
      RateLimitError::TooManyRequests(_) => "TOO_MANY_REQUESTS",
      RateLimitError::Shared(shared) => shared.code(),
    }
  }
}

impl IntoResponse for RateLimitError {
  fn into_response(self) -> Response {
    let status = self.status();
    let code = self.code();

    match self {
      RateLimitError::TooManyRequests(retry) => Response::builder()
        .header(header::RETRY_AFTER, HeaderValue::from(retry))
        .body(ErrorResponse::new(code, status, self.to_string()).into_body())
        .unwrap(),
      _ => ErrorResponse::new(code, status, self.to_string()).into_response(status),
    }
  }
}
