use std::{collections::BTreeMap, sync::Arc};

use axum::{
  Json,
  extract::{Path, State},
  http::StatusCode,
  response::IntoResponse,
};
use chrono::{DateTime, Utc};
use diesel::{
  BoolExpressionMethods, ExpressionMethods, HasQuery, QueryDsl, SelectableHelper,
  query_builder::AsChangeset,
};
use diesel_async::RunQueryDsl;
use flow::{device::Device, devices};
use jwt::SignWithKey;
use serde::{Deserialize, Serialize};

use crate::routes::{
  auth::AuthenticatedUser, state::AppState, types::CreateConnectionTokenResponse,
};

#[derive(Deserialize, Serialize)]
pub struct PartialDevice {
  organization: uuid::Uuid,
  os: Option<String>,
  hostname: Option<String>,
  username: Option<String>,
  version: Option<String>,
  metadata: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize, AsChangeset)]
#[diesel(table_name = devices)]
pub struct UpdateDevice {
  os: Option<String>,
  hostname: Option<String>,
  username: Option<String>,
  version: Option<String>,
  metadata: Option<serde_json::Value>,
  #[diesel(column_name = "lastLoggedIn")]
  pub last_logged_in: Option<DateTime<Utc>>,
}

pub async fn get_devices(
  State(AppState { context, .. }): State<AppState>,
  user: AuthenticatedUser,
) -> Result<Json<Vec<Device>>, StatusCode> {
  let mut connection = context.pg.get().await.map_err(|error| {
    tracing::error!(?error, "database connection error");
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  let results: Vec<Device> = Device::query()
    .filter(devices::organization.eq(user.org_id()?))
    .load(&mut connection)
    .await
    .map_err(|error| {
      tracing::error!(?error, "failed to fetch devices");
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  Ok(Json(results))
}

pub async fn create_device(
  State(AppState { context, .. }): State<AppState>,
  Json(body): Json<PartialDevice>,
) -> Result<impl IntoResponse, StatusCode> {
  let device = Device {
    os: body.os,
    metadata: body.metadata,
    hostname: body.hostname,
    username: body.username,
    version: body.version,
    created_at: Utc::now(),
    id: uuid::Uuid::new_v4(),
    last_logged_in: Utc::now(),
    organization: body.organization,
  };

  let mut connection = context.pg.get().await.map_err(|error| {
    tracing::error!(?error, "database connection error");
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  let device = diesel::insert_into(devices::table)
    .values(&device)
    .returning(Device::as_returning())
    .get_result::<Device>(&mut connection)
    .await
    .map_err(|error| {
      tracing::error!(?error, "failed to insert device");
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  Ok(Json(device))
}

pub async fn update_device(
  State(AppState { context, .. }): State<AppState>,
  Path(device_id): Path<uuid::Uuid>,
  Json(body): Json<PartialDevice>,
) -> Result<Json<Device>, StatusCode> {
  let mut connection = context.pg.get().await.map_err(|error| {
    tracing::error!(?error, "database connection error");
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  let update = UpdateDevice {
    os: body.os,
    version: body.version,
    hostname: body.hostname,
    username: body.username,
    metadata: body.metadata,
    last_logged_in: Some(Utc::now()),
  };

  let result: Device = diesel::update(
    devices::table.filter(
      devices::id
        .eq(device_id)
        .and(devices::organization.eq(body.organization)),
    ),
  )
  .set(&update)
  .returning(Device::as_returning())
  .get_result(&mut connection)
  .await
  .map_err(|error| match error {
    diesel::result::Error::NotFound => StatusCode::NOT_FOUND,
    _ => {
      tracing::error!(?error, "failed to update device");
      StatusCode::INTERNAL_SERVER_ERROR
    }
  })?;

  Ok(Json(result))
}

pub async fn delete_device(
  State(AppState { context, .. }): State<AppState>,
  Path(device_id): Path<uuid::Uuid>,
  user: AuthenticatedUser,
) -> Result<Json<Device>, StatusCode> {
  let mut connection = context.pg.get().await.map_err(|error| {
    tracing::error!(?error, "database connection error");
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  let result: Device = diesel::delete(
    devices::table.filter(
      devices::id
        .eq(device_id)
        .and(devices::organization.eq(user.org_id()?)),
    ),
  )
  .returning(Device::as_returning())
  .get_result(&mut connection)
  .await
  .map_err(|error| match error {
    diesel::result::Error::NotFound => StatusCode::NOT_FOUND,
    _ => {
      tracing::error!(?error, "failed to delete device");
      StatusCode::INTERNAL_SERVER_ERROR
    }
  })?;

  Ok(Json(result))
}

pub async fn create_connection_token(
  State(AppState { context, .. }): State<AppState>,
  Path(device_id): Path<uuid::Uuid>,
) -> Result<Json<CreateConnectionTokenResponse>, StatusCode> {
  let mut connection = context.pg.get().await.map_err(|error| {
    tracing::error!(?error, "database connection error");
    StatusCode::INTERNAL_SERVER_ERROR
  })?;
  let Ok(organization) = devices::table
    .select(devices::organization)
    .find(device_id)
    .get_result::<uuid::Uuid>(&mut connection)
    .await
  else {
    return Err(StatusCode::NOT_FOUND);
  };
  let mut claims = BTreeMap::new();
  claims.insert("role", "agent".into());
  claims.insert("sub", organization.to_string());
  claims.insert("device", device_id.to_string());

  let token = claims
    .sign_with_key(&Arc::unwrap_or_clone(context.auth_key.clone()))
    .map_err(|error| {
      tracing::error!(?error, "claim sign error");
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  Ok(Json(CreateConnectionTokenResponse { token }))
}
