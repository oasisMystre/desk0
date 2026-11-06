use diesel::{
  BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl,
  SelectableHelper,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use flow::{applications, attachments, device_applications, devices};

use crate::routes::applications::models::ResolvedDeviceApplication;

use super::models::{
  Application, Attachment, DeviceApplication, ResolvedApplication, attachment_fields,
};

pub struct ApplicationRepository;
pub struct DeviceApplicationRepository;

impl ApplicationRepository {
  pub async fn create(
    connection: &mut AsyncPgConnection,
    application: &Application,
  ) -> Result<Application, diesel::result::Error> {
    diesel::insert_into(applications::table)
      .values(application)
      .returning(Application::as_returning())
      .get_result(connection)
      .await
  }

  pub async fn find_by_organization(
    connection: &mut AsyncPgConnection,
    organization: uuid::Uuid,
  ) -> Result<Vec<ResolvedApplication>, diesel::result::Error> {
    let (asset_attachments, image_attachments) = diesel::alias!(
      attachments as asset_attachments,
      attachments as image_attachements,
    );

    let applications = applications::table
      .filter(applications::organization.eq(organization))
      .inner_join(
        asset_attachments.on(applications::asset.eq(asset_attachments.field(attachments::id))),
      )
      .left_join(
        image_attachments
          .on(applications::image.eq(image_attachments.field(attachments::id).nullable())),
      )
      .select((
        Application::as_select(),
        attachment_fields!(asset_attachments),
        attachment_fields!(image_attachments).nullable(),
      ))
      .load::<(Application, Attachment, Option<Attachment>)>(connection)
      .await?;

    Ok(
      applications
        .into_iter()
        .map(|(application, asset, image)| {
          Ok(ResolvedApplication {
            image,
            asset,
            id: application.id,
            os: application.os,
            name: application.name,
            created_at: application.created_at,
            organization: application.organization,
            scripts: serde_json::from_value(application.scripts)
              .map_err(|error| diesel::result::Error::DeserializationError(Box::new(error)))?,
          })
        })
        .collect::<Result<Vec<ResolvedApplication>, diesel::result::Error>>()?,
    )
  }

  pub async fn find_by_id(
    connection: &mut AsyncPgConnection,
    application: uuid::Uuid,
    organization: uuid::Uuid,
  ) -> Result<ResolvedApplication, diesel::result::Error> {
    let (asset_attachments, image_attachments) = diesel::alias!(
      attachments as asset_attachments,
      attachments as image_attachements,
    );

    let (application, asset, image) = applications::table
      .filter(
        applications::id
          .eq(application)
          .and(applications::organization.eq(organization)),
      )
      .inner_join(
        asset_attachments.on(applications::asset.eq(asset_attachments.field(attachments::id))),
      )
      .left_join(
        image_attachments
          .on(applications::image.eq(image_attachments.field(attachments::id).nullable())),
      )
      .select((
        Application::as_select(),
        attachment_fields!(asset_attachments),
        attachment_fields!(image_attachments).nullable(),
      ))
      .first::<(Application, Attachment, Option<Attachment>)>(connection)
      .await?;

    Ok(ResolvedApplication {
      image,
      asset,
      id: application.id,
      os: application.os,
      name: application.name,
      created_at: application.created_at,
      organization: application.organization,
      scripts: serde_json::from_value(application.scripts)
        .map_err(|error| diesel::result::Error::DeserializationError(Box::new(error)))?,
    })
  }

  pub async fn delete(
    connection: &mut AsyncPgConnection,
    application: uuid::Uuid,
    organization: uuid::Uuid,
  ) -> Result<Application, diesel::result::Error> {
    diesel::delete(
      applications::table.filter(
        applications::id
          .eq(application)
          .and(applications::organization.eq(organization)),
      ),
    )
    .returning(Application::as_returning())
    .get_result(connection)
    .await
  }
}
impl DeviceApplicationRepository {
  pub async fn add_device_application(
    connection: &mut AsyncPgConnection,
    value: &DeviceApplication,
  ) -> Result<DeviceApplication, diesel::result::Error> {
    diesel::insert_into(device_applications::table)
      .values(value)
      .returning(DeviceApplication::as_returning())
      .get_result(connection)
      .await
  }

  pub async fn find_device_applications(
    connection: &mut AsyncPgConnection,
    device: uuid::Uuid,
    organization: uuid::Uuid,
  ) -> Result<Vec<ResolvedDeviceApplication>, diesel::result::Error> {
    let (asset_attachments, image_attachments) = diesel::alias!(
      attachments as asset_attachments,
      attachments as image_attachements,
    );

    let applications = device_applications::table
      .inner_join(applications::table.on(applications::id.eq(device_applications::application)))
      .inner_join(
        asset_attachments.on(applications::asset.eq(asset_attachments.field(attachments::id))),
      )
      .left_join(
        image_attachments
          .on(applications::image.eq(image_attachments.field(attachments::id).nullable())),
      )
      .filter(
        device_applications::device
          .eq(device)
          .and(device_applications::organization.eq(organization)),
      )
      .select((
        DeviceApplication::as_select(),
        Application::as_select(),
        attachment_fields!(asset_attachments),
        attachment_fields!(image_attachments).nullable(),
      ))
      .load::<(
        DeviceApplication,
        Application,
        Attachment,
        Option<Attachment>,
      )>(connection)
      .await?;

    Ok(
      applications
        .into_iter()
        .map(|(device_application, application, asset, image)| {
          Ok(ResolvedDeviceApplication {
            id: device_application.id,
            device: device_application.device,
            created_at: device_application.created_at,
            application: ResolvedApplication {
              image,
              asset,
              id: application.id,
              os: application.os,
              name: application.name,
              created_at: application.created_at,
              organization: application.organization,
              scripts: serde_json::from_value(application.scripts)
                .map_err(|error| diesel::result::Error::DeserializationError(Box::new(error)))?,
            },
          })
        })
        .collect::<Result<Vec<ResolvedDeviceApplication>, diesel::result::Error>>()?,
    )
  }

  pub async fn remove_device_application(
    connection: &mut AsyncPgConnection,
    device: uuid::Uuid,
    application: uuid::Uuid,
    organization: uuid::Uuid,
  ) -> Result<DeviceApplication, diesel::result::Error> {
    diesel::delete(
      device_applications::table
        .filter(device_applications::device.eq(device))
        .filter(device_applications::application.eq(application))
        .filter(
          device_applications::device.eq_any(
            devices::table
              .filter(devices::organization.eq(organization))
              .select(devices::id),
          ),
        ),
    )
    .returning(DeviceApplication::as_returning())
    .get_result(connection)
    .await
  }
}
