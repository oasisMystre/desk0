use axum::{
  Json,
  extract::{Path, State},
};
use uuid::Uuid;

use super::{
  dto::{
    ChallengeResponse, CreateConnectionTokenRequest, CreateConnectionTokenResponse, PartialDevice,
  },
  error::DeviceError,
  models::Device,
  service::DeviceService,
};
use crate::{
  routes::devices::dto::CreateDevice,
  states::{AppState, AuthRole, AuthenticatedUser},
};

pub async fn create_device(
  State(AppState { context, .. }): State<AppState>,
  Json(body): Json<CreateDevice>,
) -> Result<Json<Device>, DeviceError> {
  let device = DeviceService::create_device(&context, body).await?;

  Ok(Json(device))
}

pub async fn update_device(
  State(AppState { context, .. }): State<AppState>,
  Path(device): Path<Uuid>,
  Json(body): Json<PartialDevice>,
) -> Result<Json<Device>, DeviceError> {
  Ok(Json(
    DeviceService::update_device(&context, device, body).await?,
  ))
}

pub async fn create_challenge(
  State(AppState { context, .. }): State<AppState>,
  Path(device): Path<Uuid>,
) -> Result<Json<ChallengeResponse>, DeviceError> {
  Ok(Json(
    DeviceService::create_challenge(&context, device).await?,
  ))
}

pub async fn create_connection_token(
  State(AppState { context, .. }): State<AppState>,
  Path(device): Path<Uuid>,
  Json(body): Json<CreateConnectionTokenRequest>,
) -> Result<Json<CreateConnectionTokenResponse>, DeviceError> {
  Ok(Json(
    DeviceService::create_connection_token(&context, device, body).await?,
  ))
}

pub async fn get_devices(
  State(AppState { context, .. }): State<AppState>,
  user: AuthenticatedUser,
) -> Result<Json<Vec<Device>>, DeviceError> {
  user.require_role(&[AuthRole::User])?;

  Ok(Json(
    DeviceService::get_devices(&context, user.org_id()?).await?,
  ))
}

pub async fn delete_device(
  State(AppState { context, .. }): State<AppState>,
  Path(device): Path<Uuid>,
  user: AuthenticatedUser,
) -> Result<Json<Device>, DeviceError> {
  user.require_role(&[AuthRole::User])?;

  Ok(Json(
    DeviceService::delete_device(&context, device, user.org_id()?).await?,
  ))
}
