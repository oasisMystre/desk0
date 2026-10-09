use anyhow::Context;

use crate::api::Api;

#[derive(Clone)]
pub struct Identity {
  pub org_id: String,
  pub device_id: String,
}

impl Identity {
  pub async fn upsert(api: &Api, db: &rocksdb::DB, org_id: String) -> anyhow::Result<Self> {
    if let Some(identity) = Self::get(db) {
      Ok(identity)
    } else {
      let mut identity = Self {
        org_id: org_id.clone(),
        device_id: uuid::Uuid::new_v4().to_string(),
      };
      let device = api
        .device
        .create_device(&identity.clone().into())
        .await
        .context("failed to create device")?;
      let device_id = device.id.to_string();
      identity.device_id = device_id.clone();
      db.put("org_id", org_id)?;
      db.put("device_id", device_id)?;

      Ok(identity)
    }
  }

  pub fn get(db: &rocksdb::DB) -> Option<Self> {
    let org_id = db
      .get("org_id")
      .ok()
      .and_then(|result| result.and_then(|value| String::from_utf8(value).ok()))?;

    let device_id = db
      .get("device_id")
      .ok()
      .and_then(|result| result.and_then(|value| String::from_utf8(value).ok()))?;

    Some(Self {
      org_id,
      device_id: device_id.clone(),
    })
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
