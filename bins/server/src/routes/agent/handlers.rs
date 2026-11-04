use axum::{Json, extract::State};

use super::{
  dto::{Agents, BuildAgentRequest, BuildAgentResponse},
  error::AgentError,
  service::AgentService,
};
use crate::{
  routes::agent::dto::{BuildAgentStatusRequest, BuildAgentStatusResponse},
  states::{AppState, AuthenticatedUser},
};

pub async fn build_agent(
  State(AppState { queue, context, .. }): State<AppState>,
  auth: AuthenticatedUser,
  Json(request): Json<BuildAgentRequest>,
) -> Result<Json<BuildAgentResponse>, AgentError> {
  let agent = AgentService::build_agent(&context, &queue, auth.org_id()?, request).await?;
  Ok(Json(agent))
}

pub async fn get_build_agent_status(
  State(AppState { queue, context, .. }): State<AppState>,
  auth: AuthenticatedUser,
  Json(request): Json<BuildAgentStatusRequest>,
) -> Result<Json<BuildAgentStatusResponse>, AgentError> {
  let status =
    AgentService::get_agent_build_status(&context, request, &queue, auth.org_id()?).await?;

  Ok(Json(status))
}

pub async fn get_agents(
  State(AppState { context, .. }): State<AppState>,
  auth: AuthenticatedUser,
) -> Result<Json<Vec<Agents>>, AgentError> {
  let agents = AgentService::get_agents(&context, auth.org_id()?).await?;
  Ok(Json(agents))
}
