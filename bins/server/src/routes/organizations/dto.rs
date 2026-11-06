use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct CreateOrganizationRequest {
  pub name: Option<String>,
}

#[derive(Serialize, ToSchema)]
#[schema(as = OrganizationConnectionTokenResponse)]
pub struct CreateConnectionTokenResponse {
  pub token: String,
}
