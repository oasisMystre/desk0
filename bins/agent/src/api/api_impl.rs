use std::sync::Arc;

use reqwest::{Client, Url, header::HeaderMap};

pub trait Api: Sized {
  fn new(api_client: Arc<ApiClient>) -> Self;
}

#[derive(Clone)]
pub struct ApiClient {
  pub client: Client,
  pub base_url: Url,
  pub headers: Option<HeaderMap>,
}

impl ApiClient {
  pub fn new(base_url: &str, headers: Option<HeaderMap>) -> Self {
    Self {
      headers,
      client: Client::new(),
      base_url: Url::parse(base_url).unwrap(),
    }
  }
}
