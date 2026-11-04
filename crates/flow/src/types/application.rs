use chrono::{DateTime, Utc};
use diesel::{HasQuery, prelude::Insertable};
use serde::{Deserialize, Serialize};

use crate::{applications, os::AgentOs};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ScriptArgs {
  pub args: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct Scripts {
  #[serde(default)]
  pub _version: Option<u64>,
  pub run: Option<ScriptArgs>,
  pub install: Option<ScriptArgs>,
  pub uninstall: Option<ScriptArgs>,
}

#[derive(HasQuery, Insertable, Serialize, Deserialize, Clone, Debug)]
#[diesel(table_name = applications)]
pub struct Application {
  pub id: uuid::Uuid,
  pub name: String,
  pub os: AgentOs,
  pub image: Option<uuid::Uuid>,
  pub asset: uuid::Uuid,
  pub organization: uuid::Uuid,
  pub scripts: serde_json::Value,
  #[diesel(column_name = "createdAt")]
  pub created_at: DateTime<Utc>,
}

impl Application {
  pub fn scripts(&self) -> Result<Scripts, serde_json::Error> {
    serde_json::from_value::<Scripts>(self.scripts.clone())
  }
}
