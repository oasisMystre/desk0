use chrono::{DateTime, Utc};
use diesel::{HasQuery, prelude::Insertable};
use serde::{Deserialize, Serialize};

use crate::attachments;

#[derive(HasQuery, Insertable, Serialize, Deserialize, Default, Clone, Debug)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[diesel(table_name = attachments)]
pub struct Attachment {
  pub id: uuid::Uuid,
  pub key: String,
  pub filename: String,
  pub uploaded: bool,
  pub organization: uuid::Uuid,
  #[diesel(column_name = "contentType")]
  pub content_type: String,
  pub metadata: Option<serde_json::Value>,
  #[diesel(column_name = "createdAt")]
  pub created_at: DateTime<Utc>,
}
