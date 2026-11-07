use std::time::Duration;

use aws_sdk_s3::presigning::PresigningConfig;
use chrono::Utc;
use shared::env::FromEnv;

use super::{
  dto::{CreateUploadRequest, CreateUploadResponse, UpdateAttachmentRequest},
  error::AttachmentError,
  models::{Attachment, UpdateAttachmentChangeset},
  repository::AttachmentRepository,
};
use crate::{context::Context, env::Env};

pub struct AttachmentService;

impl AttachmentService {
  pub async fn get_attachments(
    context: &Context,
    organization: uuid::Uuid,
  ) -> Result<Vec<Attachment>, AttachmentError> {
    let mut connection = context.get_pg().await?;

    AttachmentRepository::find_by_organization(&mut connection, organization)
      .await
      .map_err(|error| {
        tracing::error!(?error, "failed to fetch attachments");
        AttachmentError::FetchFailed
      })
  }

  pub async fn create_attachment(
    context: &Context,
    organization: uuid::Uuid,
    body: CreateUploadRequest,
  ) -> Result<CreateUploadResponse, AttachmentError> {
    let id = uuid::Uuid::new_v4();
    let key = format!("{}/{}/{}", organization, id, body.filename);

    let bucket_name = Env::S3Bucket.get().map_err(|error| {
      tracing::error!(?error, "missing S3 bucket configuration");
      AttachmentError::ConfigError
    })?;

    let presigning_config =
      PresigningConfig::expires_in(Duration::from_mins(30)).map_err(|error| {
        tracing::error!(?error, "failed to create presigning config");
        AttachmentError::PresignFailed
      })?;

    let presigned = context
      .s3
      .put_object()
      .bucket(&bucket_name)
      .key(&key)
      .content_type(&body.content_type)
      .presigned(presigning_config)
      .await
      .map_err(|error| {
        tracing::error!(?error, "failed to generate presigned upload url");
        AttachmentError::PresignFailed
      })?;

    let mut connection = context.get_pg().await?;

    let attachment = Attachment {
      key,
      id,
      uploaded: false,
      organization,
      created_at: Utc::now(),
      metadata: body.metadata,
      filename: body.filename,
      content_type: body.content_type.clone(),
    };

    AttachmentRepository::create(&mut connection, &[attachment])
      .await
      .map_err(|error| {
        tracing::error!(?error, "failed to insert attachment");
        AttachmentError::SaveFailed
      })?;

    Ok(CreateUploadResponse {
      id,
      url: presigned.uri().to_string(),
    })
  }

  pub async fn update_attachment(
    context: &Context,
    attachment: uuid::Uuid,
    organization: uuid::Uuid,
    body: UpdateAttachmentRequest,
  ) -> Result<Attachment, AttachmentError> {
    let mut connection = context.get_pg().await?;

    let changeset = UpdateAttachmentChangeset {
      uploaded: body.uploaded,
      filename: body.filename,
      metadata: body.metadata,
    };

    AttachmentRepository::update(&mut connection, attachment, organization, &changeset)
      .await
      .map_err(|error| match error {
        diesel::result::Error::NotFound => AttachmentError::NotFound,
        _ => {
          tracing::error!(?error, "failed to update attachment");
          AttachmentError::UpdateFailed
        }
      })
  }

  pub async fn delete_attachment(
    context: &Context,
    attachment: uuid::Uuid,
    organization: uuid::Uuid,
  ) -> Result<Attachment, AttachmentError> {
    let mut connection = context.get_pg().await?;

    AttachmentRepository::delete(&mut connection, attachment, organization)
      .await
      .map_err(|error| match error {
        diesel::result::Error::NotFound => AttachmentError::NotFound,
        _ => {
          tracing::error!(?error, "failed to delete attachment");
          AttachmentError::DeleteFailed
        }
      })
  }
}
