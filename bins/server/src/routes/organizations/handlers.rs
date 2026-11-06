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
use crate::{
  error::ErrorResponse,
  states::{AppState, AuthRole, AuthenticatedUser},
};

#[utoipa::path(
  get,
  path = "/organizations",
  responses(
    (status = 200, description = "List organizations", body = Vec<Organization>),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  security(
    ("bearer_auth" = [])
  ),
  tag = "organizations"
)]
pub async fn get_organizations(
  State(AppState { context, .. }): State<AppState>,
  user: AuthenticatedUser,
) -> Result<Json<Vec<Organization>>, OrganizationError> {
  user.require_role(&[AuthRole::User])?;

  let orgs = OrganizationService::get_organizations(&context, user.user).await?;
  Ok(Json(orgs))
}

#[utoipa::path(
  post,
  path = "/organizations",
  request_body = CreateOrganizationRequest,
  responses(
    (status = 200, description = "Organization created", body = Organization),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  security(
    ("bearer_auth" = [])
  ),
  tag = "organizations"
)]
pub async fn create_organization(
  State(AppState { context, .. }): State<AppState>,
  user: AuthenticatedUser,
  Json(body): Json<CreateOrganizationRequest>,
) -> Result<Json<Organization>, OrganizationError> {
  user.require_role(&[AuthRole::User])?;

  let org = OrganizationService::create_organization(&context, user.user, body).await?;
  Ok(Json(org))
}

#[utoipa::path(
  post,
  path = "/organizations/token/{device_id}",
  params(
    ("device_id" = uuid::Uuid, Path, description = "Device UUID")
  ),
  responses(
    (status = 200, description = "Connection token created", body = CreateConnectionTokenResponse),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  security(
    ("bearer_auth" = [])
  ),
  tag = "organizations"
)]
pub async fn create_organization_connection_token(
  State(AppState { context, .. }): State<AppState>,
  Path(device_id): Path<uuid::Uuid>,
  user: AuthenticatedUser,
) -> Result<Json<CreateConnectionTokenResponse>, OrganizationError> {
  user.require_role(&[AuthRole::User])?;

  let response =
    OrganizationService::create_connection_token(&context, user.user, user.org_id()?, device_id)
      .await?;
  Ok(Json(response))
}
