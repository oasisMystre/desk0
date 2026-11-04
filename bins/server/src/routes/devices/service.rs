use std::{collections::BTreeMap, sync::Arc};

use base64::{Engine, engine::general_purpose::STANDARD};
use chrono::Utc;
use deadpool_redis::redis::AsyncCommands;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use jwt::SignWithKey;
use rand::{RngExt, rng};
use subtle::ConstantTimeEq;

use super::{
  dto::{
    ChallengeResponse, CreateConnectionTokenRequest, CreateConnectionTokenResponse, PartialDevice,
  },
  error::DeviceError,
  models::{Device, UpdateDeviceChangeset},
  repository::DeviceRepository,
};
use crate::{context::Context, error::SharedError, routes::devices::dto::CreateDevice};

pub const CHALLENGE_EXPIRES_IN_SECS: u64 = 5 * 60;

pub struct DeviceService;

impl DeviceService {
  pub fn get_redis_challenge_key(device: &uuid::Uuid) -> String {
    format!("device:challenge:{}", device)
  }

  pub async fn get_devices(
    context: &Context,
    org_id: uuid::Uuid,
  ) -> Result<Vec<Device>, DeviceError> {
    let mut connection = context.get_pg().await?;

    DeviceRepository::find_by_organization(&mut connection, org_id)
      .await
      .map_err(|error| {
        tracing::error!(?error, "failed to fetch devices");
        DeviceError::FetchFailed
      })
  }

  pub async fn create_device(
    context: &Context,
    CreateDevice { pubkey, device }: CreateDevice,
  ) -> Result<Device, DeviceError> {
    let pubkey_bytes: [u8; 32] = pubkey.as_slice().try_into().map_err(|error| {
      tracing::error!(?error, "invalid pubkey");
      DeviceError::InvalidPublicKey
    })?;

    VerifyingKey::from_bytes(&pubkey_bytes).map_err(|_| DeviceError::InvalidPublicKey)?;

    let now = Utc::now();
    let device = Device {
      pubkey,
      os: device.os,
      created_at: now,
      last_logged_in: now,
      id: uuid::Uuid::new_v4(),
      version: device.version,
      metadata: device.metadata,
      hostname: device.hostname,
      username: device.username,
      organization: device.organization,
    };

    let mut connection = context.get_pg().await?;
    DeviceRepository::create(&mut connection, &device)
      .await
      .map_err(|error| {
        tracing::error!(?error, "failed to insert device");
        DeviceError::CreateFailed
      })
  }

  pub async fn update_device(
    context: &Context,
    device: uuid::Uuid,
    body: PartialDevice,
  ) -> Result<Device, DeviceError> {
    let mut connection = context.get_pg().await?;

    let changeset = UpdateDeviceChangeset {
      os: body.os,
      version: body.version,
      hostname: body.hostname,
      username: body.username,
      metadata: body.metadata,
      last_logged_in: Some(Utc::now()),
    };

    DeviceRepository::update(&mut connection, device, body.organization, &changeset)
      .await
      .map_err(|error| match error {
        diesel::result::Error::NotFound => DeviceError::NotFound,
        _ => {
          tracing::error!(?error, "failed to update device");
          DeviceError::UpdateFailed
        }
      })
  }

  pub async fn delete_device(
    context: &Context,
    device: uuid::Uuid,
    org: uuid::Uuid,
  ) -> Result<Device, DeviceError> {
    let mut connection = context.get_pg().await?;

    DeviceRepository::delete(&mut connection, device, org)
      .await
      .map_err(|error| match error {
        diesel::result::Error::NotFound => DeviceError::NotFound,
        _ => {
          tracing::error!(?error, "failed to delete device");
          DeviceError::DeleteFailed
        }
      })
  }

