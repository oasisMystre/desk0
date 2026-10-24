use std::collections::HashMap;

use bullmq::Queue;
use diesel::result::{DatabaseErrorKind, Error::DatabaseError};

use super::{
  dto::{Agents, GenerateAgentPayload, GenerateAgentRequest},
  error::AgentError,
  models::Agent,
  repository::AgentRepository,
};
use crate::context::Context;

pub struct AgentService;

impl AgentService {
  pub async fn build_agent(
    context: &Context,
    queue: &Queue,
    organization: uuid::Uuid,
    request: GenerateAgentRequest,
  ) -> Result<Agent, AgentError> {
    let mut connection = context.get_pg().await?;

    let mut retries = 0;
    let agent = loop {
      let result =
        AgentRepository::allocate_and_create_agent(&mut connection, organization).await;

      match result {
        Ok(agent) => break agent,
        Err(DatabaseError(DatabaseErrorKind::UniqueViolation, info))
          if info.constraint_name() == Some("agents_organization_version_unique") =>
        {
          retries += 1;
          if retries >= 3 {
            tracing::error!("agent version allocation retries exhausted");
            return Err(AgentError::VersionConflict);
          }

          continue;
        }
        Err(error) => {
          tracing::error!(?error, "failed to create agent version");
          return Err(AgentError::CreateFailed);
        }
      }
    };

    let payload = GenerateAgentPayload {
      agent: agent.id,
      os: request.os,
      version: agent.version,
      organization: agent.organization,
    };

    let data = serde_json::to_string(&payload).map_err(|error| {
      tracing::error!(?error, "failed to serialize payload");
      AgentError::PayloadSerializationFailed
    })?;

    queue.add("build-agent", data).await.map_err(|error| {
      tracing::error!(?error, "failed to add payload to build-agent queue");
      AgentError::QueueFailed
    })?;

    Ok(agent)
  }

  pub async fn get_agents(
    context: &Context,
    organization: uuid::Uuid,
  ) -> Result<Vec<Agents>, AgentError> {
    let mut connection = context.get_pg().await?;

    let agent_rows =
      AgentRepository::find_agents_by_organization(&mut connection, organization)
        .await
        .map_err(|error| {
          tracing::error!(?error, "failed to fetch agents");
          AgentError::FetchFailed
        })?;

    let mut result = Vec::with_capacity(agent_rows.len());

    for agent in agent_rows {
      let asset_rows = AgentRepository::find_assets_by_agent(&mut connection, agent.id)
        .await
        .map_err(|error| {
          tracing::error!(?error, agent_id = %agent.id, "failed to fetch agent assets");
          AgentError::AssetsFetchFailed
        })?;

      let mut assets = HashMap::new();

      for asset in asset_rows {
        assert_eq!(
          asset.version, agent.version,
          "agent asset version must match its agent version"
        );

        assets.insert(asset.version, asset);
      }

      result.push(Agents { agent, assets });
    }

    Ok(result)
  }
}
