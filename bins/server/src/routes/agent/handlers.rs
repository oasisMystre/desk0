use axum::{Json, extract::State};

use super::{
  dto::{Agents, GenerateAgentRequest},
  error::AgentError,
  models::Agent,
  service::AgentService,
};
use crate::states::{AppState, AuthenticatedUser};

pub async fn build_agent(
  State(AppState { queue, context, .. }): State<AppState>,
  auth: AuthenticatedUser,
  Json(request): Json<GenerateAgentRequest>,
) -> Result<Json<Agent>, AgentError> {
  let agent = AgentService::build_agent(&context, &queue, auth.org_id()?, request).await?;
  Ok(Json(agent))
}

pub async fn get_agents(
  State(AppState { context, .. }): State<AppState>,
  auth: AuthenticatedUser,
) -> Result<Json<Vec<Agents>>, AgentError> {
  let agents = AgentService::get_agents(&context, auth.org_id()?).await?;
  Ok(Json(agents))
}
