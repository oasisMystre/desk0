use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::models::{AgentOs, Attachment, Scripts};

#[derive(Deserialize, Serialize, Clone, Debug, ToSchema)]
pub struct AddDeviceApplicationRequest {
  pub application: uuid::Uuid,
}

#[derive(Debug)]
pub struct CreateApplicationRequest {
  pub os: AgentOs,
  pub name: String,
  pub scripts: Scripts,
  pub asset: Attachment,
  pub image: Option<Attachment>,
}
