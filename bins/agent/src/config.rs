use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct AgentConfig {
  pub api_url: String,
  pub tunnel_url: String,
  #[allow(dead_code)]
  pub version: i32,
  #[allow(dead_code)]
  pub agent: uuid::Uuid,
  pub organization: uuid::Uuid,
}

impl AgentConfig {
  #[cfg(not(debug_assertions))]
  pub fn load_config() -> anyhow::Result<Self> {
    use anyhow::Context;
    use std::fs;

    let config_path = Self::get_data_dir().join("config.json");
    let config_content = fs::read_to_string(&config_path)
      .with_context(|| format!("Failed to read config file at {:?}", config_path))?;

    let config: Self =
      serde_json::from_str(&config_content).context("Failed to parse config.json format")?;

    Ok(config)
  }

  #[cfg(debug_assertions)]
  pub fn load_config() -> anyhow::Result<Self> {
    Ok(Self {
      version: 0,
      api_url: "http://172.20.10.4:8080".into(),
      tunnel_url: "ws://172.20.10.4:8080".into(),
      agent: "11111111-1111-1111-1111-111111111111".parse()?,
      organization: "11111111-1111-1111-1111-111111111111".parse()?,
    })
  }

  pub fn get_data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    let base_dir = {
      let program_data =
        std::env::var("ProgramData").unwrap_or_else(|_| r"C:\ProgramData".to_string());
      PathBuf::from(program_data).join("Desk0")
    };

    #[cfg(target_os = "macos")]
    let base_dir = PathBuf::from("/Library/Application Support/Desk0");

    #[cfg(target_os = "linux")]
    let base_dir = PathBuf::from("/var/lib/desk0");

    base_dir
  }

  pub fn get_db_data_dir() -> PathBuf {
    Self::get_data_dir().join("data")
  }
}
