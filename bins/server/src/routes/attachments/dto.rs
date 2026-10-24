use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateUploadRequest {
  pub filename: String,
  pub content_type: String,
  pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct CreateUploadResponse {
  pub upload_url: String,
  pub file_id: uuid::Uuid,
  pub public_url: String,
}

#[derive(Deserialize)]
pub struct UpdateAttachmentRequest {
  pub uploaded: Option<bool>,
  pub filename: Option<String>,
  pub metadata: Option<serde_json::Value>,
}
