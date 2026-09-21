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
use serde::{Deserialize, Serialize};

use crate::routes::{auth::AuthenticatedUser, state::AppState};

#[derive(Deserialize, Serialize)]
pub struct PartialDevice {
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
      tracing::error!(?error, "database query error");
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  Ok(Json(results))
}

pub async fn create_device(
  State(AppState { context, .. }): State<AppState>,
  user: AuthenticatedUser,
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
    organization: user.org_id()?,
    last_logged_in: Utc::now(),
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
      tracing::error!(?error, "database insert error");
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  Ok(Json(device))
}

pub async fn update_device(
  State(AppState { context, .. }): State<AppState>,

  Path(device_id): Path<uuid::Uuid>,
  user: AuthenticatedUser,
  Json(body): Json<PartialDevice>,
) -> Result<Json<Device>, StatusCode> {
  let mut connection = context.pg.get().await.map_err(|error| {
    tracing::error!(?error, "database connection error");
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  let update = UpdateDevice {
    os: body.os,
    hostname: body.hostname,
    username: body.username,
    version: body.version,
    last_logged_in: Some(Utc::now()),
  };

  let result: Device = diesel::update(
    devices::table.filter(
      devices::id
        .eq(device_id)
        .and(devices::organization.eq(user.org_id()?)),
    ),
  )
  .set(&update)
  .returning(Device::as_returning())
  .get_result(&mut connection)
  .await
  .map_err(|error| match error {
    diesel::result::Error::NotFound => StatusCode::NOT_FOUND,
    _ => {
      tracing::error!(?error, "database update error");
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
      tracing::error!(?error, "pg update error");
      StatusCode::INTERNAL_SERVER_ERROR
    }
  })?;

  Ok(Json(result))
}
