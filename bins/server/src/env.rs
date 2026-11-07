use shared::env::FromEnv;
use strum::Display;

#[derive(Display)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum Env {
  AuthToken,
  S3Bucket,
  S3Endpoint,
  S3AccessKeyId,
  S3AccessKey,
  ResendApiKey,
}

impl FromEnv for Env {}
