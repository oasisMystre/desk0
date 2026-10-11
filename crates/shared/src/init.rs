#[cfg(feature = "postgres")]
use diesel_async::{
  AsyncPgConnection,
  pooled_connection::{AsyncDieselConnectionManager, bb8::Pool},
};
#[cfg(feature = "opentelemetry")]
use opentelemetry_sdk::Resource;
#[cfg(feature = "opentelemetry")]
use opentelemetry_sdk::{
  logs::SdkLoggerProvider, metrics::SdkMeterProvider, trace::SdkTracerProvider,
};
#[cfg(feature = "otel")]
use tracing_error::ErrorLayer;
#[cfg(feature = "otel")]
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};
#[cfg(feature = "otel")]
use tracing_throttle::TracingRateLimitLayer;

#[cfg(any(feature = "opentelemetry", feature = "postgres", feature = "redis"))]
use crate::env::{Env, FromEnv};

#[cfg(feature = "postgres")]
pub async fn init_db_pool(max_size: u32) -> anyhow::Result<Pool<AsyncPgConnection>> {
  let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(Env::DatabaseUrl.get()?);

  Pool::builder()
    .max_size(max_size)
    .build(manager)
    .await
    .map_err(|error| anyhow::anyhow!(error))
}

#[cfg(feature = "opentelemetry")]
fn get_resource(name: &str) -> Resource {
  let resource = Resource::builder()
    .with_service_name(name.to_owned())
    .build();
  resource
}

#[cfg(feature = "opentelemetry")]
type InitTracingResult = (
  Option<SdkTracerProvider>,
  Option<SdkMeterProvider>,
  Option<SdkLoggerProvider>,
);

#[cfg(feature = "otel")]
#[cfg(not(feature = "opentelemetry"))]
type InitTracingResult = ();

#[cfg(feature = "otel")]
pub fn init_tracing(
  #[cfg(feature = "opentelemetry")] resource_name: &str,
) -> anyhow::Result<InitTracingResult> {
  let rate_limit = TracingRateLimitLayer::new();

  #[cfg(feature = "opentelemetry")]
  {
    use opentelemetry::{global, trace::TracerProvider};
    use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
    use opentelemetry_otlp::{LogExporter, MetricExporter, SpanExporter, WithExportConfig};
    use tracing_opentelemetry::OpenTelemetryLayer;

    let exporter = LogExporter::builder()
      .with_tonic()
      .with_endpoint(Env::OtelExporterOtlpEndpoint.get()?)
      .build()?;
    let logger_provider = SdkLoggerProvider::builder()
      .with_batch_exporter(exporter)
      .with_resource(get_resource(resource_name))
      .build();

    let otel_layer = OpenTelemetryTracingBridge::new(&logger_provider);
    let filter_otel = EnvFilter::new("info")
      .add_directive("hyper=warn".parse()?)
      .add_directive("tonic=warn".parse()?)
      .add_directive("h2=warn".parse()?)
      .add_directive("mio=warn".parse()?)
      .add_directive("reqwest=warn".parse()?);

    let otel_layer = otel_layer.with_filter(filter_otel);

    let filter_fmt = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let fmt_layer = tracing_subscriber::fmt::layer()
      .with_thread_names(true)
      .with_file(true)
      .with_target(true)
      .with_line_number(true)
      .with_filter(filter_fmt);

    let register = tracing_subscriber::registry()
      .with(otel_layer)
      .with(fmt_layer)
      .with(rate_limit)
      .with(ErrorLayer::default());

    let exporter = SpanExporter::builder()
      .with_tonic()
      .with_endpoint(Env::OtelExporterOtlpEndpoint.get()?)
      .build()?;

    let tracing_provider = SdkTracerProvider::builder()
      .with_batch_exporter(exporter)
      .with_resource(get_resource(resource_name))
      .build();

    global::set_tracer_provider(tracing_provider.clone());

    let tracer = tracing_provider.tracer(resource_name.to_string());
    let otel_trace_layer = OpenTelemetryLayer::new(tracer);

    let register = register.with(otel_trace_layer);
    let exporter = MetricExporter::builder()
      .with_tonic()
      .with_endpoint(Env::OtelExporterOtlpEndpoint.get()?)
      .build()?;

    let meter_provider = SdkMeterProvider::builder()
      .with_periodic_exporter(exporter)
      .with_resource(get_resource(resource_name))
      .build();

    global::set_meter_provider(meter_provider.clone());

    register.init();

    Ok((
      Some(tracing_provider),
      Some(meter_provider),
      Some(logger_provider),
    ))
  }

  #[cfg(not(feature = "opentelemetry"))]
  {
    tracing_log::LogTracer::init()?;

    let filter_otel = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer()
      .with_thread_names(true)
      .with_file(true)
      .with_line_number(true)
      .with_filter(filter_otel);

    let subscriber = tracing_subscriber::registry()
      .with(fmt_layer)
      .with(rate_limit);

    subscriber.with(ErrorLayer::default()).try_init().ok();

    Ok(())
  }
}

