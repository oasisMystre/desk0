use diesel::{
  BoolExpressionMethods, ExpressionMethods, HasQuery, QueryDsl, SelectableHelper,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use flow::{device::Device, devices};

use super::models::UpdateDeviceChangeset;

pub struct DeviceRepository;

impl DeviceRepository {
  pub async fn find_by_organization(
    connection: &mut AsyncPgConnection,
    org_id: uuid::Uuid,
  ) -> Result<Vec<Device>, diesel::result::Error> {
    Device::query()
      .filter(devices::organization.eq(org_id))
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
    device_id: uuid::Uuid,
    org_id: uuid::Uuid,
    changeset: &UpdateDeviceChangeset,
  ) -> Result<Device, diesel::result::Error> {
    diesel::update(
      devices::table.filter(
        devices::id
          .eq(device_id)
          .and(devices::organization.eq(org_id)),
      ),
    )
    .set(changeset)
    .returning(Device::as_returning())
    .get_result(connection)
    .await
  }

  pub async fn delete(
    connection: &mut AsyncPgConnection,
    device_id: uuid::Uuid,
    org_id: uuid::Uuid,
  ) -> Result<Device, diesel::result::Error> {
    diesel::delete(
      devices::table.filter(
        devices::id
          .eq(device_id)
          .and(devices::organization.eq(org_id)),
      ),
    )
    .returning(Device::as_returning())
    .get_result(connection)
    .await
  }

  #[allow(dead_code)]
  pub async fn get_device_organization(
    connection: &mut AsyncPgConnection,
    device_id: uuid::Uuid,
  ) -> Result<uuid::Uuid, diesel::result::Error> {
    devices::table
      .select(devices::organization)
      .find(device_id)
      .get_result(connection)
      .await
  }

  pub async fn find_by_id(
    connection: &mut AsyncPgConnection,
    device_id: uuid::Uuid,
  ) -> Result<Device, diesel::result::Error> {
    Device::query()
      .filter(devices::id.eq(device_id))
      .first(connection)
      .await
  }

  pub async fn update_last_logged_in(
    connection: &mut AsyncPgConnection,
    device_id: uuid::Uuid,
  ) -> Result<(), diesel::result::Error> {
    diesel::update(devices::table.filter(devices::id.eq(device_id)))
      .set(devices::lastLoggedIn.eq(chrono::Utc::now()))
      .execute(connection)
      .await?;
    Ok(())
  }
}
