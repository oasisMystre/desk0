use diesel::query_builder::AsChangeset;
pub use flow::{attachment::Attachment, attachments};
use serde::Deserialize;

#[derive(Deserialize, AsChangeset)]
#[diesel(table_name = attachments)]
pub struct UpdateAttachmentChangeset {
  pub uploaded: Option<bool>,
  pub filename: Option<String>,
  pub metadata: Option<serde_json::Value>,
}
