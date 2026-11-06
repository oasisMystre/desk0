use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::Os"]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[serde(rename_all = "lowercase")]
pub enum AgentOs {
  Windows,
  Macos,
  Linux,
  Android,
  Ios,
}

impl std::str::FromStr for AgentOs {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "windows" => Ok(AgentOs::Windows),
      "macos" => Ok(AgentOs::Macos),
      "linux" => Ok(AgentOs::Linux),
      "android" => Ok(AgentOs::Android),
      "ios" => Ok(AgentOs::Ios),
      other => Err(format!("Unknown OS: {}", other)),
    }
  }
}

impl std::fmt::Display for AgentOs {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      AgentOs::Windows => write!(f, "windows"),
      AgentOs::Macos => write!(f, "macos"),
      AgentOs::Linux => write!(f, "linux"),
      AgentOs::Android => write!(f, "android"),
      AgentOs::Ios => write!(f, "ios"),
    }
  }
}
