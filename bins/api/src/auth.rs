use std::{collections::BTreeMap, sync::Arc};

use axum::{
  extract::FromRequestParts,
  http::{StatusCode, request::Parts},
};
use diesel::{HasQuery, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use flow::organization::Organization;
use jwt::VerifyWithKey;

use crate::context::Context;

pub struct AuthenticatedUser {
  pub org_id: uuid::Uuid,
}

pub struct MaybeGuestUser {
  pub uid: Option<String>,
}

impl FromRequestParts<Context> for AuthenticatedUser {
  type Rejection = StatusCode;

  async fn from_request_parts(parts: &mut Parts, state: &Context) -> Result<Self, Self::Rejection> {
    let authorization = parts
      .headers
      .get("authorization")
      .and_then(|value| value.to_str().ok());

    if let Some(authorization) = authorization {
      let token = authorization
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::BAD_REQUEST)?;
      let claims: BTreeMap<String, String> = token
        .verify_with_key(&Arc::unwrap_or_clone(state.auth_key.clone()))
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
      let sub = claims["sub"].clone();
      return Ok(Self {
        org_id: sub
          .parse::<uuid::Uuid>()
          .map_err(|_| StatusCode::BAD_REQUEST)?,
      });
    }

    let org_id = parts
      .headers
      .get("x-org-id")
      .and_then(|value| value.to_str().ok());

    if let Some(org_id) = org_id {
      let org_id = org_id
        .parse::<uuid::Uuid>()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
      let mut connection = state.pg.get().await.map_err(|error| {
        tracing::error!(?error, "pg connection error");
        StatusCode::INTERNAL_SERVER_ERROR
      })?;

      let result: Option<Organization> = Organization::query()
        .find(org_id)
        .first(&mut connection)
        .await
        .optional()
        .map_err(|error| {
          tracing::error!(?error, "pg query error");
          StatusCode::INTERNAL_SERVER_ERROR
        })?;

      if let Some(result) = result
        && result.uid.is_none()
      {
        return Ok(Self { org_id });
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
