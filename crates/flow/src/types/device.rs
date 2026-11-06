use chrono::{DateTime, Utc};
use diesel::{HasQuery, prelude::Insertable};
use serde::{Deserialize, Serialize};

use crate::schema::devices;

#[derive(HasQuery, Insertable, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[diesel(table_name = devices)]
pub struct Device {
  pub id: uuid::Uuid,
  pub os: Option<String>,
  #[cfg_attr(feature = "utoipa", schema(value_type = Vec<u8>))]
  pub pubkey: Vec<u8>,
  pub hostname: Option<String>,
  pub username: Option<String>,
  pub version: Option<String>,
  pub organization: uuid::Uuid,
  pub metadata: Option<serde_json::Value>,
  #[diesel(column_name = "lastLoggedIn")]
  pub last_logged_in: DateTime<Utc>,
  #[diesel(column_name = "createdAt")]
  pub created_at: DateTime<Utc>,
}
