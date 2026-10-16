use std::path::PathBuf;

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{api::Api, config::AgentConfig};

#[derive(Clone, Deserialize, Serialize)]
pub struct Identity {
  pub org_id: uuid::Uuid,
  pub device_id: uuid::Uuid,
}

impl Identity {
  fn state_path() -> PathBuf {
    AgentConfig::get_data_dir().join("identity.json")
  }

  pub async fn upsert(api: &Api, org_id: uuid::Uuid) -> anyhow::Result<Self> {
    if let Some(identity) = Self::get() {
      Ok(identity)
    } else {
      let mut identity = Self {
        org_id,
        device_id: uuid::Uuid::new_v4(),
      };

      let device = api
        .device
        .create_device(&identity.clone().into())
        .await
        .context("failed to create device")?;

      identity.device_id = device.id;

      let json_string = serde_json::to_string_pretty(&identity)?;
      std::fs::write(Self::state_path(), json_string).context("Failed to save identity state")?;

      Ok(identity)
    }
  }

  pub fn get() -> Option<Self> {
    let content = std::fs::read_to_string(Self::state_path()).ok()?;
    serde_json::from_str(&content).ok()
  }

  pub fn hostname(&self) -> Option<String> {
    std::env::var("COMPUTERNAME").ok()
  }

  pub fn username(&self) -> Option<String> {
    std::env::var("USERNAME").ok()
  }

  pub fn os(&self) -> String {
    std::env::consts::OS.to_string()
  }

  pub fn version(&self) -> String {
    env!("CARGO_PKG_VERSION").to_string()
  }

  pub fn metadata(&self) -> Option<serde_json::Value> {
    Some(serde_json::json!({
        "_version": 1,
        // "width": width,
        // "height": height,
    }))
  }
}
