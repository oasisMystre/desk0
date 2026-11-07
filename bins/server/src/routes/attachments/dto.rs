use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct CreateUploadRequest {
  pub filename: String,
  pub content_type: String,
  pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize, ToSchema)]
pub struct CreateUploadResponse {
  pub id: uuid::Uuid,
  pub url: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateAttachmentRequest {
  pub uploaded: Option<bool>,
  pub filename: Option<String>,
  pub metadata: Option<serde_json::Value>,
}
