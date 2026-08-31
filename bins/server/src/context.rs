use std::sync::Arc;

use crate::types::{PgPool, RedisPool};

pub struct Context {
  pub pg: Arc<PgPool>,
  pub redis: Arc<RedisPool>,
}

impl Context {}
