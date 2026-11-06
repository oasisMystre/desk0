use diesel::{ExpressionMethods, HasQuery, QueryDsl, SelectableHelper};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use flow::{organization::Organization, organizations};

pub struct OrganizationRepository;

impl OrganizationRepository {
  pub async fn find_by_user(
    connection: &mut AsyncPgConnection,
    user_id: uuid::Uuid,
  ) -> Result<Vec<Organization>, diesel::result::Error> {
    Organization::query()
      .filter(organizations::user.eq(user_id))
      .load(connection)
      .await
  }

  pub async fn create(
    connection: &mut AsyncPgConnection,
    organization: &Organization,
  ) -> Result<Organization, diesel::result::Error> {
    diesel::insert_into(organizations::table)
      .values(organization)
      .returning(Organization::as_returning())
      .get_result(connection)
      .await
  }
}
