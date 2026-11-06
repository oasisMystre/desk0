use axum::{
  Json,
  extract::{Multipart, Path, State},
};
use uuid::Uuid;

use super::{
  dto::{AddDeviceApplicationRequest, CreateApplicationRequest},
  error::ApplicationError,
  models::{Application, DeviceApplication, ResolvedApplication, ResolvedDeviceApplication},
  service::{ApplicationService, DeviceApplicationService},
};
use crate::{
  error::ErrorResponse,
  states::{AppState, AuthRole, AuthenticatedUser},
};

#[utoipa::path(
  post,
  path = "/applications",
  responses(
    (status = 200, description = "Application created", body = ResolvedApplication),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  security(
    ("bearer_auth" = [])
  ),
  tag = "applications"
)]
pub async fn create_application(
  State(AppState { context, .. }): State<AppState>,
  user: AuthenticatedUser,
  multipart: Multipart,
) -> Result<Json<ResolvedApplication>, ApplicationError> {
  user.require_role(&[AuthRole::User])?;

  Ok(Json(
    ApplicationService::create_application(
      &context,
      user.org_id()?,
      CreateApplicationRequest::try_from_multipart(multipart).await?,
    )
    .await?,
  ))
}

#[utoipa::path(
  get,
  path = "/applications",
  responses(
    (status = 200, description = "List applications", body = Vec<ResolvedApplication>),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  security(
    ("bearer_auth" = [])
  ),
  tag = "applications"
)]
pub async fn get_applications(
  State(AppState { context, .. }): State<AppState>,
  user: AuthenticatedUser,
) -> Result<Json<Vec<ResolvedApplication>>, ApplicationError> {
  user.require_role(&[AuthRole::User])?;

  Ok(Json(
    ApplicationService::get_applications(&context, user.org_id()?).await?,
  ))
}

#[utoipa::path(
  get,
  path = "/applications/{id}",
  params(
    ("id" = uuid::Uuid, Path, description = "Application UUID")
  ),
  responses(
    (status = 200, description = "Get application by id", body = ResolvedApplication),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  security(
    ("bearer_auth" = [])
  ),
  tag = "applications"
)]
pub async fn get_application(
  State(AppState { context, .. }): State<AppState>,
  Path(id): Path<Uuid>,
  user: AuthenticatedUser,
) -> Result<Json<ResolvedApplication>, ApplicationError> {
  user.require_role(&[AuthRole::User])?;

  Ok(Json(
    ApplicationService::get_application(&context, user.org_id()?, id).await?,
  ))
}

#[utoipa::path(
  delete,
  path = "/applications/{id}",
  params(
    ("id" = uuid::Uuid, Path, description = "Application UUID")
  ),
  responses(
    (status = 200, description = "Application deleted", body = Application),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  security(
    ("bearer_auth" = [])
  ),
  tag = "applications"
)]
pub async fn delete_application(
  State(AppState { context, .. }): State<AppState>,
  Path(id): Path<Uuid>,
  user: AuthenticatedUser,
) -> Result<Json<Application>, ApplicationError> {
  user.require_role(&[AuthRole::User])?;

  Ok(Json(
    ApplicationService::delete_application(&context, user.org_id()?, id).await?,
  ))
}

#[utoipa::path(
  get,
  path = "/applications/device/{device}",
  params(
    ("device" = uuid::Uuid, Path, description = "Device UUID")
  ),
  responses(
    (status = 200, description = "List device applications", body = Vec<ResolvedDeviceApplication>),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  security(
    ("bearer_auth" = [])
  ),
  tag = "applications"
)]
pub async fn get_device_applications(
  State(AppState { context, .. }): State<AppState>,
  Path(device): Path<Uuid>,
  auth: AuthenticatedUser,
) -> Result<Json<Vec<ResolvedDeviceApplication>>, ApplicationError> {
  auth.require_role(&[AuthRole::User])?;

  Ok(Json(
    DeviceApplicationService::get_device_applications(&context, auth.org_id()?, device).await?,
  ))
}

#[utoipa::path(
  post,
  path = "/applications/device/{device}",
  params(
    ("device" = uuid::Uuid, Path, description = "Device UUID")
  ),
  request_body = AddDeviceApplicationRequest,
  responses(
    (status = 200, description = "Application added to device", body = DeviceApplication),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  security(
    ("bearer_auth" = [])
  ),
  tag = "applications"
)]
pub async fn add_device_application(
  State(AppState { context, .. }): State<AppState>,
  Path(device): Path<Uuid>,
  user: AuthenticatedUser,
  Json(body): Json<AddDeviceApplicationRequest>,
) -> Result<Json<DeviceApplication>, ApplicationError> {
  user.require_role(&[AuthRole::User])?;

  Ok(Json(
    DeviceApplicationService::add_device_application(
      &context,
      user.org_id()?,
      device,
      body.application,
    )
    .await?,
  ))
}

#[utoipa::path(
  delete,
  path = "/applications/device/{device}/{application}",
  params(
    ("device" = uuid::Uuid, Path, description = "Device UUID"),
    ("application" = uuid::Uuid, Path, description = "Application UUID")
  ),
  responses(
    (status = 200, description = "Application removed from device", body = DeviceApplication),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  security(
    ("bearer_auth" = [])
  ),
  tag = "applications"
)]
pub async fn remove_device_application(
  State(AppState { context, .. }): State<AppState>,
  Path((device, application)): Path<(Uuid, Uuid)>,
  user: AuthenticatedUser,
) -> Result<Json<DeviceApplication>, ApplicationError> {
  user.require_role(&[AuthRole::User])?;

  Ok(Json(
    DeviceApplicationService::remove_device_application(
      &context,
      user.org_id()?,
      device,
      application,
    )
    .await?,
  ))
}
