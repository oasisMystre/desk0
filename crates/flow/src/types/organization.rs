use chrono::{DateTime, Utc};
use diesel::{HasQuery, associations::Identifiable, prelude::Insertable};
use serde::{Deserialize, Serialize};

use crate::schema::organizations;

#[derive(HasQuery, Insertable, Serialize, Deserialize, Identifiable, Default)]
#[diesel(table_name = organizations)]
pub struct Organization {
  pub id: uuid::Uuid,
  pub name: Option<String>,
  pub user: uuid::Uuid,
  #[diesel(column_name = "createdAt")]
  pub created_at: DateTime<Utc>,
}
