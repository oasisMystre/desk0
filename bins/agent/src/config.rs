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
      agent: "dc0308c0-29b2-47e9-952c-286ab65dc8fe".parse()?,
      organization: "00000000-0000-0000-0000-000000000000".parse()?,
    })
  }

  #[cfg(debug_assertions)]
  pub fn get_data_dir() -> PathBuf {
    std::env::current_dir().unwrap()
  }

  #[cfg(not(debug_assertions))]
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

  #[cfg(feature = "ipc")]
  pub fn get_ipc_share_memory_path() -> String {
    let path = Self::get_data_dir()
      .join("desk0")
      .to_string_lossy()
      .into_owned();

    path
  }

  #[cfg(feature = "ipc")]
  pub fn get_ipc_path<'a>() -> anyhow::Result<ipc::Name<'a>> {
    #[cfg(unix)]
    {
      use ipc::{GenericFilePath, ToFsName};

      let path = Self::get_data_dir()
        .join("desk0.sock")
        .to_string_lossy()
        .into_owned();

      Ok(path.to_fs_name::<GenericFilePath>()?)
    }

    #[cfg(windows)]
    {
      use ipc::{GenericNamespaced, ToNsName};

      let name = r"\\.\pipe\desk0".to_owned();

      Ok(name.to_ns_name::<GenericNamespaced>()?)
    }
  }
}
