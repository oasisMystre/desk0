use axum::extract::multipart::Field;
use bytes::{Bytes, BytesMut};

pub const MAX_PARTS_LIMIT: i32 = 10_000;
pub const MIN_PART_SIZE: usize = 5 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum S3UploadError {
  #[error("Failed to initiate multipart upload: {0}")]
  Initiate(String),
  #[error("Missing upload ID from create multipart upload response")]
  MissingUploadId,
  #[error("Failed to read chunk from multipart field: {0}")]
  ReadChunk(String),
  #[error("Failed to upload part {part_number}: {error}")]
  UploadPart { part_number: i32, error: String },
  #[error("Multipart upload exceeded maximum parts limit (10,000)")]
  MaxPartsExceeded,
  #[error("Failed to complete multipart upload: {0}")]
  Complete(String),
  #[error("Failed to put object: {0}")]
  PutObject(String),
  #[error("Failed to abort object")]
  AbortObject,
}

pub async fn upload_multipart_from_field(
  s3: &aws_sdk_s3::Client,
  bucket: &str,
  key: &str,
  content_type: &str,
  mut field: Field<'_>,
) -> Result<(), S3UploadError> {
  let create_response = s3
    .create_multipart_upload()
    .bucket(bucket)
    .key(key)
    .content_type(content_type)
    .send()
    .await
    .map_err(|error| {
      tracing::error!(?error, key, "failed to initiate multipart upload");
      S3UploadError::Initiate(error.to_string())
    })?;

  let upload_id = match create_response.upload_id() {
    Some(id) => id.to_string(),
    None => {
      tracing::error!(
        key,
        "missing upload_id from create_multipart_upload response"
      );
      return Err(S3UploadError::MissingUploadId);
    }
  };

  let upload_result: Result<(), S3UploadError> = async {
    let mut buffer = BytesMut::with_capacity(MIN_PART_SIZE);
    let mut part_number: i32 = 1;
    let mut completed_parts = Vec::new();

    while let Some(chunk) = field.chunk().await.map_err(|error| {
      tracing::error!(?error, key, "failed to read chunk from multipart field");
      S3UploadError::ReadChunk(error.to_string())
    })? {
      buffer.extend_from_slice(&chunk);

      while buffer.len() >= MIN_PART_SIZE {
        if part_number > MAX_PARTS_LIMIT {
          tracing::error!(
            key,
            "multipart upload exceeded maximum allowed parts limit (10,000)"
          );
          return Err(S3UploadError::MaxPartsExceeded);
        }

        let part_data = buffer.split_to(MIN_PART_SIZE).freeze();
        let upload_part_res = s3
          .upload_part()
          .bucket(bucket)
          .key(key)
          .upload_id(&upload_id)
          .part_number(part_number)
          .body(part_data.into())
          .send()
          .await
          .map_err(|error| {
            tracing::error!(?error, key, part_number, "failed to upload part to s3");
            S3UploadError::UploadPart {
              part_number,
              error: error.to_string(),
            }
          })?;

        let e_tag = upload_part_res.e_tag().map(ToString::to_string);
        completed_parts.push(
          aws_sdk_s3::types::CompletedPart::builder()
            .part_number(part_number)
            .set_e_tag(e_tag)
            .build(),
        );
        part_number += 1;
      }
    }

    if !buffer.is_empty() || completed_parts.is_empty() {
      if part_number > MAX_PARTS_LIMIT {
        tracing::error!(
          key,
          "multipart upload exceeded maximum allowed parts limit (10,000)"
        );
        return Err(S3UploadError::MaxPartsExceeded);
      }

      let part_data = buffer.split().freeze();
      let upload_part_res = s3
        .upload_part()
        .bucket(bucket)
        .key(key)
        .upload_id(&upload_id)
        .part_number(part_number)
        .body(part_data.into())
        .send()
        .await
        .map_err(|error| {
          tracing::error!(
            ?error,
            key,
            part_number,
            "failed to upload final part to s3"
          );
          S3UploadError::UploadPart {
            part_number,
            error: error.to_string(),
          }
        })?;

      let e_tag = upload_part_res.e_tag().map(ToString::to_string);
      completed_parts.push(
        aws_sdk_s3::types::CompletedPart::builder()
          .part_number(part_number)
          .set_e_tag(e_tag)
          .build(),
      );
    }

    let completed_upload = aws_sdk_s3::types::CompletedMultipartUpload::builder()
      .set_parts(Some(completed_parts))
      .build();

    s3.complete_multipart_upload()
      .bucket(bucket)
      .key(key)
      .upload_id(&upload_id)
      .multipart_upload(completed_upload)
      .send()
      .await
      .map_err(|error| {
        tracing::error!(?error, key, "failed to complete multipart upload");
        S3UploadError::Complete(error.to_string())
      })?;

    Ok(())
  }
  .await;

  if let Err(error) = upload_result {
    tracing::error!(
      key,
      upload_id,
      ?error,
      "aborting multipart upload due to error"
    );

    s3.abort_multipart_upload()
      .bucket(bucket)
      .key(key)
      .upload_id(&upload_id)
      .send()
      .await
      .map_err(|error| {
        tracing::error!(?error, "aborting multipart upload error");
        S3UploadError::AbortObject
      })?;

    return Err(error);
  }

  Ok(())
}

pub async fn put_object(
  s3: &aws_sdk_s3::Client,
  bucket: &str,
  key: &str,
  content_type: &str,
  bytes: Bytes,
) -> Result<(), S3UploadError> {
  s3.put_object()
    .bucket(bucket)
    .key(key)
    .content_type(content_type)
    .body(bytes.into())
    .send()
    .await
    .map_err(|error| {
      tracing::error!(?error, key, "failed to upload object to s3");
      S3UploadError::PutObject(error.to_string())
    })?;
  Ok(())
}

pub async fn delete_object(
  s3: &aws_sdk_s3::Client,
  bucket: &str,
  key: &str,
) -> Result<(), aws_sdk_s3::Error> {
  s3.delete_object()
    .bucket(bucket)
    .key(key)
    .send()
    .await
    .map_err(aws_sdk_s3::Error::from)?;
  Ok(())
}

pub async fn delete_objects(s3: &aws_sdk_s3::Client, bucket: &str, keys: &[impl AsRef<str>]) {
  for key in keys {
    let key_ref = key.as_ref();
    if let Err(error) = s3.delete_object().bucket(bucket).key(key_ref).send().await {
      tracing::warn!(?error, key = %key_ref, "failed to cleanup s3 object");
    }
  }
}
