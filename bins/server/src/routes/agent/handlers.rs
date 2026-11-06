use axum::{Json, extract::State};

use super::{
  dto::{Agents, BuildAgentRequest, BuildAgentResponse},
  error::{AgentError},
  service::AgentService,
};
use crate::{
  error::ErrorResponse, routes::agent::dto::{BuildAgentStatusRequest, BuildAgentStatusResponse}, states::{AppState, AuthenticatedUser},
};

#[utoipa::path(
  post,
  path = "/agents",
  request_body = BuildAgentRequest,
  responses(
    (status = 200, description = "Agent build job queued", body = BuildAgentResponse),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 409, description = "Agent version conflict", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  security(
    ("bearer_auth" = [])
  ),
  tag = "agents"
)]
pub async fn build_agent(
  State(AppState { queue, context, .. }): State<AppState>,
  auth: AuthenticatedUser,
  Json(request): Json<BuildAgentRequest>,
) -> Result<Json<BuildAgentResponse>, AgentError> {
  let agent = AgentService::build_agent(&context, &queue, auth.org_id()?, request).await?;
  Ok(Json(agent))
}

#[utoipa::path(
  get,
  path = "/agents/status",
  request_body = BuildAgentStatusRequest,
  responses(
    (status = 200, description = "Agent build status", body = BuildAgentStatusResponse),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  security(
    ("bearer_auth" = [])
  ),
  tag = "agents"
)]
pub async fn get_build_agent_status(
  State(AppState { queue, context, .. }): State<AppState>,
  auth: AuthenticatedUser,
  Json(request): Json<BuildAgentStatusRequest>,
) -> Result<Json<BuildAgentStatusResponse>, AgentError> {
  let status =
    AgentService::get_agent_build_status(&context, request, &queue, auth.org_id()?).await?;

  Ok(Json(status))
}

#[utoipa::path(
  get,
  path = "/agents",
  responses(
    (status = 200, description = "List built agents", body = Vec<Agents>),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  security(
    ("bearer_auth" = [])
  ),
  tag = "agents"
)]
pub async fn get_agents(
  State(AppState { context, .. }): State<AppState>,
  auth: AuthenticatedUser,
) -> Result<Json<Vec<Agents>>, AgentError> {
  let agents = AgentService::get_agents(&context, auth.org_id()?).await?;
  Ok(Json(agents))
}
