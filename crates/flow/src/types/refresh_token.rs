use chrono::{DateTime, Utc};
use diesel::{prelude::Insertable, Selectable, associations::Identifiable, deserialize::Queryable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::refresh_tokens;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, Insertable)]
#[diesel(table_name = refresh_tokens)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RefreshToken {
  pub id: Uuid,
  pub session: Uuid,
  pub token: Vec<u8>,
  #[diesel(column_name = "expiredAt")]
  pub expires_at: DateTime<Utc>,
  #[diesel(column_name = "createdAt")]
  pub created_at: DateTime<Utc>,
  #[diesel(column_name = "consumedAt")]
  pub consumed_at: Option<DateTime<Utc>>,
  #[diesel(column_name = "revokedAt")]
  pub revoked_at: Option<DateTime<Utc>>,
}
