use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct CreateDevice {
  #[schema(value_type = Vec<u8>)]
  pub pubkey: Vec<u8>,
  pub device: PartialDevice,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct PartialDevice {
  pub organization: uuid::Uuid,
  pub os: Option<String>,
  pub hostname: Option<String>,
  pub username: Option<String>,
  pub version: Option<String>,
  pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = DeviceConnectionTokenResponse)]
pub struct CreateConnectionTokenResponse {
  pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChallengeResponse {
  pub id: uuid::Uuid,
  pub challenge: String,
  pub expires_in: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CreateConnectionTokenRequest {
  pub challenge: String,
  pub signature: String,
}
