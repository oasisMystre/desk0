use strum::Display;

pub trait FromEnv: ToString + std::fmt::Display {
  fn get(&self) -> anyhow::Result<String> {
    use anyhow::Context;

    let default = format!("APP_{}", self.to_string());
    std::env::var(default)
      .or_else(|_| std::env::var(self.to_string()))
      .with_context(|| format!("{} environment variable not found", self))
  }

  fn get_optional(&self) -> Option<String> {
    let default = format!("APP_{}", self.to_string());
    std::env::var(default)
      .or_else(|_| std::env::var(self.to_string()))
      .ok()
  }
}

#[derive(Display)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum Env {
  DatabaseUrl,
  RedisUrl,
  RedisSentinelUrls,
  RedisMasterName,
  RedisUsername,
  RedisPassword,
  RedisDatabase,
  ClickhouseUrl,
  ClickhouseUser,
  ClickhousePassword,
  ClickhouseDatabase,
  OtelExporterOtlpEndpoint,
}

impl FromEnv for Env {}
