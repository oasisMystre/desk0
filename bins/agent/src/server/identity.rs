#[derive(Clone)]
pub struct Identity {
  pub org_id: String,
  pub device_id: String,
}

impl Identity {
  pub fn upsert(db: &rocksdb::DB, org_id: String) -> anyhow::Result<Self> {
    let device_id = if let Some(device_id) = db
      .get("device_id")
      .map(|result| result.and_then(|value| String::from_utf8(value).ok()))?
    {
      device_id
    } else {
      let device_id = uuid::Uuid::new_v4().to_string();
      db.put("device_id", device_id.clone())?;
      device_id
    };

    Ok(Self { org_id, device_id })
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
}
