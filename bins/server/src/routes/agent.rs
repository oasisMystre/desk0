use std::collections::HashMap;

use axum::{Json, extract::State, http::StatusCode};
use diesel::{
  ExpressionMethods, QueryDsl, SelectableHelper,
  dsl::{insert_into, max},
  result::{DatabaseErrorKind, Error::DatabaseError},
};
use diesel_async::{AsyncConnection, RunQueryDsl};
use flow::{
  agent::{Agent, AgentAsset},
  agent_assets, agents, organizations,
  os::AgentOs,
};
use serde::{Deserialize, Serialize};

use crate::routes::{auth::AuthenticatedUser, state::AppState};

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
  agent: Agent,
  assets: HashMap<i32, AgentAsset>,
}

pub async fn build_agent(
  State(AppState { queue, context, .. }): State<AppState>,
  auth: AuthenticatedUser,
  Json(request): Json<GenerateAgentRequest>,
) -> Result<Json<Agent>, StatusCode> {
  let mut connection = context.pg.get().await.map_err(|error| {
    tracing::error!(?error, "database connection error");
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  let organization = auth.org_id()?;

  let mut retries = 0;
  let agent = loop {
    let result = connection
      .transaction(async |connection| {
        organizations::table
          .filter(organizations::id.eq(organization))
          .for_update()
          .execute(connection)
          .await?;

        let version = agents::table
          .filter(agents::organization.eq(organization))
          .select(max(agents::version))
          .first::<Option<i32>>(connection)
          .await?;
        let version = version.unwrap_or(0) + 1;

        insert_into(agents::table)
          .values(((
            agents::organization.eq(organization),
            agents::version.eq(version),
          ),))
          .get_result::<Agent>(connection)
          .await
      })
      .await;

    match result {
      Ok(agent) => break agent,
      Err(DatabaseError(DatabaseErrorKind::UniqueViolation, info))
        if info.constraint_name() == Some("agents_organization_version_unique") =>
      {
        retries += 1;
        if retries >= 3 {
          tracing::error!("agent version allocation retries exhausted");
          return Err(StatusCode::CONFLICT);
        }

        continue;
      }
      Err(error) => {
        tracing::error!(?error, "failed to create agent version");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
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
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  queue.add("build-agent", data).await.map_err(|error| {
    tracing::error!(?error, "failed to add payload to build-agent queue");
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  Ok(Json(agent))
}

pub async fn get_agents(
  State(AppState { context, .. }): State<AppState>,
  auth: AuthenticatedUser,
) -> Result<Json<Vec<Agents>>, StatusCode> {
  let mut connection = context.pg.get().await.map_err(|error| {
    tracing::error!(?error, "database connection error");
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  let agent_rows = agents::table
    .filter(agents::organization.eq(auth.org_id()?))
    .select(Agent::as_select())
    .load::<Agent>(&mut connection)
    .await
    .map_err(|error| {
      tracing::error!(?error, "failed to fetch agents");
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  let mut result = Vec::with_capacity(agent_rows.len());

  for agent in agent_rows {
    let asset_rows = agent_assets::table
      .filter(agent_assets::agent.eq(agent.id))
      .select(AgentAsset::as_select())
      .load::<AgentAsset>(&mut connection)
      .await
      .map_err(|error| {
        tracing::error!(?error, agent_id = %agent.id, "failed to fetch agent assets");
        StatusCode::INTERNAL_SERVER_ERROR
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

  Ok(Json(result))
}
