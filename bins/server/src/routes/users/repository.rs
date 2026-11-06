use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use flow::users;

use super::models::User;

pub struct UserRepository;

impl UserRepository {
  pub async fn delete_user_by_id(
    connection: &mut AsyncPgConnection,
    user_id: uuid::Uuid,
  ) -> Result<User, diesel::result::Error> {
    diesel::delete(users::table.filter(users::id.eq(user_id)))
      .returning(User::as_returning())
      .get_result(connection)
      .await
  }
}
