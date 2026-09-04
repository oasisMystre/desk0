use std::{collections::BTreeMap, sync::Arc};

use axum::{
  extract::FromRequestParts,
  http::{StatusCode, request::Parts},
};
use diesel::{ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use flow::{organization::Organization, organizations, user::User, users};
use jwt::VerifyWithKey;

use crate::context::Context;

pub struct AuthenticatedUser {
  pub user: uuid::Uuid,
  pub org_id: Option<uuid::Uuid>,
}

pub struct MaybeGuestUser {
  pub uid: Option<String>,
}

impl AuthenticatedUser {
  pub fn org_id(&self) -> anyhow::Result<uuid::Uuid, StatusCode> {
    self.org_id.ok_or_else(|| StatusCode::UNAUTHORIZED)
  }
}

impl FromRequestParts<Context> for AuthenticatedUser {
  type Rejection = StatusCode;

  async fn from_request_parts(parts: &mut Parts, state: &Context) -> Result<Self, Self::Rejection> {
    let authorization = parts
      .headers
      .get("authorization")
      .and_then(|value| value.to_str().ok());

    let org_id = parts
      .headers
      .get("x-org-id")
      .and_then(|value| value.to_str().ok());

    if let Some(authorization) = authorization {
      let token = authorization
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::BAD_REQUEST)?;

      let claims: BTreeMap<String, String> = token
        .verify_with_key(&Arc::unwrap_or_clone(state.auth_key.clone()))
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

      let sub = claims["sub"].clone();
      let org_id = org_id.map(|value| value.parse::<uuid::Uuid>()).clone();

      return Ok(Self {
        org_id: if let Some(org_id) = org_id {
          Some(org_id.map_err(|_| StatusCode::BAD_REQUEST)?)
        } else {
          None
        },
        user: sub
          .parse::<uuid::Uuid>()
          .map_err(|_| StatusCode::BAD_REQUEST)?,
      });
    }

    if let Some(org_id) = org_id {
      let org_id = org_id
        .parse::<uuid::Uuid>()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
      let mut connection = state.pg.get().await.map_err(|error| {
        tracing::error!(?error, "pg connection error");
        StatusCode::INTERNAL_SERVER_ERROR
      })?;

      let result: Option<(User, Organization)> = organizations::table
        .inner_join(users::table.on(users::id.eq(organizations::user)))
        .filter(organizations::id.eq(org_id))
        .select((users::all_columns, organizations::all_columns))
        .first(&mut connection)
        .await
        .optional()
        .map_err(|error| {
          tracing::error!(?error, "pg query error");
          StatusCode::INTERNAL_SERVER_ERROR
        })?;

      if let Some((user, organization)) = result
        && user.uid.is_none()
      {
        return Ok(Self {
          user: user.id,
          org_id: Some(organization.id),
        });
      }
    }

    Err(StatusCode::UNAUTHORIZED)
  }
}

impl FromRequestParts<Context> for MaybeGuestUser {
  type Rejection = StatusCode;

  async fn from_request_parts(
    _parts: &mut Parts,
    _state: &Context,
  ) -> Result<Self, Self::Rejection> {
    Ok(Self { uid: None })
  }
}