  pub async fn create_challenge(
    context: &Context,
    device_id: uuid::Uuid,
  ) -> Result<ChallengeResponse, DeviceError> {
    let mut connection = context.get_pg().await?;
    let _ = DeviceRepository::find_by_id(&mut connection, device_id)
      .await
      .map_err(|error| match error {
        diesel::result::Error::NotFound => DeviceError::NotFound,
        _ => {
          tracing::error!(?error, "failed to query device for challenge");
          DeviceError::FetchFailed
        }
      })?;

    let challenge_bytes: [u8; 32] = rng().random();
    let challenge = STANDARD.encode(challenge_bytes);

    let mut redis = context.get_redis().await?;
    let key = Self::get_redis_challenge_key(&device_id);

    let _: () = redis
      .set_ex(&key, &challenge, CHALLENGE_EXPIRES_IN_SECS)
      .await
      .map_err(|error| {
        tracing::error!(?error, "failed to save challenge in redis");
        DeviceError::ChallengeStorageFailed
      })?;

    Ok(ChallengeResponse {
      challenge,
      id: device_id,
      expires_in: CHALLENGE_EXPIRES_IN_SECS,
    })
  }

  pub async fn create_connection_token(
    context: &Context,
    device_id: uuid::Uuid,
    body: CreateConnectionTokenRequest,
  ) -> Result<CreateConnectionTokenResponse, DeviceError> {
    let mut redis = context.get_redis().await?;
    let key = Self::get_redis_challenge_key(&device_id);

    let challenge: Option<String> = redis.get(&key).await.map_err(|error| {
      tracing::error!(?error, "failed to get challenge from redis");
      SharedError::RedisOperation
    })?;

    let Some(challenge) = challenge else {
      return Err(DeviceError::InvalidChallenge);
    };

    let is_valid_challenge = challenge.len() == body.challenge.len()
      && bool::from(challenge.as_bytes().ct_eq(body.challenge.as_bytes()));

    if !is_valid_challenge {
      return Err(DeviceError::InvalidChallenge);
    }

    let _: () = redis.del(&key).await.map_err(|error| {
      tracing::error!(?error, "failed to delete challenge from redis");
      SharedError::RedisOperation
    })?;

    let mut connection = context.get_pg().await?;
    let device = DeviceRepository::find_by_id(&mut connection, device_id)
      .await
      .map_err(|error| match error {
        diesel::result::Error::NotFound => DeviceError::NotFound,
        _ => {
          tracing::error!(?error, "failed to fetch device for token creation");
          DeviceError::FetchFailed
        }
      })?;

    let pubkey_bytes: [u8; 32] = device
      .pubkey
      .as_slice()
      .try_into()
      .map_err(|_| DeviceError::InvalidPublicKey)?;

    let verifying_key =
      VerifyingKey::from_bytes(&pubkey_bytes).map_err(|_| DeviceError::InvalidPublicKey)?;

    let signature_bytes = STANDARD.decode(&body.signature).map_err(|error| {
      tracing::error!(?error, "failed to hex decode signature");
      DeviceError::InvalidSignature
    })?;

    let signature_array: [u8; 64] = signature_bytes
      .as_slice()
      .try_into()
      .map_err(|_| DeviceError::InvalidSignature)?;

    let signature = Signature::from_bytes(&signature_array);

    let is_verified = match verifying_key.verify(body.challenge.as_bytes(), &signature) {
      Ok(()) => true,
      Err(_) => {
        if let Ok(raw) = STANDARD.decode(&body.challenge) {
          verifying_key.verify(&raw, &signature).is_ok()
        } else {
          false
        }
      }
    };

    if !is_verified {
      return Err(DeviceError::InvalidSignature);
    }

    if let Err(error) = DeviceRepository::update_last_logged_in(&mut connection, device_id).await {
      tracing::warn!(?error, "failed to update device last_logged_in");
    }

    let now = Utc::now();
    let jti = uuid::Uuid::new_v4().to_string();
    let mut claims = BTreeMap::new();
    claims.insert("role", "agent".into());
    claims.insert("sub", device.organization.to_string());
    claims.insert("device", device_id.to_string());
    claims.insert("iat", now.timestamp().to_string());
    claims.insert("jti", jti.clone());

    let token = claims
      .sign_with_key(&Arc::unwrap_or_clone(context.auth_key.clone()))
      .map_err(|error| {
        tracing::error!(?error, "claim sign error");
        DeviceError::TokenSignFailed
      })?;

    crate::states::register_connection_token(context, &token, &jti).await?;

    Ok(CreateConnectionTokenResponse { token })
  }
}
