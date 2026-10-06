use std::sync::Arc;

use crate::api::{
  api_impl::{Api as _, ApiClient},
  device::DeviceApi,
};

mod api_impl;
pub mod channel;
pub mod device;

pub struct Api {
  pub device: DeviceApi,
}

#[derive(Default)]
pub struct ApiConfig<'a> {
  pub base_url: &'a str,
}

impl Api {
  pub fn from_config<'a>(config: ApiConfig<'a>) -> Self {
    let api_client = Arc::new(ApiClient::new(config.base_url, None));
    let device = DeviceApi::new(api_client);

    Self { device }
  }
}
