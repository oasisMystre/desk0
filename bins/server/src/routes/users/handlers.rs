use axum::{Json, extract::State};

use super::{error::UserError, models::User, service::UserService};
use crate::states::{AppState, AuthRole, AuthenticatedUser};

pub async fn delete_user(
  State(AppState { context, .. }): State<AppState>,
  user: AuthenticatedUser,
) -> Result<Json<User>, UserError> {
  user.require_role(&[AuthRole::User])?;

  let result = UserService::delete_user(&context, user.user).await?;
  Ok(Json(result))
}