#[cfg(feature = "sentinel")]
pub fn init_redis_pool(max_size: usize) -> anyhow::Result<deadpool_redis::sentinel::Pool> {
  use anyhow::Context as _;

  let master_name = Env::RedisMasterName.get_optional();
  let sentinel_urls = Env::RedisSentinelUrls.get_optional();

  if let (Some(sentinel_urls), Some(master_name)) = (sentinel_urls, master_name) {
    let sentinel_urls: Vec<String> = sentinel_urls
      .split(",")
      .map(str::trim)
      .filter(|value| !value.is_empty())
      .map(String::from)
      .collect();

    let config = deadpool_redis::sentinel::Config::from_urls(
      sentinel_urls,
      master_name,
      deadpool_redis::sentinel::SentinelServerType::Master,
    );

    let mut config = config.with_node_connection_info(Some(
      deadpool_redis::sentinel::SentinelNodeConnectionInfo {
        tls_mode: None,
        redis_connection_info: Some(deadpool_redis::RedisConnectionInfo {
          username: Env::RedisUsername.get_optional(),
          password: Env::RedisPassword.get_optional(),
          protocol: deadpool_redis::ProtocolVersion::RESP2,
          db: Env::RedisDatabase
            .get_optional()
            .and_then(|value| value.parse::<i64>().ok())
            .unwrap_or(0),
        }),
      },
    ));

    config.pool = Some(deadpool_redis::PoolConfig::new(max_size));

    config
      .create_pool(None)
      .context("init sentinel pool failed")
  } else {
    Err(anyhow::anyhow!("invalid sentinel config"))
  }
}

#[cfg(feature = "redis")]
#[cfg(not(feature = "sentinel"))]
pub fn init_redis_pool(max_size: usize) -> anyhow::Result<deadpool_redis::Pool> {
  use anyhow::Context as _;
  let url = Env::RedisUrl.get()?;
  
  println!("Redis URL: {}", url);
  
  let mut config = deadpool_redis::Config::from_url(url);
  config.pool = Some(deadpool_redis::PoolConfig::new(max_size));
  config
    .create_pool(Some(deadpool_redis::Runtime::Tokio1))
    .context("init redis pool failed")
}

#[cfg(feature = "redis")]
#[cfg(not(feature = "sentinel"))]
pub fn init_redis() -> anyhow::Result<deadpool_redis::redis::Client> {
  use anyhow::Context as _;

  let url = Env::RedisUrl.get()?;
  
  println!("Redis URL: {}", url);

  let client = deadpool_redis::redis::Client::open(url)
    .context("init redis client failed")?;

  Ok(client)
}

pub fn load_env() {
  let workspace_env = std::env::current_dir()
    .ok()
    .and_then(|path| path.ancestors().nth(2).map(|root| root.join(".env")));

  if let Some(path) = workspace_env {
    dotenvy::from_path(path).ok();
  }

  dotenvy::from_path(".env").ok();
}
