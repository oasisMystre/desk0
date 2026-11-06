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
  error::ErrorResponse,
  routes::devices::dto::CreateDevice,
  states::{AppState, AuthRole, AuthenticatedUser},
};

#[utoipa::path(
  post,
  path = "/devices",
  request_body = CreateDevice,
  responses(
    (status = 200, description = "Device registered", body = Device),
    (status = 400, description = "Bad request", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  tag = "devices"
)]
pub async fn create_device(
  State(AppState { context, .. }): State<AppState>,
  Json(body): Json<CreateDevice>,
) -> Result<Json<Device>, DeviceError> {
  let device = DeviceService::create_device(&context, body).await?;

  Ok(Json(device))
}

#[utoipa::path(
  patch,
  path = "/devices/{device}",
  params(
    ("device" = uuid::Uuid, Path, description = "Device UUID")
  ),
  request_body = PartialDevice,
  responses(
    (status = 200, description = "Device updated", body = Device),
    (status = 400, description = "Bad request", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  tag = "devices"
)]
pub async fn update_device(
  State(AppState { context, .. }): State<AppState>,
  Path(device): Path<Uuid>,
  Json(body): Json<PartialDevice>,
) -> Result<Json<Device>, DeviceError> {
  Ok(Json(
    DeviceService::update_device(&context, device, body).await?,
  ))
}

#[utoipa::path(
  post,
  path = "/devices/challenge/{device}",
  params(
    ("device" = uuid::Uuid, Path, description = "Device UUID")
  ),
  responses(
    (status = 200, description = "Challenge created", body = ChallengeResponse),
    (status = 400, description = "Bad request", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  tag = "devices"
)]
pub async fn create_challenge(
  State(AppState { context, .. }): State<AppState>,
  Path(device): Path<Uuid>,
) -> Result<Json<ChallengeResponse>, DeviceError> {
  Ok(Json(
    DeviceService::create_challenge(&context, device).await?,
  ))
}

#[utoipa::path(
  post,
  path = "/devices/token/{device}",
  operation_id = "create_device_connection_token",
  params(
    ("device" = uuid::Uuid, Path, description = "Device UUID")
  ),
  request_body = CreateConnectionTokenRequest,
  responses(
    (status = 200, description = "Device connection token", body = CreateConnectionTokenResponse),
    (status = 400, description = "Bad request", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  tag = "devices"
)]
pub async fn create_connection_token(
  State(AppState { context, .. }): State<AppState>,
  Path(device): Path<Uuid>,
  Json(body): Json<CreateConnectionTokenRequest>,
) -> Result<Json<CreateConnectionTokenResponse>, DeviceError> {
  Ok(Json(
    DeviceService::create_connection_token(&context, device, body).await?,
  ))
}

#[utoipa::path(
  get,
  path = "/devices",
  responses(
    (status = 200, description = "List organization devices", body = Vec<Device>),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  security(
    ("bearer_auth" = [])
  ),
  tag = "devices"
)]
pub async fn get_devices(
  State(AppState { context, .. }): State<AppState>,
  user: AuthenticatedUser,
) -> Result<Json<Vec<Device>>, DeviceError> {
  user.require_role(&[AuthRole::User])?;

  Ok(Json(
    DeviceService::get_devices(&context, user.org_id()?).await?,
  ))
}

#[utoipa::path(
  delete,
  path = "/devices/{device}",
  params(
    ("device" = uuid::Uuid, Path, description = "Device UUID")
  ),
  responses(
    (status = 200, description = "Device deleted", body = Device),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  security(
    ("bearer_auth" = [])
  ),
  tag = "devices"
)]
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
