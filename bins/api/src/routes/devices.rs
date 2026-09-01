use axum::{
  Json,
  extract::{Path, State},
  http::StatusCode,
  response::IntoResponse,
};
use diesel::{
  BoolExpressionMethods, ExpressionMethods, HasQuery, QueryDsl, SelectableHelper,
  query_builder::AsChangeset,
};
use diesel_async::RunQueryDsl;
use flow::{device::Device, devices};
use serde::{Deserialize, Serialize};

use crate::{auth::AuthenticatedUser, context::Context};

#[derive(Deserialize, Serialize, AsChangeset)]
#[diesel(table_name = devices)]
pub struct PartialDevice {
  os: Option<String>,
  hostname: Option<String>,
  username: Option<String>,
  version: Option<String>,
}

pub async fn get_devices(
  State(context): State<Context>,
  user: AuthenticatedUser,
) -> Result<impl IntoResponse, StatusCode> {
  let mut connection = context.pg.get().await.map_err(|error| {
    tracing::error!(?error, "database connection error");
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  let devices: Vec<Device> = Device::query()
    .filter(devices::organization.eq(user.org_id))
    .load(&mut connection)
    .await
    .map_err(|error| {
      tracing::error!(?error, "database query error");
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  Ok(Json(devices))
}

pub async fn create_device(
  State(context): State<Context>,
  user: AuthenticatedUser,
  Json(body): Json<PartialDevice>,
) -> Result<impl IntoResponse, StatusCode> {
  let device = Device {
    id: uuid::Uuid::new_v4(),
    os: body.os,
    hostname: body.hostname,
    username: body.username,
    version: body.version,
    organization: user.org_id,
    ..Default::default()
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

pub async fn delete_device(
  State(context): State<Context>,
  Path(device_id): Path<uuid::Uuid>,
  user: AuthenticatedUser,
) -> Result<impl IntoResponse, StatusCode> {
  let mut connection = context.pg.get().await.map_err(|error| {
    tracing::error!(?error, "database connection error");
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  let result: Device = diesel::delete(
    devices::table.filter(
      devices::id
        .eq(device_id)
        .and(devices::organization.eq(user.org_id)),
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

  Ok(Json(serde_json::json!(result)))
}

pub async fn update_device(
  State(context): State<Context>,
  Path(device_id): Path<uuid::Uuid>,
  user: AuthenticatedUser,
  Json(body): Json<PartialDevice>,
) -> Result<impl IntoResponse, StatusCode> {
  let mut connection = context.pg.get().await.map_err(|error| {
    tracing::error!(?error, "database connection error");
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  let result: Device = diesel::update(
    devices::table.filter(
      devices::id
        .eq(device_id)
        .and(devices::organization.eq(user.org_id)),
    ),
  )
  .set(&body)
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
