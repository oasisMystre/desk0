use std::sync::Arc;

use hmac::{Hmac, Mac};
use sha2::Sha256;
use shared::{
  env::FromEnv,
  init::{init_db_pool, init_redis_pool},
  types::{PgPool, RedisPool},
};

use crate::env::Env;

#[derive(Clone)]
pub struct Context {
  pub pg: Arc<PgPool>,
  #[allow(dead_code)]
  pub redis: Arc<RedisPool>,
  pub auth_key: Arc<Hmac<Sha256>>,
}

impl Context {
  pub async fn from_env() -> anyhow::Result<Self> {
    let pg = Arc::new(init_db_pool(1000).await?);
    let redis = Arc::new(init_redis_pool(1000)?);

    let auth_key = Arc::new(Hmac::new_from_slice(Env::AuthToken.get()?.as_bytes())?);

    Ok(Self {
      pg,
      redis,
      auth_key,
    })
  }
}
