use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
  Organization,
  Agent,
}

#[derive(Deserialize)]
pub struct CreateConnectionTokenQueryParams {
  pub role: Role,
}

#[derive(Serialize)]
pub struct CreateConnectionTokenResponse {
  pub token: String,
}
