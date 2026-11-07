use std::{sync::Arc, time::Duration};

use axum::{
  http::HeaderMap,
  response::{IntoResponse, Response},
};
use deadpool_redis::redis::{RedisResult, Script};
use futures_util::future::BoxFuture;

use crate::{context::Context, utils::rate_limit::error::RateLimitError};

#[derive(Clone, Debug)]
pub struct RateLimitConfig {
  pub limit: u64,
  pub prefix: &'static str,
  pub window: Duration,
}

#[derive(Clone)]
pub struct RateLimitLayer {
  context: Arc<Context>,
  config: RateLimitConfig,
}

impl RateLimitLayer {
  pub fn new(context: Arc<Context>, config: RateLimitConfig) -> Self {
    Self { context, config }
  }
}

impl<S> tower::Layer<S> for RateLimitLayer {
  type Service = RateLimitService<S>;

  fn layer(&self, inner: S) -> Self::Service {
    RateLimitService {
      inner,
      context: self.context.clone(),
      config: self.config.clone(),
    }
  }
}

#[derive(Clone)]
pub struct RateLimitService<S> {
  inner: S,
  context: Arc<Context>,
  config: RateLimitConfig,
}

impl<S> RateLimitService<S> {
  const RATE_LIMIT_SCRIPT: &str = r#"
      local count = redis.call('INCR', KEYS[1])
      if count == 1 then
          redis.call('PEXPIRE', KEYS[1], ARGV[1])
      end

      local ttl = redis.call('PTTL', KEYS[1])

      return { count, ttl }
    "#;
}

impl<S> tower::Service<axum::extract::Request> for RateLimitService<S>
where
  S: tower::Service<axum::extract::Request, Response = Response> + Clone + Send + 'static,
  S::Future: Send + 'static,
{
  type Error = S::Error;
  type Response = Response;
  type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

  fn poll_ready(
    &mut self,
    context: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Result<(), Self::Error>> {
    self.inner.poll_ready(context)
  }

  fn call(&mut self, request: axum::extract::Request) -> Self::Future {
    let clone = self.inner.clone();
    let mut inner = std::mem::replace(&mut self.inner, clone);
    let context = self.context.clone();
    let config = self.config.clone();

    Box::pin(async move {
      let ip = extract_ip(request.headers())
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_owned());

      let key = format!("{}:{}", config.prefix, ip);
      let mut redis = match context.get_redis().await {
        Ok(value) => value,
        Err(error) => return Ok(error.into_response()),
      };

      let result: RedisResult<(u64, i64)> = Script::new(Self::RATE_LIMIT_SCRIPT)
        .key(&key)
        .arg(config.window.as_millis() as u64)
        .invoke_async(&mut redis)
        .await;

      let (count, ttl) = match result {
        Ok(result) => result,
        Err(error) => {
          tracing::error!(?error, "rate limit redis error");
          return Ok(RateLimitError::Unavailable.into_response());
        }
      };

      if count > config.limit {
        let retry_after = (ttl.max(0) as u64).div_ceil(1000);
        return Ok(RateLimitError::TooManyRequests(retry_after).into_response());
      }

      inner.call(request).await
    })
  }
}

pub fn extract_ip(headers: &HeaderMap) -> Option<ipnet::IpNet> {
  let ip = headers
    .get("x-forwarded-for")
    .and_then(|header| header.to_str().ok())
    .and_then(|value| value.split(',').next())
    .map(|value| value.trim())
    .or_else(|| {
      headers
        .get("x-real-ip")
        .and_then(|header| header.to_str().ok())
        .map(|value| value.trim())
    });

  ip.and_then(|value| value.parse::<std::net::IpAddr>().ok())
    .map(|ip| ip.into())
}
