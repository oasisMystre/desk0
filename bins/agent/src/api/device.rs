use std::sync::Arc;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
  api::api_impl::{Api, ApiClient},
  server::identity::Identity,
};

pub struct DeviceApi {
  client: Arc<ApiClient>,
}

#[derive(Serialize, Deserialize)]
pub struct Device {
  pub id: uuid::Uuid,
  pub os: Option<String>,
  pub hostname: Option<String>,
  pub username: Option<String>,
  pub version: Option<String>,
  pub organization: uuid::Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct CreateDevice {
  pub os: Option<String>,
  pub hostname: Option<String>,
  pub username: Option<String>,
  pub version: Option<String>,
  pub organization: uuid::Uuid,
  pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateDevice {
  pub organization: uuid::Uuid,
  pub os: Option<String>,
  pub hostname: Option<String>,
  pub username: Option<String>,
  pub version: Option<String>,
  pub metadata: Option<serde_json::Value>,
}

impl From<Identity> for CreateDevice {
  fn from(value: Identity) -> Self {
    Self {
      os: Some(value.os()),
      metadata: value.metadata(),
      hostname: value.hostname(),
      username: value.username(),
      version: Some(value.version()),
      organization: value.org_id.parse::<Uuid>().unwrap(),
    }
  }
}

impl From<Identity> for UpdateDevice {
  fn from(value: Identity) -> Self {
    Self {
      os: Some(value.os()),
      metadata: value.metadata(),
      hostname: value.hostname(),
      username: value.username(),
      version: Some(value.version()),
      organization: value.org_id.parse::<Uuid>().unwrap(),
    }
  }
}

impl Api for DeviceApi {
  fn new(client: Arc<ApiClient>) -> Self {
    Self { client }
  }
}

impl DeviceApi {
  pub async fn create_device(&self, payload: &CreateDevice) -> anyhow::Result<Device> {
    let mut request = self
      .client
      .client
      .post(self.client.base_url.join("devices")?)
      .json(&payload);

    if let Some(ref headers) = self.client.headers {
      request = request.headers(headers.clone());
    }

    let request = request.send().await?;
    let status = request.status();

    if status != StatusCode::OK {
      let response = request.text().await?;
      anyhow::bail!(response)
    }

    let response = request.json::<Device>().await?;

    Ok(response)
  }

  #[allow(dead_code)]
  pub async fn update_device(
    &self,
    device_id: &str,
    payload: &UpdateDevice,
  ) -> anyhow::Result<Device> {
    let mut request = self
      .client
      .client
      .post(
        self
          .client
          .base_url
          .join(&format!("devices/{}", device_id))?,
      )
      .json(&payload);

    if let Some(ref headers) = self.client.headers {
      request = request.headers(headers.clone());
    }

    let request = request.send().await?;

    if request.status() != StatusCode::OK {
      let response = request.text().await?;
      anyhow::bail!(response)
    }

    let response = request.json::<Device>().await?;

    Ok(response)
  }

  pub async fn get_token(&self, device_id: &str) -> anyhow::Result<String> {
    let mut request = self
      .client
      .client
      .post(
        self
          .client
          .base_url
          .join(&format!("devices/token/{device_id}"))?,
      )
      .json(&serde_json::json!({"role": "agent"}));

    if let Some(ref headers) = self.client.headers {
      request = request.headers(headers.clone());
    }

    let request = request.send().await?;

    if request.status() != StatusCode::OK {
      let response = request.text().await?;
      anyhow::bail!(response)
    }

    #[derive(Deserialize)]
    struct Response {
      token: String,
    }

    let response = request.json::<Response>().await?;

    Ok(response.token)
  }
}
