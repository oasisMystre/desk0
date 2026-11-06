use std::str::FromStr;

use axum::extract::Multipart;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{
  error::ApplicationError,
  models::{AgentOs, RawFileFromRequest, Scripts},
};

#[derive(Deserialize, Serialize, Clone, Debug, ToSchema)]
pub struct AddDeviceApplicationRequest {
  pub application: uuid::Uuid,
}

pub struct CreateApplicationRequest {
  pub os: AgentOs,
  pub name: String,
  pub scripts: Scripts,
  pub asset: RawFileFromRequest,
  pub image: Option<RawFileFromRequest>,
}

impl CreateApplicationRequest {
  pub async fn try_from_multipart(mut value: Multipart) -> Result<Self, ApplicationError> {
    let mut os = None;
    let mut name = None;
    let mut asset = None;
    let mut image = None;
    let mut scripts = None;

    while let Some(field) = value.next_field().await.map_err(|error| {
      tracing::error!(?error, "parsing multipart error");
      ApplicationError::ParseMultipart
    })? {
      let field_name = field.name().unwrap_or_default().to_owned();
      let field_name = field_name.as_str();

      match field_name {
        "os" => {
          let text = field.text().await.map_err(|error| {
            tracing::error!(?error, "failed to parse os field");
            ApplicationError::InvalidField("os")
          })?;

          let parsed_os = AgentOs::from_str(&text).map_err(|error| {
            tracing::error!(?error, "invalid os value");
            ApplicationError::InvalidField("os")
          })?;

          os = Some(parsed_os);
        }
        "name" => {
          name = Some(field.text().await.map_err(|error| {
            tracing::error!(?error, "failed to parse name field");
            ApplicationError::InvalidField("name")
          })?);
        }
        "asset" => {
          let filename = field.file_name().map(ToString::to_string);
          let content_type = field.content_type().map(ToString::to_string);
          let bytes = field.bytes().await.map_err(|error| {
            tracing::error!(?error, "failed to parse asset field");
            ApplicationError::InvalidField("asset")
          })?;
          asset = Some(RawFileFromRequest::new(bytes, filename, content_type));
        }
        "image" => {
          let filename = field.file_name().map(ToString::to_string);
          let content_type = field.content_type().map(ToString::to_string);
          let bytes = field.bytes().await.map_err(|error| {
            tracing::error!(?error, "failed to parse image field");
            ApplicationError::InvalidField("image")
          })?;

          image = Some(RawFileFromRequest::new(bytes, filename, content_type));
        }
        "scripts" => {
          let value = field.text().await.map_err(|error| {
            tracing::error!(?error, "failed to parse scripts field");
            ApplicationError::InvalidField("scripts")
          })?;

          scripts = Some(serde_json::from_str::<Scripts>(&value).map_err(|error| {
            tracing::error!(?error, "failed to deserialize scripts field");
            ApplicationError::InvalidField("scripts")
          })?);
        }
        _ => {}
      }
    }

    let os = os.ok_or(ApplicationError::MissingField("os"))?;
    let name = name.ok_or(ApplicationError::MissingField("name"))?;
    let asset = asset.ok_or(ApplicationError::MissingField("asset"))?;
    let scripts = scripts.unwrap_or_default();

    Ok(Self {
      os,
      name,
      asset,
      image,
      scripts,
    })
  }
}
