use chrono::{DateTime, Utc};
use diesel::query_builder::AsChangeset;
use serde::{Deserialize, Serialize};

pub use flow::{device::Device, devices};

#[derive(Deserialize, Serialize, AsChangeset)]
#[diesel(table_name = devices)]
pub struct UpdateDeviceChangeset {
  pub os: Option<String>,
  pub hostname: Option<String>,
  pub username: Option<String>,
  pub version: Option<String>,
  pub metadata: Option<serde_json::Value>,
  #[diesel(column_name = "lastLoggedIn")]
  pub last_logged_in: Option<DateTime<Utc>>,
}
