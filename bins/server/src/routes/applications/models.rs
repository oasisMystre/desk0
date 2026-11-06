pub use flow::{
  application::{Application, Scripts},
  attachment::Attachment,
  device_application::DeviceApplication,
  os::AgentOs,
  raw_file_from_request::RawFileFromRequest,
};

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, utoipa::ToSchema)]
pub struct ResolvedApplication {
  pub id: uuid::Uuid,
  pub name: String,
  pub os: AgentOs,
  pub scripts: Scripts,
  pub asset: Attachment,
  pub image: Option<Attachment>,
  pub organization: uuid::Uuid,
  pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, utoipa::ToSchema)]
pub struct ResolvedDeviceApplication {
  pub id: uuid::Uuid,
  pub device: uuid::Uuid,
  pub application: ResolvedApplication,
  pub created_at: chrono::DateTime<chrono::Utc>,
}

macro_rules! attachment_fields {
  ($alias:ident) => {
    (
      $alias.field(attachments::id),
      $alias.field(attachments::key),
      $alias.field(attachments::filename),
      $alias.field(attachments::uploaded),
      $alias.field(attachments::organization),
      $alias.field(attachments::contentType),
      $alias.field(attachments::publicUrl),
      $alias.field(attachments::metadata),
      $alias.field(attachments::createdAt),
    )
  };
}

pub(crate) use attachment_fields;
