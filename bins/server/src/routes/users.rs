use axum::{Json, extract::State, http::StatusCode};
use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{AsyncConnection, RunQueryDsl};
use flow::{organization::Organization, organizations, user::User, users};
use serde::Serialize;

use crate::routes::{auth::AuthenticatedUser, state::AppState};

#[derive(Serialize)]
pub struct CreateUserResponse {
  pub user: User,
  pub organization: Organization,
}

pub async fn create_user(
  State(AppState { context, .. }): State<AppState>,
) -> Result<Json<CreateUserResponse>, StatusCode> {
  let mut connection = context.pg.get().await.map_err(|error| {
    tracing::error!(?error, "database connection error");
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  let user = User {
    uid: None,
    id: uuid::Uuid::new_v4(),
    created_at: Utc::now(),
    last_logged_in: Utc::now(),
  };

  let organization = Organization {
    id: uuid::Uuid::new_v4(),
    name: Some("default".into()),
    user: user.id,
    created_at: Utc::now(),
  };

  let user = connection
    .transaction(async |connection| {
      let user: User = diesel::insert_into(users::table)
        .values(user)
        .returning(User::as_returning())
        .get_result(connection)
        .await?;

      let organization: Organization = diesel::insert_into(organizations::table)
        .values(organization)
        .returning(Organization::as_returning())
        .get_result(connection)
        .await?;

      Ok(CreateUserResponse { user, organization })
    })
    .await
    .map_err(|error: diesel::result::Error| {
      tracing::error!(?error, "database insert error");
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  Ok(Json(user))
}

pub async fn delete_user(
  State(AppState { context, .. }): State<AppState>,
  user: AuthenticatedUser,
) -> Result<Json<User>, StatusCode> {
  let mut connection = context.pg.get().await.map_err(|error| {
    tracing::error!(?error, "database connection error");
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  let result: User = diesel::delete(users::table.filter(users::id.eq(user.user)))
    .returning(User::as_returning())
    .get_result(&mut connection)
    .await
    .map_err(|error| match error {
      diesel::result::Error::NotFound => StatusCode::NOT_FOUND,
      _ => {
        tracing::error!(?error, "pg update error");
        StatusCode::INTERNAL_SERVER_ERROR
      }
    })?;

  Ok(Json(result))
}
