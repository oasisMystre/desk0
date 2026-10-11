use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::Os"]
#[derive(Serialize, Deserialize)]
#[serde(rename_all="lowercase")]
pub enum AgentOs {
  Windows,
  Macos,
  Linux,
  Android,
  Ios,
}
