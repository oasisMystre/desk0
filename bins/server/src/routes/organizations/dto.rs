use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateOrganizationRequest {
  pub name: Option<String>,
}

#[derive(Serialize)]
pub struct CreateConnectionTokenResponse {
  pub token: String,
}
