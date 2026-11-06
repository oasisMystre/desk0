use chrono::{DateTime, Utc};
use diesel::{Selectable, associations::Identifiable, deserialize::Queryable, prelude::Insertable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::sessions;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, Insertable)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[diesel(table_name = sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Session {
  pub id: Uuid,
  pub user: Uuid,
  #[diesel(column_name = "userAgent")]
  pub user_agent: Option<String>,
  #[diesel(column_name = "ipAddress")]
  #[cfg_attr(feature = "utoipa", schema(value_type = Option<String>))]
  pub ip_address: Option<ipnet::IpNet>,
  #[diesel(column_name = "createdAt")]
  pub created_at: DateTime<Utc>,
  #[diesel(column_name = "expiresAt")]
  pub expires_at: DateTime<Utc>,
  #[diesel(column_name = "revokedAt")]
  pub revoked_at: Option<DateTime<Utc>>,
  #[diesel(column_name = "lastUsedAt")]
  pub last_used_at: Option<DateTime<Utc>>,
}
