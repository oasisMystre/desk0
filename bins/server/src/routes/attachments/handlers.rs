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
use crate::{
  error::ErrorResponse,
  states::{AppState, AuthenticatedUser},
};

#[utoipa::path(
  get,
  path = "/attachments",
  responses(
    (status = 200, description = "List attachments", body = Vec<Attachment>),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  security(
    ("bearer_auth" = [])
  ),
  tag = "attachments"
)]
pub async fn get_attachments(
  State(AppState { context, .. }): State<AppState>,
  user: AuthenticatedUser,
) -> Result<Json<Vec<Attachment>>, AttachmentError> {
  let attachments = AttachmentService::get_attachments(&context, user.org_id()?).await?;
  Ok(Json(attachments))
}

#[utoipa::path(
  post,
  path = "/attachments",
  request_body = CreateUploadRequest,
  responses(
    (status = 200, description = "Attachment upload URL generated", body = CreateUploadResponse),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  security(
    ("bearer_auth" = [])
  ),
  tag = "attachments"
)]
pub async fn create_attachment(
  State(AppState { context, .. }): State<AppState>,
  user: AuthenticatedUser,
  Json(body): Json<CreateUploadRequest>,
) -> Result<Json<CreateUploadResponse>, AttachmentError> {
  let response = AttachmentService::create_attachment(&context, user.org_id()?, body).await?;
  Ok(Json(response))
}

#[utoipa::path(
  patch,
  path = "/attachments/{file_id}",
  params(
    ("file_id" = uuid::Uuid, Path, description = "Attachment File UUID")
  ),
  request_body = UpdateAttachmentRequest,
  responses(
    (status = 200, description = "Attachment updated", body = Attachment),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  security(
    ("bearer_auth" = [])
  ),
  tag = "attachments"
)]
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

#[utoipa::path(
  delete,
  path = "/attachments/{file_id}",
  params(
    ("file_id" = uuid::Uuid, Path, description = "Attachment File UUID")
  ),
  responses(
    (status = 200, description = "Attachment deleted", body = Attachment),
    (status = 401, description = "Unauthorized", body = ErrorResponse),
    (status = 500, description = "Internal server error", body = ErrorResponse)
  ),
  security(
    ("bearer_auth" = [])
  ),
  tag = "attachments"
)]
pub async fn delete_attachment(
  State(AppState { context, .. }): State<AppState>,
  Path(file_id): Path<uuid::Uuid>,
  user: AuthenticatedUser,
) -> Result<Json<Attachment>, AttachmentError> {
  let result = AttachmentService::delete_attachment(&context, file_id, user.org_id()?).await?;
  Ok(Json(result))
}
