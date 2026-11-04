use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::models::{Agent, AgentAsset, AgentOs};

#[derive(Deserialize)]
pub struct BuildAgentRequest {
  pub os: Vec<AgentOs>,
}

#[derive(Serialize)]
pub struct BuildAgentResponse {
  pub job: String,
  pub agent: Agent,
}

#[derive(Serialize)]
pub struct BuildAgentPayload {
  pub version: i32,
  pub os: Vec<AgentOs>,
  pub agent: uuid::Uuid,
  pub organization: uuid::Uuid,
}

#[derive(Deserialize)]
pub struct BuildAgentStatusRequest {
  pub job: String,
  pub asset: uuid::Uuid,
}

#[derive(Serialize)]
pub struct BuildAgentStatusResponse {
  pub error: Option<String>,
  pub state: bullmq::JobState,
  pub assets: Option<Vec<AgentAsset>>,
}

#[derive(Serialize)]
pub struct Agents {
  pub agent: Agent,
  pub assets: HashMap<i32, Vec<AgentAsset>>,
}
