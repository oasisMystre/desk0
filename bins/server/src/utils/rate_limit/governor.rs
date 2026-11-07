use axum::body::Body;
use governor::{clock::QuantaInstant, middleware::NoOpMiddleware};
use tower_governor::{
  GovernorLayer, governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor,
};

pub fn create_governor_rate_limit(
  per_second: u64,
  burst_size: u32,
) -> GovernorLayer<SmartIpKeyExtractor, NoOpMiddleware<QuantaInstant>, Body> {
  let config = GovernorConfigBuilder::default()
    .key_extractor(SmartIpKeyExtractor)
    .per_second(per_second)
    .burst_size(burst_size)
    .finish()
    .unwrap();

  GovernorLayer::new(config)
}
