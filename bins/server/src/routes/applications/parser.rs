use std::str::FromStr;

use axum::extract::Multipart;
use chrono::Utc;
use uuid::Uuid;

use super::{
  dto::CreateApplicationRequest,
  error::ApplicationError,
  models::{AgentOs, Attachment, Scripts},
};
use crate::utils::s3::{put_object, upload_multipart_from_field};

pub async fn parse_application_multipart(
  s3: &aws_sdk_s3::Client,
  bucket: &str,
  organization: Uuid,
  mut multipart: Multipart,
  uploaded_keys: &mut Vec<String>,
) -> Result<CreateApplicationRequest, ApplicationError> {
  let mut os = None;
  let mut name = None;
  let mut asset = None;
  let mut image = None;
  let mut scripts = None;

  while let Some(field) = multipart.next_field().await.map_err(|error| {
    tracing::error!(?error, "parsing multipart error");
    ApplicationError::ParseMultipart
  })? {
    let field_name = field.name().unwrap_or_default().to_owned();

    match field_name.as_str() {
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
        let text = field.text().await.map_err(|error| {
          tracing::error!(?error, "failed to parse name field");
          ApplicationError::InvalidField("name")
        })?;

        name = Some(text);
      }
      "scripts" => {
        let text = field.text().await.map_err(|error| {
          tracing::error!(?error, "failed to parse scripts field");
          ApplicationError::InvalidField("scripts")
        })?;

        let parsed_scripts = serde_json::from_str::<Scripts>(&text).map_err(|error| {
          tracing::error!(?error, "failed to deserialize scripts field");
          ApplicationError::InvalidField("scripts")
        })?;

        scripts = Some(parsed_scripts);
      }
      "asset" => {
        if asset.is_some() {
          return Err(ApplicationError::InvalidField("asset"));
        }

        let id = Uuid::new_v4();
        let filename = field.file_name().unwrap_or("asset.bin").to_string();
        let content_type = field
          .content_type()
          .unwrap_or("application/octet-stream")
          .to_string();

        let key = format!("{}/{}/{}", organization, id, filename);

        upload_multipart_from_field(s3, bucket, &key, &content_type, field).await?;
        uploaded_keys.push(key.clone());

        asset = Some(Attachment {
          id,
          key,
          filename,
          organization,
          content_type,
          uploaded: true,
          created_at: Utc::now(),
          ..Default::default()
        });
      }
      "image" => {
        if image.is_some() {
          return Err(ApplicationError::InvalidField("image"));
        }

        let id = Uuid::new_v4();
        let filename = field.file_name().unwrap_or("image.png").to_string();
        let content_type = field.content_type().unwrap_or("image/png").to_string();

        let key = format!("{}/{}/{}", organization, id, filename);

        let bytes = field.bytes().await.map_err(|error| {
          tracing::error!(?error, "failed to parse image field");
          ApplicationError::InvalidField("image")
        })?;

        put_object(s3, bucket, &key, &content_type, bytes).await?;
        uploaded_keys.push(key.clone());

        image = Some(Attachment {
          id,
          key,
          filename,
          organization,
          content_type,
          uploaded: true,
          created_at: Utc::now(),
          ..Default::default()
        });
      }
      _ => {}
    }
  }

  let scripts = scripts.unwrap_or_default();
  let os = os.ok_or(ApplicationError::MissingField("os"))?;
  let name = name.ok_or(ApplicationError::MissingField("name"))?;
  let asset = asset.ok_or(ApplicationError::MissingField("asset"))?;

  Ok(CreateApplicationRequest {
    os,
    name,
    asset,
    image,
    scripts,
  })
}
