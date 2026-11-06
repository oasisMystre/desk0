use axum::{Json, extract::State};

use super::{error::UserError, models::User, service::UserService};
use crate::{
  error::ErrorResponse,
  states::{AppState, AuthRole, AuthenticatedUser},
};

#[utoipa::path(
  delete,
  path = "/user",
  responses(
    (status = 200, description = "User deleted successfully", body = User),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  security(
    ("bearer_auth" = [])
  ),
  tag = "users"
)]
pub async fn delete_user(
  State(AppState { context, .. }): State<AppState>,
  user: AuthenticatedUser,
) -> Result<Json<User>, UserError> {
  user.require_role(&[AuthRole::User])?;

  let result = UserService::delete_user(&context, user.user).await?;
  Ok(Json(result))
}
