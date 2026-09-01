use std::sync::Arc;

use shared::{
  init::{init_db_pool, init_redis_pool},
  types::{PgPool, RedisPool},
};

#[derive(Clone)]
pub struct Context {
  pub pg: Arc<PgPool>,
  pub redis: Arc<RedisPool>,
}

impl Context {
  pub async fn from_env() -> anyhow::Result<Self> {
    let pg = Arc::new(init_db_pool(1000).await?);
    let redis = Arc::new(init_redis_pool(1000)?);

    Ok(Self { pg, redis })
  }
}
