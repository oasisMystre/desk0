use std::collections::HashMap;

use bullmq::Queue;
use diesel::result::{DatabaseErrorKind, Error::DatabaseError};
use uuid::Uuid;

use super::{
  dto::{
    Agents, BuildAgentPayload, BuildAgentRequest, BuildAgentResponse, BuildAgentStatusRequest,
    BuildAgentStatusResponse,
  },
  error::AgentError,
  models::{Agent, AgentAsset},
  repository::AgentRepository,
};
use crate::context::Context;

pub struct AgentService;

impl AgentService {
  pub async fn build_agent(
    context: &Context,
    queue: &Queue,
    organization: uuid::Uuid,
    request: BuildAgentRequest,
  ) -> Result<BuildAgentResponse, AgentError> {
    let mut connection = context.get_pg().await?;

    let mut retries = 0;
    let agent = loop {
      let result = AgentRepository::allocate_and_create_agent(&mut connection, organization).await;

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

    let payload = BuildAgentPayload {
      agent: agent.id,
      os: request.os,
      version: agent.version,
      organization: agent.organization,
    };

    let data = serde_json::to_string(&payload).map_err(|error| {
      tracing::error!(?error, "failed to serialize payload");
      AgentError::PayloadSerializationFailed
    })?;

    let job = queue.add("build-agent", data).await.map_err(|error| {
      tracing::error!(?error, "failed to add payload to build-agent queue");
      AgentError::QueueFailed
    })?;

    Ok(BuildAgentResponse {
      agent,
      job: job.id().to_owned(),
    })
  }

  pub async fn get_agent_build_status(
    context: &Context,
    request: BuildAgentStatusRequest,
    queue: &Queue,
    organization: uuid::Uuid,
  ) -> Result<BuildAgentStatusResponse, AgentError> {
    let mut connection = context.get_pg().await?;

    let state = queue.get_job_state(&request.job).await.map_err(|error| {
      tracing::error!(?error, "failed to fetch build asset job state");
      AgentError::FetchQueueFailed
    })?;

    Ok(match state {
      bullmq::JobState::Completed => {
        let assets = AgentRepository::find_agent_assets_by_agent_and_organization(
          &mut connection,
          request.asset,
          organization,
        )
        .await
        .map_err(|error| {
          tracing::error!(?error, "failed to fetch agent assets");
          AgentError::AssetFetchFailed
        })?;

        BuildAgentStatusResponse {
          state,
          error: None,
          assets: Some(assets),
        }
      }
      bullmq::JobState::Failed => {
        let job = queue.get_job(&request.job).await.map_err(|error| {
          tracing::error!(?error, "failed to fetch build asset job");
          AgentError::FetchQueueFailed
        })?;

        let error = job.map(|job| job.failed_reason().to_owned());

        BuildAgentStatusResponse {
          state,
          error,
          assets: None,
        }
      }
      state => BuildAgentStatusResponse {
        state,
        error: None,
        assets: None,
      },
    })
  }

  pub async fn get_agents(
    context: &Context,
    organization: uuid::Uuid,
  ) -> Result<Vec<Agents>, AgentError> {
    let mut connection = context.get_pg().await?;

    let agents = AgentRepository::find_agents_by_organization(&mut connection, organization)
      .await
      .map_err(|error| {
        tracing::error!(?error, "failed to fetch agents");
        AgentError::FetchFailed
      })?;

    let mut grouped: HashMap<Uuid, (Agent, HashMap<i32, Vec<AgentAsset>>)> = HashMap::new();

    for (agent, asset) in agents {
      let (_, versions) = grouped
        .entry(agent.id)
        .or_insert_with(|| (agent, HashMap::new()));

      versions
        .entry(asset.version)
        .or_insert_with(Vec::new)
        .push(asset);
    }

    Ok(
      grouped
        .into_values()
        .map(|(agent, assets)| {
          return Agents { agent, assets };
        })
        .collect(),
    )
  }
}
