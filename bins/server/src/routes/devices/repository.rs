use diesel::{BoolExpressionMethods, ExpressionMethods, HasQuery, QueryDsl, SelectableHelper};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use flow::devices;

use super::models::{Device, UpdateDeviceChangeset};

pub struct DeviceRepository;

impl DeviceRepository {
  pub async fn find_by_organization(
    connection: &mut AsyncPgConnection,
    organization: uuid::Uuid,
  ) -> Result<Vec<Device>, diesel::result::Error> {
    Device::query()
      .filter(devices::organization.eq(organization))
      .load(connection)
      .await
  }

  pub async fn create(
    connection: &mut AsyncPgConnection,
    device: &Device,
  ) -> Result<Device, diesel::result::Error> {
    diesel::insert_into(devices::table)
      .values(device)
      .returning(Device::as_returning())
      .get_result(connection)
      .await
  }

  pub async fn update(
    connection: &mut AsyncPgConnection,
    device: uuid::Uuid,
    organization: uuid::Uuid,
    changeset: &UpdateDeviceChangeset,
  ) -> Result<Device, diesel::result::Error> {
    diesel::update(
      devices::table.filter(
        devices::id
          .eq(device)
          .and(devices::organization.eq(organization)),
      ),
    )
    .set(changeset)
    .returning(Device::as_returning())
    .get_result(connection)
    .await
  }

  pub async fn delete(
    connection: &mut AsyncPgConnection,
    device: uuid::Uuid,
    organization: uuid::Uuid,
  ) -> Result<Device, diesel::result::Error> {
    diesel::delete(
      devices::table.filter(
        devices::id
          .eq(device)
          .and(devices::organization.eq(organization)),
      ),
    )
    .returning(Device::as_returning())
    .get_result(connection)
    .await
  }

  pub async fn find_by_id(
    connection: &mut AsyncPgConnection,
    device: uuid::Uuid,
  ) -> Result<Device, diesel::result::Error> {
    Device::query()
      .filter(devices::id.eq(device))
      .first(connection)
      .await
  }

  pub async fn update_last_logged_in(
    connection: &mut AsyncPgConnection,
    device: uuid::Uuid,
  ) -> Result<(), diesel::result::Error> {
    diesel::update(devices::table.filter(devices::id.eq(device)))
      .set(devices::lastLoggedIn.eq(chrono::Utc::now()))
      .execute(connection)
      .await?;
    Ok(())
  }
}
