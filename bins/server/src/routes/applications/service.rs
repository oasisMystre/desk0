use axum::extract::Multipart;
use chrono::Utc;
use diesel_async::AsyncConnection;
use shared::env::FromEnv;

use super::{
  error::ApplicationError,
  models::{Application, DeviceApplication, ResolvedApplication, ResolvedDeviceApplication},
  parser::parse_application_multipart,
  repository::ApplicationRepository,
  repository::DeviceApplicationRepository,
};
use crate::{
  SharedError,
  context::Context,
  env::Env,
  routes::attachments::repository::AttachmentRepository,
  utils::s3::delete_objects,
};

pub struct ApplicationService;
pub struct DeviceApplicationService;

impl ApplicationService {
  pub async fn create_application(
    context: &Context,
    organization: uuid::Uuid,
    multipart: Multipart,
  ) -> Result<ResolvedApplication, ApplicationError> {
    let bucket = Env::S3Bucket.get().map_err(|error| {
      tracing::error!(?error, "missing S3 bucket configuration");
      ApplicationError::ConfigError
    })?;

    let mut uploaded_keys = Vec::new();

    let result = async {
      let request = parse_application_multipart(
        &context.s3,
        &bucket,
        organization,
        multipart,
        &mut uploaded_keys,
      )
      .await?;

      let mut pg = context.get_pg().await?;

      let scripts = serde_json::to_value(&request.scripts).map_err(|error| {
        tracing::error!(?error, "failed to serialize scripts");
        ApplicationError::InvalidField("scripts")
      })?;

      let resolved = pg
        .transaction::<_, ApplicationError, _>(async |connection| {
          let application = Application {
            scripts,
            organization,
            os: request.os,
            name: request.name,
            created_at: Utc::now(),
            id: uuid::Uuid::new_v4(),
            asset: request.asset.id,
            image: request.image.as_ref().map(|value| value.id),
          };

          let mut attachments = Vec::with_capacity(2);
          attachments.push(request.asset);
          if let Some(attachment) = request.image {
            attachments.push(attachment);
          }

          AttachmentRepository::create(connection, &attachments)
            .await
            .map_err(|error| {
              tracing::error!(?error, "failed to save attachments");
              ApplicationError::CreateFailed
            })?;

          let application = ApplicationRepository::create(connection, &application)
            .await
            .map_err(|error| {
              tracing::error!(?error, "failed to insert application");
              ApplicationError::CreateFailed
            })?;

          let image = attachments.pop();
          let asset = attachments.pop().expect("asset should not be none");

          Ok(ResolvedApplication {
            asset,
            image,
            id: application.id,
            os: application.os,
            name: application.name,
            scripts: request.scripts,
            created_at: application.created_at,
            organization: application.organization,
          })
        })
        .await?;

      Ok(resolved)
    }
    .await;

    match result {
      Ok(application) => Ok(application),
      Err(error) => {
        delete_objects(&context.s3, &bucket, &uploaded_keys).await;
        Err(error)
      }
    }
  }

  pub async fn get_applications(
    context: &Context,
    organization: uuid::Uuid,
  ) -> Result<Vec<ResolvedApplication>, ApplicationError> {
    let mut connection = context.get_pg().await?;

    ApplicationRepository::find_by_organization(&mut connection, organization)
      .await
      .map_err(|error| {
        tracing::error!(?error, "failed to fetch applications");
        ApplicationError::FetchFailed
      })
  }

  pub async fn get_application(
    context: &Context,
    organization_id: uuid::Uuid,
    application_id: uuid::Uuid,
  ) -> Result<ResolvedApplication, ApplicationError> {
    let mut connection = context.get_pg().await?;

    ApplicationRepository::find_by_id(&mut connection, application_id, organization_id)
      .await
      .map_err(|error| match error {
        diesel::result::Error::NotFound => ApplicationError::NotFound,
        _ => {
          tracing::error!(?error, "failed to fetch application");
          ApplicationError::FetchFailed
        }
      })
  }

  pub async fn delete_application(
    context: &Context,
    organization_id: uuid::Uuid,
    application_id: uuid::Uuid,
  ) -> Result<Application, ApplicationError> {
    let mut connection = context.get_pg().await?;

    ApplicationRepository::delete(&mut connection, application_id, organization_id)
      .await
      .map_err(|error| match error {
        diesel::result::Error::NotFound => ApplicationError::NotFound,
        _ => {
          tracing::error!(?error, "failed to delete application");
          ApplicationError::DeleteFailed
        }
      })
  }
}

impl DeviceApplicationService {
  pub async fn get_device_applications(
    context: &Context,
    organization: uuid::Uuid,
    device: uuid::Uuid,
  ) -> Result<Vec<ResolvedDeviceApplication>, ApplicationError> {
    let mut connection = context.get_pg().await?;

    DeviceApplicationRepository::find_device_applications(&mut connection, device, organization)
      .await
      .map_err(|error| {
        tracing::error!(?error, "failed to fetch device applications");
        ApplicationError::FetchFailed
      })
  }

  pub async fn add_device_application(
    context: &Context,
    organization: uuid::Uuid,
    device: uuid::Uuid,
    application: uuid::Uuid,
  ) -> Result<DeviceApplication, ApplicationError> {
    let mut connection = context.get_pg().await?;

    DeviceApplicationRepository::add_device_application(
      &mut connection,
      &DeviceApplication {
        device,
        application,
        organization,
        created_at: Utc::now(),
        id: uuid::Uuid::new_v4(),
      },
    )
    .await
    .map_err(|error| match error {
      diesel::result::Error::NotFound => ApplicationError::NotFound,
      diesel::result::Error::DatabaseError(
        diesel::result::DatabaseErrorKind::ForeignKeyViolation,
        info,
      ) => match info.constraint_name() {
        Some("device_applications_device_devices_id_fkey") => ApplicationError::DeviceNotFound,
        Some("device_applications_application_applications_id_fkey") => ApplicationError::NotFound,
        Some("device_applications_device_organization_fkey") => SharedError::Unauthorized.into(),
        _ => ApplicationError::CreateFailed,
      },

      error => {
        tracing::error!(?error, "failed to add application to device");
        ApplicationError::CreateFailed
      }
    })
  }

  pub async fn remove_device_application(
    context: &Context,
    organization: uuid::Uuid,
    device: uuid::Uuid,
    application: uuid::Uuid,
  ) -> Result<DeviceApplication, ApplicationError> {
    let mut connection = context.get_pg().await?;

    DeviceApplicationRepository::remove_device_application(
      &mut connection,
      device,
      application,
      organization,
    )
    .await
    .map_err(|error| match error {
      diesel::result::Error::NotFound => ApplicationError::DeviceNotFound,
      _ => {
        tracing::error!(?error, "failed to remove application from device");
        ApplicationError::DeleteFailed
      }
    })
  }
}
