use std::{collections::BTreeMap, sync::Arc};

use axum::{extract::FromRequestParts, http::request::Parts};
use chrono::Utc;
use deadpool_redis::redis::AsyncCommands;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use flow::organizations;
use jwt::VerifyWithKey;

use crate::{context::Context, error::SharedError, states::AppState};

pub const CONNECTION_TOKEN_TTL_SECS: u64 = 15 * 60;

pub fn connection_token_keys(token: &str, jti: Option<&str>) -> Vec<String> {
  let mut keys = vec![format!("connection_token:{}", token)];
  if let Some(jti) = jti {
    keys.push(format!("connection_token:{}", jti));
  }
  keys
}

pub async fn register_connection_token(
  context: &Context,
  token: &str,
  jti: &str,
) -> Result<(), SharedError> {
  let mut redis = context.get_redis().await?;
  let keys = connection_token_keys(token, Some(jti));
  for key in keys {
    let _: () = redis
      .set_ex(&key, "unused", CONNECTION_TOKEN_TTL_SECS)
      .await
      .map_err(|error| {
        tracing::error!(?error, "failed to save connection token in redis");
        SharedError::RedisOperation
      })?;
  }
  Ok(())
}

pub async fn consume_connection_token(
  context: &Context,
  token: &str,
  jti: Option<&str>,
) -> Result<(), SharedError> {
  let mut redis = context.get_redis().await?;
  let keys = connection_token_keys(token, jti);
  let deleted: u64 = redis.del(&keys).await.map_err(|error| {
    tracing::error!(?error, "failed to delete connection token from redis");
    SharedError::RedisOperation
  })?;

  if deleted == 0 {
    tracing::warn!("connection token has already been used or expired");
    return Err(SharedError::InvalidToken);
  }

  Ok(())
}

#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  Eq,
  strum::Display,
  strum::EnumString,
  serde::Serialize,
  serde::Deserialize,
)]
#[strum(serialize_all = "snake_case")]
pub enum AuthRole {
  User,
  Agent,
  Organization,
}

#[derive(Clone)]
pub struct AuthenticatedUser {
  pub role: AuthRole,
  pub user: uuid::Uuid,
  pub org_id: Option<uuid::Uuid>,
  pub session_id: Option<uuid::Uuid>,
}

impl AuthenticatedUser {
  pub fn org_id(&self) -> Result<uuid::Uuid, SharedError> {
    self.org_id.ok_or(SharedError::OrgMembershipRequired)
  }

  pub fn session_id(&self) -> Option<uuid::Uuid> {
    self.session_id
  }

  pub fn require_role(&self, allowed: &[AuthRole]) -> Result<(), SharedError> {
    if allowed.contains(&self.role) {
      Ok(())
    } else {
      Err(SharedError::UnsupportedRole)
    }
  }

  fn validate_user_claims(
    claims: &BTreeMap<String, String>,
  ) -> Result<(uuid::Uuid, Option<uuid::Uuid>), SharedError> {
    let exp = claims
      .get("exp")
      .map(|value| value.parse::<i64>())
      .ok_or(SharedError::InvalidToken)?
      .map_err(|error| {
        tracing::error!(?error, "invalid exp claim in token");
        SharedError::InvalidToken
      })?;

    if Utc::now().timestamp() >= exp {
      return Err(SharedError::InvalidToken);
    }

    let session_id = claims
      .get("sid")
      .and_then(|value| value.parse::<uuid::Uuid>().ok());

    let user_id = claims
      .get("sub")
      .map(|value| value.parse::<uuid::Uuid>())
      .ok_or(SharedError::MissingSubjectClaim)?
      .map_err(|error| {
        tracing::error!(?error, "invalid user id in token");
        SharedError::InvalidClaimId
      })?;

    Ok((user_id, session_id))
  }
}

impl FromRequestParts<AppState> for AuthenticatedUser {
  type Rejection = SharedError;

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

    let Some(authorization) = authorization else {
      return Err(SharedError::MissingAuthHeader);
    };

    let Some(token) = authorization.strip_prefix("Bearer ") else {
      return Err(SharedError::InvalidAuthHeader);
    };

    let claims: BTreeMap<String, String> = token
      .verify_with_key(&Arc::unwrap_or_clone(context.auth_key.clone()))
      .map_err(|_| SharedError::InvalidToken)?;

    if let Some(exp) = claims.get("exp") {
      let exp = exp.parse::<i64>().map_err(|error| {
        tracing::error!(?error, "invalid exp claim in token");
        SharedError::InvalidToken
      })?;

      if Utc::now().timestamp() >= exp {
        return Err(SharedError::InvalidToken);
      }
    }

    let role = claims
      .get("role")
      .ok_or(SharedError::MissingRoleClaim)?
      .parse::<AuthRole>()
      .map_err(|error| {
        tracing::error!(?error, "unsupported role");
        SharedError::UnsupportedRole
      })?;

    match role {
      AuthRole::Organization | AuthRole::Agent => {
        let jti = claims.get("jti").map(|s| s.as_str());
        consume_connection_token(context, token, jti).await?;

        let org_id = claims
          .get("sub")
          .map(|value| value.parse::<uuid::Uuid>())
          .ok_or(SharedError::MissingOrgClaim)?
          .map_err(|error| {
            tracing::error!(?error, "invalid organization id in token");
            SharedError::InvalidClaimId
          })?;

        let user_id = match claims.get("user") {
          Some(user_str) => user_str.parse::<uuid::Uuid>().map_err(|error| {
            tracing::error!(?error, "invalid user id in token");
            SharedError::InvalidClaimId
          })?,
          None if role == AuthRole::Agent => claims
            .get("device")
            .map(|value| value.parse::<uuid::Uuid>())
            .ok_or(SharedError::MissingSubjectClaim)?
            .map_err(|error| {
              tracing::error!(?error, "invalid device id in agent token");
              SharedError::InvalidClaimId
            })?,
          None => return Err(SharedError::MissingUserClaim),
        };

        Ok(Self {
          role,
          user: user_id,
          session_id: None,
          org_id: Some(org_id),
        })
      }
      AuthRole::User => {
        let (user_id, session_id) = Self::validate_user_claims(&claims)?;

        let org_id = org_id
          .map(|value| {
            value.parse::<uuid::Uuid>().map_err(|error| {
              tracing::error!(?error, "invalid organization id header");
              SharedError::InvalidOrgHeader
            })
          })
          .transpose()?;

        if let Some(org_id) = org_id {
          let mut connection = context.get_pg().await?;

          let organization: Option<uuid::Uuid> = organizations::table
            .filter(organizations::id.eq(org_id))
            .filter(organizations::user.eq(user_id))
            .select(organizations::id)
            .first(&mut connection)
            .await
            .optional()
            .map_err(|error| {
              tracing::error!(?error, "organization membership query error");
              SharedError::DatabaseOperation
            })?;

          if organization.is_none() {
            return Err(SharedError::OrgMembershipForbidden);
          }
        }

        Ok(Self {
          role,
          org_id,
          session_id,
          user: user_id,
        })
      }
    }
  }
}
