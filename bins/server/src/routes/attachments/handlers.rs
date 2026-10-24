use axum::{
  Json,
  extract::{Path, State},
};

use super::{
  dto::{CreateUploadRequest, CreateUploadResponse, UpdateAttachmentRequest},
  error::AttachmentError,
  models::Attachment,
  service::AttachmentService,
};
use crate::states::{AppState, AuthenticatedUser};

pub async fn get_attachments(
  State(AppState { context, .. }): State<AppState>,
  user: AuthenticatedUser,
) -> Result<Json<Vec<Attachment>>, AttachmentError> {
  let attachments = AttachmentService::get_attachments(&context, user.org_id()?).await?;
  Ok(Json(attachments))
}

pub async fn create_attachment(
  State(AppState { context, .. }): State<AppState>,
  user: AuthenticatedUser,
  Json(body): Json<CreateUploadRequest>,
) -> Result<Json<CreateUploadResponse>, AttachmentError> {
  let response = AttachmentService::create_attachment(&context, user.org_id()?, body).await?;
  Ok(Json(response))
}

pub async fn update_attachment(
  State(AppState { context, .. }): State<AppState>,
  Path(file_id): Path<uuid::Uuid>,
  user: AuthenticatedUser,
  Json(body): Json<UpdateAttachmentRequest>,
) -> Result<Json<Attachment>, AttachmentError> {
  let result =
    AttachmentService::update_attachment(&context, file_id, user.org_id()?, body).await?;
  Ok(Json(result))
}

pub async fn delete_attachment(
  State(AppState { context, .. }): State<AppState>,
  Path(file_id): Path<uuid::Uuid>,
  user: AuthenticatedUser,
) -> Result<Json<Attachment>, AttachmentError> {
  let result = AttachmentService::delete_attachment(&context, file_id, user.org_id()?).await?;
  Ok(Json(result))
}
