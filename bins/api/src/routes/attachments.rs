use std::time::Duration;

use aws_sdk_s3::presigning::PresigningConfig;
use axum::{
  Json,
  extract::{Path, State},
  http::StatusCode,
};
use diesel::{
  BoolExpressionMethods, ExpressionMethods, HasQuery, QueryDsl, SelectableHelper,
  query_builder::AsChangeset,
};
use diesel_async::RunQueryDsl;
use flow::{attachment::Attachment, attachments};
use serde::{Deserialize, Serialize};
use shared::env::FromEnv;

use crate::{auth::AuthenticatedUser, context::Context, env::Env};

#[derive(Deserialize)]
pub struct CreateUploadRequest {
  pub filename: String,
  pub content_type: String,
  pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct CreateUploadResponse {
  pub upload_url: String,
  pub file_id: uuid::Uuid,
  pub public_url: String,
}

#[derive(Deserialize, AsChangeset)]
#[diesel(table_name = attachments)]
pub struct UpdateAttachment {
  pub uploaded: Option<bool>,
  pub filename: Option<String>,
  pub metadata: Option<serde_json::Value>,
}

pub async fn get_attachments(
  State(context): State<Context>,
  user: AuthenticatedUser,
) -> Result<Json<Vec<Attachment>>, StatusCode> {
  let mut connection = context.pg.get().await.map_err(|error| {
    tracing::error!(?error, "database connection error");
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  let results: Vec<Attachment> = Attachment::query()
    .filter(attachments::organization.eq(user.org_id()?))
    .load(&mut connection)
    .await
    .map_err(|error| {
      tracing::error!(?error, "database query error");
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  Ok(Json(results))
}

pub async fn create_attachment(
  State(context): State<Context>,
  user: AuthenticatedUser,
  Json(body): Json<CreateUploadRequest>,
) -> Result<Json<CreateUploadResponse>, StatusCode> {
  let file_id = uuid::Uuid::new_v4();
  let key = format!("{}/{}/{}", user.org_id()?, file_id, body.filename);
  let presigned = context
    .s3
    .put_object()
    .bucket(
      &Env::S3Bucket
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    )
    .key(&key)
    .content_type(&body.content_type)
    .presigned(
      PresigningConfig::expires_in(Duration::from_mins(30)).map_err(|error| {
        tracing::error!(?error);
        StatusCode::INTERNAL_SERVER_ERROR
      })?,
    )
    .await
    .map_err(|error| {
      tracing::error!(?error);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  let mut connection = context.pg.get().await.map_err(|error| {
    tracing::error!(?error, "database connection error");
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  let public_url = format!(
    "{}/{}",
    Env::S3PublicBaseUrl
      .get()
      .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    key
  );

  let attachment = Attachment {
    key,
    id: file_id,
    uploaded: false,
    metadata: body.metadata,
    filename: body.filename,
    organization: user.org_id()?,
    public_url: public_url.clone(),
    content_type: body.content_type.clone(),
    ..Default::default()
  };

  diesel::insert_into(attachments::table)
    .values(&attachment)
    .execute(&mut connection)
    .await
    .map_err(|error| {
      tracing::error!(?error, "database insert error");
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  Ok(Json(CreateUploadResponse {
    file_id,
    public_url,
    upload_url: presigned.uri().to_string(),
  }))
}

pub async fn update_attachment(
  State(context): State<Context>,
  Path(file_id): Path<uuid::Uuid>,
  user: AuthenticatedUser,
  Json(body): Json<UpdateAttachment>,
) -> Result<Json<Attachment>, StatusCode> {
  let mut connection = context.pg.get().await.map_err(|error| {
    tracing::error!(?error, "database connection error");
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  let result: Attachment = diesel::update(
    attachments::table.filter(
      attachments::id
        .eq(file_id)
        .and(attachments::organization.eq(user.org_id()?)),
    ),
  )
  .set(&body)
  .returning(Attachment::as_returning())
  .get_result(&mut connection)
  .await
  .map_err(|error| match error {
    diesel::result::Error::NotFound => StatusCode::NOT_FOUND,
    _ => {
      tracing::error!(?error, "pg update error");
      StatusCode::INTERNAL_SERVER_ERROR
    }
  })?;

  Ok(Json(result))
}

pub async fn delete_attachment(
  State(context): State<Context>,
  Path(file_id): Path<uuid::Uuid>,
  user: AuthenticatedUser,
) -> Result<Json<Attachment>, StatusCode> {
  let mut connection = context.pg.get().await.map_err(|error| {
    tracing::error!(?error, "database connection error");
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  let result: Attachment = diesel::delete(
    attachments::table.filter(
      attachments::id
        .eq(file_id)
        .and(attachments::organization.eq(user.org_id()?)),
    ),
  )
  .returning(Attachment::as_returning())
  .get_result(&mut connection)
  .await
  .map_err(|error| match error {
    diesel::result::Error::NotFound => StatusCode::NOT_FOUND,
    _ => {
      tracing::error!(?error, "pg update error");
      StatusCode::INTERNAL_SERVER_ERROR
    }
  })?;

  Ok(Json(result))
}
