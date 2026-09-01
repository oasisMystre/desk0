use std::{collections::BTreeMap, sync::Arc};

use axum::{
  Json,
  extract::{Path, State},
  http::StatusCode,
};
use diesel::SelectableHelper;
use diesel_async::RunQueryDsl;
use flow::{organization::Organization, organizations};
use jwt::SignWithKey;
use serde::{Deserialize, Serialize};

use crate::{
  auth::{AuthenticatedUser, MaybeGuestUser},
  context::Context,
};

#[derive(Deserialize)]
pub struct CreateOrganization {
  pub name: Option<String>,
}

#[derive(Serialize)]
pub struct CreateConnectionTokenResponse {
  pub token: String,
}

pub async fn create_organization(
  State(context): State<Context>,
  user: MaybeGuestUser,
  Json(body): Json<CreateOrganization>,
) -> Result<Json<Organization>, StatusCode> {
  let mut connection = context.pg.get().await.map_err(|error| {
    tracing::error!(?error, "database connection error");
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  let organization = Organization {
    id: uuid::Uuid::new_v4(),
    uid: user.uid,
    name: body.name,
    ..Default::default()
  };
  let organization = diesel::insert_into(organizations::table)
    .values(&organization)
    .returning(Organization::as_returning())
    .get_result::<Organization>(&mut connection)
    .await
    .map_err(|error| {
      tracing::error!(?error, "database insert error");
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  Ok(Json(organization))
}

pub async fn create_connection_token(
  State(context): State<Context>,
  Path(device_id): Path<uuid::Uuid>,
  user: AuthenticatedUser,
) -> Result<Json<CreateConnectionTokenResponse>, StatusCode> {
  let mut claims = BTreeMap::new();
  claims.insert("sub", user.org_id.to_string());
  claims.insert("device", device_id.to_string());

  let token = claims
    .sign_with_key(&Arc::unwrap_or_clone(context.auth_key))
    .map_err(|error| {
      tracing::error!(?error, "claim sign error");
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  Ok(Json(CreateConnectionTokenResponse { token }))
}
