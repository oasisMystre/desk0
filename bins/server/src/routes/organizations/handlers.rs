use axum::{
  Json,
  extract::{Path, State},
};

use super::{
  dto::{CreateConnectionTokenResponse, CreateOrganizationRequest},
  error::OrganizationError,
  models::Organization,
  service::OrganizationService,
};
use crate::states::{AppState, AuthRole, AuthenticatedUser};

pub async fn get_organizations(
  State(AppState { context, .. }): State<AppState>,
  user: AuthenticatedUser,
) -> Result<Json<Vec<Organization>>, OrganizationError> {
  user.require_role(&[AuthRole::User])?;

  let orgs = OrganizationService::get_organizations(&context, user.user).await?;
  Ok(Json(orgs))
}

pub async fn create_organization(
  State(AppState { context, .. }): State<AppState>,
  user: AuthenticatedUser,
  Json(body): Json<CreateOrganizationRequest>,
) -> Result<Json<Organization>, OrganizationError> {
  user.require_role(&[AuthRole::User])?;

  let org = OrganizationService::create_organization(&context, user.user, body).await?;
  Ok(Json(org))
}

pub async fn create_connection_token(
  State(AppState { context, .. }): State<AppState>,
  Path(device_id): Path<uuid::Uuid>,
  user: AuthenticatedUser,
) -> Result<Json<CreateConnectionTokenResponse>, OrganizationError> {
  user.require_role(&[AuthRole::User])?;

  let response =
    OrganizationService::create_connection_token(&context, user.user, user.org_id()?, device_id).await?;
  Ok(Json(response))
}
