use std::path::PathBuf;

use anyhow::Context;
use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};

use crate::{api::Api, config::AgentConfig};

#[derive(Clone, Deserialize, Serialize)]
pub struct Identity {
  pub os: String,
  pub hostname: Option<String>,
  pub username: Option<String>,
  pub pubkey: Vec<u8>,
  pub org_id: uuid::Uuid,
  pub device_id: uuid::Uuid,
  #[serde(skip)]
  signing_key: Option<SigningKey>,
}

impl Identity {
  fn state_path() -> PathBuf {
    AgentConfig::get_data_dir().join("identity.json")
  }

  pub fn signing_key(&self) -> &SigningKey {
    self
      .signing_key
      .as_ref()
      .expect("signing key to be initialized")
  }

  pub async fn upsert(api: &Api, org_id: uuid::Uuid) -> anyhow::Result<Self> {
    let username = Self::username();
    let username = username.as_deref().unwrap_or("unknown");
    let entry = Self::get_entry(username)?;

    if let Some(mut identity) = Self::load()? {
      anyhow::ensure!(
        identity.org_id == org_id,
        "stored device identity belongs to a different organization"
      );

      let key = Self::get_signing_key(&entry)?;
      let pubkey = key.verifying_key().as_bytes().to_vec();

      anyhow::ensure!(
        identity.pubkey == pubkey,
        "stored identity public key does not match the device signing key"
      );

      identity.signing_key = Some(key);

      Ok(identity)
    } else {
      let mut identity = Self {
        org_id,
        pubkey: vec![],
        os: Self::os(),
        signing_key: None,
        hostname: Self::hostname(),
        username: Self::username(),
        device_id: uuid::Uuid::new_v4(),
      };

      let key = Self::upsert_signing_key(&entry)?;
      identity.pubkey = key.verifying_key().as_bytes().to_vec();
      identity.signing_key = Some(key);

      let device = api
        .device
        .create_device(&identity.clone().into())
        .await
        .context("failed to create device")?;

      identity.device_id = device.id;
      identity.save()?;

      Ok(identity)
    }
  }

  fn save(&self) -> anyhow::Result<()> {
    let path = Self::state_path();
    std::fs::create_dir_all(
      path
        .parent()
        .context("identity path has no parent directory")?,
    )
    .context("create identity directory")?;

    let json = serde_json::to_string_pretty(self)?;
    std::fs::write(Self::state_path(), json).context("Failed to save identity state")?;

    Ok(())
  }

  fn upsert_signing_key(entry: &keyring::Entry) -> anyhow::Result<SigningKey> {
    match Self::get_signing_key(&entry) {
      Ok(entry) => Ok(entry),
      Err(error)
        if matches!(
          error.downcast_ref::<keyring::Error>(),
          Some(&keyring::Error::NoEntry)
        ) =>
      {
        Self::create_signing_key(&entry)
      }
      Err(error) => return Err(error).context("read device signing key"),
    }
  }

  fn load() -> anyhow::Result<Option<Self>> {
    let path = Self::state_path();

    match std::fs::read_to_string(path) {
      Ok(content) => Ok(Some(
        serde_json::from_str(&content).context("parse identity state")?,
      )),
      Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
      Err(error) => Err(error).context("read identity state"),
    }
  }

  fn hostname() -> Option<String> {
    std::env::var("COMPUTERNAME").ok()
  }

  fn username() -> Option<String> {
    std::env::var("USERNAME").ok()
  }

  fn os() -> String {
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

  fn get_entry(username: &str) -> anyhow::Result<keyring::Entry> {
    let entry =
      keyring::Entry::new("com.desk0.agent", username).context("open device key store")?;

    Ok(entry)
  }

  fn create_signing_key(entry: &keyring::Entry) -> anyhow::Result<SigningKey> {
    let mut rng = rand_core::UnwrapErr(rand::rngs::SysRng);
    let key = SigningKey::generate(&mut rng);
    entry
      .set_secret(&key.to_bytes())
      .context("save device signing key")?;

    Ok(key)
  }

  fn get_signing_key(entry: &keyring::Entry) -> anyhow::Result<SigningKey> {
    let bytes = entry.get_secret().context("load device signing key")?;
    let bytes: [u8; 32] = bytes
      .try_into()
      .map_err(|_| anyhow::anyhow!("invalid device signing key length"))?;

    Ok(SigningKey::from_bytes(&bytes))
  }
}
