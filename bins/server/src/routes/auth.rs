use std::{collections::BTreeMap, sync::Arc};

use axum::{
  extract::FromRequestParts,
  http::{StatusCode, request::Parts},
};
use diesel::{ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use flow::{devices, organization::Organization, organizations, user::User, users};
use jwt::VerifyWithKey;

use crate::{context::Context, routes::state::AppState};

pub struct AuthenticatedUser {
  pub user: uuid::Uuid,
  pub org_id: Option<uuid::Uuid>,
}

/// all agent maybe guest user
/// for some requests valid org_id or device_id is need
pub struct MaybeGuestUser {
  pub org_id: Option<uuid::Uuid>,
  pub device_id: Option<uuid::Uuid>,
}

impl AuthenticatedUser {
  pub fn org_id(&self) -> anyhow::Result<uuid::Uuid, StatusCode> {
    self.org_id.ok_or_else(|| StatusCode::UNAUTHORIZED)
  }
}

impl FromRequestParts<AppState> for AuthenticatedUser {
  type Rejection = StatusCode;

  async fn from_request_parts(
    parts: &mut Parts,
    AppState { context, .. }: &AppState,
  ) -> Result<Self, Self::Rejection> {
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
        .verify_with_key(&Arc::unwrap_or_clone(context.auth_key.clone()))
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
      let mut connection = context.pg.get().await.map_err(|error| {
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

impl MaybeGuestUser {
  pub fn device_id(&self) -> Result<uuid::Uuid, StatusCode> {
    match self.device_id {
      Some(value) => Ok(value),
      None => Err(StatusCode::UNAUTHORIZED),
    }
  }

  pub fn org_id(&self) -> Result<uuid::Uuid, StatusCode> {
    match self.org_id {
      Some(value) => Ok(value),
      None => Err(StatusCode::UNAUTHORIZED),
    }
  }
}

impl FromRequestParts<Context> for MaybeGuestUser {
  type Rejection = StatusCode;

  async fn from_request_parts(parts: &mut Parts, state: &Context) -> Result<Self, Self::Rejection> {
    let device_id = parts
      .headers
      .get("x-device-id")
      .and_then(|value| value.to_str().ok());

    let mut connection = state.pg.get().await.map_err(|error| {
      tracing::error!(?error, "pg connection error");
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if let Some(device_id) = device_id {
      let device_id = device_id.parse::<uuid::Uuid>().map_err(|error| {
        tracing::error!(?error, "parsing uuid error");
        StatusCode::BAD_REQUEST
      })?;
      let org_id: Option<uuid::Uuid> = devices::table
        .select(devices::organization)
        .filter(devices::id.eq(device_id))
        .first(&mut connection)
        .await
        .optional()
        .map_err(|error| {
          tracing::error!(?error, "pg query error");
          StatusCode::INTERNAL_SERVER_ERROR
        })?;

      return Ok(Self {
        org_id,
        device_id: Some(device_id),
      });
    }

    Ok(Self {
      org_id: None,
      device_id: None,
    })
  }
}
