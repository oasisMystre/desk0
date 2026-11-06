use std::sync::Arc;

use serde::{Deserialize, Serialize};

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
  pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateDevice {
  pub pubkey: Vec<u8>,
  pub device: Device,
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
#[derive(Serialize, Deserialize)]
pub struct Challenge {
  pub id: uuid::Uuid,
  pub challenge: String,
  pub expires_in: u64,
}

#[derive(Serialize, Deserialize)]
pub struct CreateConnectionTokenRequest {
  pub challenge: String,
  pub signature: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateConnectionTokenResponse {
  pub token: String,
}

impl From<Identity> for CreateDevice {
  fn from(value: Identity) -> Self {
    Self {
      pubkey: value.pubkey.clone(),
      device: Device {
        id: uuid::Uuid::new_v4(),
        organization: value.org_id,
        metadata: value.metadata(),
        os: Some(value.os.clone()),
        version: Some(value.version()),
        hostname: value.hostname.clone(),
        username: value.username.clone(),
      },
    }
  }
}

impl From<Identity> for UpdateDevice {
  fn from(value: Identity) -> Self {
    Self {
      organization: value.org_id,
      metadata: value.metadata(),
      os: Some(value.os.clone()),
      version: Some(value.version()),
      hostname: value.hostname.clone(),
      username: value.username.clone(),
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

    let response = request
      .send()
      .await?
      .error_for_status()?
      .json::<Device>()
      .await?;

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

    let response = request
      .send()
      .await?
      .error_for_status()?
      .json::<Device>()
      .await?;

    Ok(response)
  }

  pub async fn get_challenge(&self, device: &uuid::Uuid) -> anyhow::Result<Challenge> {
    let mut request = self
      .client
      .client
      .post(
        self
          .client
          .base_url
          .join(&format!("devices/challenge/{device}"))?,
      )
      .json(&serde_json::json!({"role": "agent"}));

    if let Some(ref headers) = self.client.headers {
      request = request.headers(headers.clone());
    }

    let response = request
      .send()
      .await?
      .error_for_status()?
      .json::<Challenge>()
      .await?;

    Ok(response)
  }

  pub async fn get_token(
    &self,
    device: &uuid::Uuid,
    body: CreateConnectionTokenRequest,
  ) -> anyhow::Result<String> {
    let mut request = self
      .client
      .client
      .post(
        self
          .client
          .base_url
          .join(&format!("devices/token/{device}"))?,
      )
      .json(&body);

    if let Some(ref headers) = self.client.headers {
      request = request.headers(headers.clone());
    }

    let response = request
      .send()
      .await?
      .error_for_status()?
      .json::<CreateConnectionTokenResponse>()
      .await?;

    Ok(response.token)
  }
}
