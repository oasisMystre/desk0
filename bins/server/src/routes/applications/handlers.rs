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
use crate::states::{AppState, AuthRole, AuthenticatedUser};

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

pub async fn get_applications(
  State(AppState { context, .. }): State<AppState>,
  user: AuthenticatedUser,
) -> Result<Json<Vec<ResolvedApplication>>, ApplicationError> {
  user.require_role(&[AuthRole::User])?;

  Ok(Json(
    ApplicationService::get_applications(&context, user.org_id()?).await?,
  ))
}

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
