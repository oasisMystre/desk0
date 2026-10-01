use chrono::{DateTime, Utc};
use diesel::{Selectable, associations::Identifiable, deserialize::Queryable, prelude::Insertable};
use serde::{Deserialize, Serialize};

use crate::users;

#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize, Identifiable, Default, Debug)]
#[diesel(table_name = users)]
pub struct User {
  pub id: uuid::Uuid,
  pub uid: Option<String>,

  #[diesel(column_name = "lastLoggedIn")]
  pub last_logged_in: DateTime<Utc>,
  #[diesel(column_name = "createdAt")]
  pub created_at: DateTime<Utc>,
}
