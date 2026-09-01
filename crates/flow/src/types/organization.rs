use crate::schema::organizations;
use chrono::{DateTime, Utc};
use diesel::{HasQuery, prelude::Insertable};
use serde::{Deserialize, Serialize};

#[derive(HasQuery, Insertable, Serialize, Deserialize, Default)]
#[diesel(table_name = organizations)]
pub struct Organization {
  pub id: uuid::Uuid,
  pub uid: Option<String>,
  pub name: Option<String>,
  #[diesel(column_name = "createdAt")]
  pub created_at: DateTime<Utc>,
}
