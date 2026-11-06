use axum::body::Bytes;

pub struct RawFileFromRequest {
  pub bytes: Bytes,
  filename: Option<String>,
  content_type: Option<String>,
}

impl RawFileFromRequest {
  pub fn new(bytes: Bytes, filename: Option<String>, content_type: Option<String>) -> Self {
    Self {
      bytes,
      filename,
      content_type,
    }
  }

  pub fn filename(&self) -> &str {
    self.filename.as_deref().unwrap_or("asset.bin")
  }

  pub fn content_type(&self) -> &str {
    self
      .content_type
      .as_deref()
      .unwrap_or("application/octet-stream")
  }
}
