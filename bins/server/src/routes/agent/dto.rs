use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::models::{Agent, AgentAsset, AgentOs};

#[derive(Deserialize)]
pub struct GenerateAgentRequest {
  pub os: Vec<AgentOs>,
}

#[derive(Serialize)]
pub struct GenerateAgentPayload {
  pub agent: uuid::Uuid,
  pub organization: uuid::Uuid,
  pub version: i32,
  pub os: Vec<AgentOs>,
}

#[derive(Serialize)]
pub struct Agents {
  pub agent: Agent,
  pub assets: HashMap<i32, AgentAsset>,
}
