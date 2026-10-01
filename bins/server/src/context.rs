use std::sync::Arc;

use aws_config::Region;
use aws_sdk_s3::config::Credentials;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use shared::{
  env::FromEnv,
  init::{init_db_pool, init_redis, init_redis_pool},
  types::{PgPool, RedisPool},
};

use crate::env::Env;

#[derive(Clone)]
pub struct Context {
  pub pg: Arc<PgPool>,
  pub redis_pool: Arc<RedisPool>,
  pub s3: Arc<aws_sdk_s3::Client>,
  pub auth_key: Arc<Hmac<Sha256>>,
  pub redis: Arc<deadpool_redis::redis::Client>,
}

impl Context {
  pub async fn from_env() -> anyhow::Result<Self> {
    let pg = Arc::new(init_db_pool(1000).await?);
    let s3 = Arc::new(Self::build_s3_from_env()?);

    let redis = Arc::new(init_redis()?);
    let redis_pool = Arc::new(init_redis_pool(1000)?);
    let auth_key = Arc::new(Hmac::new_from_slice(Env::AuthToken.get()?.as_bytes())?);

    Ok(Self {
      s3,
      pg,
      redis,
      auth_key,
      redis_pool,
    })
  }

  fn build_s3_from_env() -> anyhow::Result<aws_sdk_s3::client::Client> {
    let config = aws_sdk_s3::Config::builder()
      .endpoint_url(Env::S3Endpoint.get()?)
      .region(Region::new("auto"))
      .credentials_provider(Credentials::new(
        Env::S3AccessKeyId.get()?,
        Env::S3AccessKey.get()?,
        None,
        None,
        "r2",
      ))
      .behavior_version_latest()
      .build();

    Ok(aws_sdk_s3::Client::from_conf(config))
  }
}
