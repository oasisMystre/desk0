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
    org_id: uuid::Uuid,
  ) -> Result<Vec<Attachment>, AttachmentError> {
    let mut connection = context.get_pg().await?;

    AttachmentRepository::find_by_organization(&mut connection, org_id)
      .await
      .map_err(|error| {
        tracing::error!(?error, "failed to fetch attachments");
        AttachmentError::FetchFailed
      })
  }

  pub async fn create_attachment(
    context: &Context,
    org_id: uuid::Uuid,
    body: CreateUploadRequest,
  ) -> Result<CreateUploadResponse, AttachmentError> {
    let file_id = uuid::Uuid::new_v4();
    let key = format!("{}/{}/{}", org_id, file_id, body.filename);

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

    let public_base_url = Env::S3PublicBaseUrl.get().map_err(|error| {
      tracing::error!(?error, "missing S3 public base url configuration");
      AttachmentError::ConfigError
    })?;

    let public_url = format!("{}/{}", public_base_url, key);

    let attachment = Attachment {
      key,
      id: file_id,
      uploaded: false,
      metadata: body.metadata,
      filename: body.filename,
      organization: org_id,
      public_url: public_url.clone(),
      content_type: body.content_type.clone(),
      created_at: Utc::now(),
    };

    AttachmentRepository::create(&mut connection, &[attachment])
      .await
      .map_err(|error| {
        tracing::error!(?error, "failed to insert attachment");
        AttachmentError::SaveFailed
      })?;

    Ok(CreateUploadResponse {
      file_id,
      public_url,
      upload_url: presigned.uri().to_string(),
    })
  }

  pub async fn update_attachment(
    context: &Context,
    file_id: uuid::Uuid,
    org_id: uuid::Uuid,
    body: UpdateAttachmentRequest,
  ) -> Result<Attachment, AttachmentError> {
    let mut connection = context.get_pg().await?;

    let changeset = UpdateAttachmentChangeset {
      uploaded: body.uploaded,
      filename: body.filename,
      metadata: body.metadata,
    };

    AttachmentRepository::update(&mut connection, file_id, org_id, &changeset)
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
    file_id: uuid::Uuid,
    org_id: uuid::Uuid,
  ) -> Result<Attachment, AttachmentError> {
    let mut connection = context.get_pg().await?;

    AttachmentRepository::delete(&mut connection, file_id, org_id)
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
