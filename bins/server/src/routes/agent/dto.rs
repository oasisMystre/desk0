use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::models::{Agent, AgentAsset, AgentOs};

#[derive(Deserialize, ToSchema)]
pub struct BuildAgentRequest {
  pub os: Vec<AgentOs>,
}

#[derive(Serialize, ToSchema)]
pub struct BuildAgentResponse {
  pub job: String,
  pub agent: Agent,
}

#[derive(Serialize, ToSchema)]
pub struct BuildAgentPayload {
  pub version: i32,
  pub os: Vec<AgentOs>,
  pub agent: uuid::Uuid,
  pub organization: uuid::Uuid,
}

#[derive(Deserialize, ToSchema)]
pub struct BuildAgentStatusRequest {
  pub job: String,
  pub asset: uuid::Uuid,
}

#[derive(Serialize, ToSchema)]
pub struct BuildAgentStatusResponse {
  pub error: Option<String>,
  #[schema(value_type = String, example = "completed")]
  pub state: bullmq::JobState,
  pub assets: Option<Vec<AgentAsset>>,
}

#[derive(Serialize, ToSchema)]
pub struct Agents {
  pub agent: Agent,
  #[schema(value_type = HashMap<String, Vec<AgentAsset>>)]
  pub assets: HashMap<i32, Vec<AgentAsset>>,
}
