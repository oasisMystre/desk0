use serde::Serialize;

#[derive(Serialize)]
pub struct CreateConnectionTokenResponse {
  pub token: String,
}
