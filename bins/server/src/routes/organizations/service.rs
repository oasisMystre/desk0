use std::{collections::BTreeMap, sync::Arc};

use chrono::Utc;
use jwt::SignWithKey;
use uuid::Uuid;

use super::{
  dto::{CreateConnectionTokenResponse, CreateOrganizationRequest},
  error::OrganizationError,
  models::Organization,
  repository::OrganizationRepository,
};
use crate::context::Context;

pub struct OrganizationService;

impl OrganizationService {
  pub async fn get_organizations(
    context: &Context,
    user: uuid::Uuid,
  ) -> Result<Vec<Organization>, OrganizationError> {
    let mut connection = context.get_pg().await?;

    OrganizationRepository::find_by_user(&mut connection, user)
      .await
      .map_err(|error| {
        tracing::error!(?error, "failed to fetch organization");
        OrganizationError::FetchFailed
      })
  }

  pub async fn create_organization(
    context: &Context,
    user: Uuid,
    request: CreateOrganizationRequest,
  ) -> Result<Organization, OrganizationError> {
    let mut connection = context.get_pg().await?;

    OrganizationRepository::create(
      &mut connection,
      &Organization {
        user,
        name: request.name,
        created_at: Utc::now(),
        id: uuid::Uuid::new_v4(),
      },
    )
    .await
    .map_err(|error| {
      tracing::error!(?error, "failed to insert organization");
      OrganizationError::CreateFailed
    })
  }

  pub async fn create_connection_token(
    context: &Context,
    user_id: uuid::Uuid,
    org_id: uuid::Uuid,
    device: uuid::Uuid,
  ) -> Result<CreateConnectionTokenResponse, OrganizationError> {
    let now = Utc::now();
    let jti = uuid::Uuid::new_v4().to_string();

    let mut claims = BTreeMap::new();

    claims.insert("jti", jti.clone());
    claims.insert("sub", org_id.to_string());
    claims.insert("user", user_id.to_string());
    claims.insert("device", device.to_string());
    claims.insert("role", "organization".into());
    claims.insert("iat", now.timestamp().to_string());

    let token = claims
      .sign_with_key(&Arc::unwrap_or_clone(context.auth_key.clone()))
      .map_err(|error| {
        tracing::error!(?error, "claim sign error");
        OrganizationError::TokenSignFailed
      })?;

    crate::states::register_connection_token(context, &token, &jti).await?;

    Ok(CreateConnectionTokenResponse { token })
  }
}
