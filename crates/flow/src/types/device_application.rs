use chrono::{DateTime, Utc};
use diesel::{HasQuery, prelude::Insertable};
use serde::{Deserialize, Serialize};

use crate::device_applications;

#[derive(HasQuery, Insertable, Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[diesel(table_name = device_applications)]
pub struct DeviceApplication {
  pub id: uuid::Uuid,
  pub device: uuid::Uuid,
  pub application: uuid::Uuid,
  pub organization: uuid::Uuid,
  #[diesel(column_name = "createdAt")]
  pub created_at: DateTime<Utc>,
}
