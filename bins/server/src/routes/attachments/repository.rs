use diesel::{
  BoolExpressionMethods, ExpressionMethods, HasQuery, QueryDsl, SelectableHelper,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use flow::{attachment::Attachment, attachments};

use super::models::UpdateAttachmentChangeset;

pub struct AttachmentRepository;

impl AttachmentRepository {
  pub async fn find_by_organization(
    connection: &mut AsyncPgConnection,
    org_id: uuid::Uuid,
  ) -> Result<Vec<Attachment>, diesel::result::Error> {
    Attachment::query()
      .filter(attachments::organization.eq(org_id))
      .load(connection)
      .await
  }

  pub async fn create(
    connection: &mut AsyncPgConnection,
    attachment: &Attachment,
  ) -> Result<(), diesel::result::Error> {
    diesel::insert_into(attachments::table)
      .values(attachment)
      .execute(connection)
      .await?;
    Ok(())
  }

  pub async fn update(
    connection: &mut AsyncPgConnection,
    file_id: uuid::Uuid,
    org_id: uuid::Uuid,
    changeset: &UpdateAttachmentChangeset,
  ) -> Result<Attachment, diesel::result::Error> {
    diesel::update(
      attachments::table.filter(
        attachments::id
          .eq(file_id)
          .and(attachments::organization.eq(org_id)),
      ),
    )
    .set(changeset)
    .returning(Attachment::as_returning())
    .get_result(connection)
    .await
  }

  pub async fn delete(
    connection: &mut AsyncPgConnection,
    file_id: uuid::Uuid,
    org_id: uuid::Uuid,
  ) -> Result<Attachment, diesel::result::Error> {
    diesel::delete(
      attachments::table.filter(
        attachments::id
          .eq(file_id)
          .and(attachments::organization.eq(org_id)),
      ),
    )
    .returning(Attachment::as_returning())
    .get_result(connection)
    .await
  }
}
