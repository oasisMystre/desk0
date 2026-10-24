use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CreateDevice {
  pub pubkey: Vec<u8>,
  pub device: PartialDevice,
}

#[derive(Deserialize, Serialize)]
pub struct PartialDevice {
  pub organization: uuid::Uuid,
  pub os: Option<String>,
  pub hostname: Option<String>,
  pub username: Option<String>,
  pub version: Option<String>,
  pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConnectionTokenResponse {
  pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResponse {
  pub id: uuid::Uuid,
  pub challenge: String,
  pub expires_in: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateConnectionTokenRequest {
  pub challenge: String,
  pub signature: String,
}
