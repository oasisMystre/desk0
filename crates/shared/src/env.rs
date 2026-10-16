use strum::Display;

pub trait FromEnv: ToString + std::fmt::Display {
  fn get(&self) -> anyhow::Result<String> {
    use anyhow::Context;

    let default = format!("APP_{}", self);
    std::env::var(default)
      .or_else(|_| std::env::var(self.to_string()))
      .with_context(|| format!("{} environment variable not found", self))
  }

  fn get_optional(&self) -> Option<String> {
    let default = format!("APP_{}", self);
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
  #[cfg(feature = "sentinel")]
  RedisSentinelUrls,
  #[cfg(feature = "sentinel")]
  RedisMasterName,
  #[cfg(feature = "sentinel")]
  RedisUsername,
  #[cfg(feature = "sentinel")]
  RedisPassword,
  #[cfg(feature = "sentinel")]
  RedisDatabase,
  #[cfg(feature = "opentelemetry")]
  OtelExporterOtlpEndpoint,
}

impl FromEnv for Env {}
