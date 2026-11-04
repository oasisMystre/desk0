use chrono::{DateTime, Utc};
use diesel::{HasQuery, prelude::Insertable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{agent_assets, agents, os::AgentOs};

#[derive(HasQuery, Insertable, Serialize, Deserialize, Clone, Debug)]
#[diesel(table_name = agents)]
pub struct Agent {
  pub id: Uuid,
  pub organization: Uuid,
  pub version: i32,
  #[diesel(column_name = "createdAt")]
  pub created_at: DateTime<Utc>,
}

#[derive(HasQuery, Insertable, Serialize, Deserialize, Clone, Debug)]
#[diesel(table_name = agent_assets)]
pub struct AgentAsset {
  pub agent: Uuid,
  pub os: AgentOs,
  pub arch: String,
  pub version: i32,
  pub attachment: Uuid,
}
