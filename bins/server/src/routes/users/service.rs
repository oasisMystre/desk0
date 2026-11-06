use super::{error::UserError, models::User, repository::UserRepository};
use crate::context::Context;

pub struct UserService;

impl UserService {
  pub async fn delete_user(context: &Context, user_id: uuid::Uuid) -> Result<User, UserError> {
    let mut connection = context.get_pg().await?;

    let result = UserRepository::delete_user_by_id(&mut connection, user_id)
      .await
      .map_err(|error| match error {
        diesel::result::Error::NotFound => UserError::NotFound,
        _ => {
          tracing::error!(?error, "pg update error");
          UserError::DeleteFailed
        }
      })?;

    Ok(result)
  }
}
