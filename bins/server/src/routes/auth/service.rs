use std::{collections::BTreeMap, sync::Arc};

use base64::{Engine, engine::general_purpose::STANDARD};
use chrono::Utc;
use deadpool_redis::redis::{AsyncTypedCommands, RedisResult, Script, aio::MultiplexedConnection};
use diesel_async::AsyncConnection;
use jwt::SignWithKey;
use rand::{RngExt, rng};
use resend_rs::types::CreateEmailBaseOptions;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use uuid::Uuid;

use super::{
  dto::{
    EmailSignUpResponse, RefreshTokenResponse, RevokeSessionResponse, SessionResponse,
    VerifyOtpResponse,
  },
  error::AuthError,
  models::{Organization, RefreshToken, Session},
  repository::{AuthRepository, RefreshRotationOutcome},
};
use crate::{
  context::Context, error::SharedError, routes::organizations::repository::OrganizationRepository,
};

const SESSION_DURATION_DAYS: i64 = 30;
const OTP_EXPIRES_IN_SECS: u64 = 5 * 60;
const ACCESS_TOKEN_TTL_SECS: i64 = 24 * 60 * 60;

const EMAIL_OTP_TEMPLATE: &str = include_str!("../../../templates/email-otp.html");

const CHECK_OTP_SCRIPT: &str = r#"
local otpHash = redis.call("GET", KEYS[1])
local attempts = redis.call("INCR", KEYS[2])
if attempts == 1 then
  redis.call("EXPIRE", KEYS[2], ARGV[1])
end
return {otpHash, attempts}
"#;

pub struct AuthService;

impl AuthService {
  pub fn generate_otp(length: u8) -> String {
    let min = 10_u128.pow(length as u32 - 1);
    let max = 10u128.pow(length as u32);

    rng().random_range(min..max).to_string()
  }

  pub fn get_redis_otp_key(email: &str) -> String {
    format!("otp:{}", email.trim().to_lowercase())
  }

  pub fn get_redis_otp_attempts_key(email: &str) -> String {
    format!("otp-attempts:{}", email.trim().to_lowercase())
  }

  pub async fn check_otp_attempts(
    connection: &mut MultiplexedConnection,
    otp_key: &str,
    attempts_key: &str,
    attempt_expiry_seconds: u64,
  ) -> RedisResult<(Option<String>, u64)> {
    Script::new(CHECK_OTP_SCRIPT)
      .key(otp_key)
      .key(attempts_key)
      .arg(attempt_expiry_seconds)
      .invoke_async(connection)
      .await
  }

  pub async fn email_signup(
    context: &Context,
    email: String,
  ) -> Result<EmailSignUpResponse, AuthError> {
    let otp = Self::generate_otp(6);
    let mut redis = context.get_redis().await?;

    redis
      .set_ex(
        Self::get_redis_otp_key(&email),
        format!("{:x}", Sha256::digest(otp.as_bytes())),
        OTP_EXPIRES_IN_SECS,
      )
      .await
      .map_err(|error| {
        tracing::error!(?error, "failed to save otp to redis");
        AuthError::OtpStorageFailed
      })?;

    drop(redis);

    let email_options = CreateEmailBaseOptions::new(
      "Desk0 <noreply@desk0.xyz>",
      [email],
      format!("{} is your Desk0 sign-in code", otp),
    )
    .with_html(
      &EMAIL_OTP_TEMPLATE
        .replace("__OTP_CODE__", &otp)
        .replace("__APP_NAME", "Desk0")
        .replace(
          "__OTP_EXPIRES_IN_MINUTES__",
          &(OTP_EXPIRES_IN_SECS / 60).to_string(),
        ),
    );

    let _ = context
      .resend
      .emails
      .send(email_options)
      .await
      .map_err(|error| {
        tracing::error!(?error, "sending email error");
        AuthError::OtpSendFailed
      })?;

    Ok(EmailSignUpResponse {
      message: "otp sent".into(),
    })
  }

  pub async fn verify_otp(
    context: &Context,
    email: String,
    otp: String,
    user_agent: Option<String>,
    ip_address: Option<ipnet::IpNet>,
  ) -> Result<VerifyOtpResponse, AuthError> {
    let mut redis = context.get_redis().await?;

    let otp_redis_key = Self::get_redis_otp_key(&email);
    let otp_attempts_key = Self::get_redis_otp_attempts_key(&email);
    let candidate_hash = format!("{:x}", Sha256::digest(otp.as_bytes()));

    let (otp_hash, attempts) = Self::check_otp_attempts(
      &mut redis,
      &otp_redis_key,
      &otp_attempts_key,
      OTP_EXPIRES_IN_SECS,
    )
    .await
    .map_err(|error| {
      tracing::error!(?error, "check otp attempts error");
      SharedError::RedisOperation
    })?;

    if attempts > 5 {
      return Err(AuthError::TooManyOtpAttempts);
    }

    if let Some(otp_hash) = otp_hash {
      let is_valid = otp_hash.len() == candidate_hash.len()
        && bool::from(otp_hash.as_bytes().ct_eq(candidate_hash.as_bytes()));

      if is_valid {
        let mut postgres = context.get_pg().await?;

        let response = postgres
          .transaction::<_, AuthError, _>(async |connection| {
            let (user, is_new_user) = AuthRepository::upsert_user(connection, &email)
              .await
              .map_err(|error| {
                tracing::error!(?error, "failed to get user");
                AuthError::UserCreateFailed
              })?;

            if is_new_user {
              OrganizationRepository::create(
                connection,
                &Organization {
                  user: user.id,
                  created_at: Utc::now(),
                  id: uuid::Uuid::new_v4(),
                  name: Some("default".to_owned()),
                },
              )
              .await
              .map_err(|error| {
                tracing::error!(?error, "failed to create organization for new user");
                AuthError::OrganizationCreateFailed
              })?;
            }

            let now = Utc::now();
            let session_id = Uuid::new_v4();
            let session_expires_at = now + chrono::Duration::days(SESSION_DURATION_DAYS);

            let raw_token_bytes: [u8; 32] = rng().random();
            let refresh_token_string = STANDARD.encode(raw_token_bytes);
            let token_hash = Sha256::digest(&raw_token_bytes).to_vec();

            AuthRepository::create_session_with_refresh_token(
              connection,
              Session {
                id: session_id,
                user: user.id,
                user_agent,
                ip_address,
                expires_at: session_expires_at,
                created_at: now,
                last_used_at: Some(now),
                revoked_at: None,
              },
              RefreshToken {
                id: Uuid::new_v4(),
                session: session_id,
                token: token_hash,
                expires_at: session_expires_at,
                created_at: now,
                consumed_at: None,
                revoked_at: None,
              },
            )
            .await
            .map_err(|error| {
              tracing::error!(
                ?error,
                "failed to create user session and refresh token in transaction"
              );
              AuthError::SessionCreateFailed
            })?;

            let now = Utc::now().timestamp();

            let mut claims = BTreeMap::new();
            claims.insert("role", "user".into());
            claims.insert("iat", now.to_string());
            claims.insert("sub", user.id.to_string());
            claims.insert("sid", session_id.to_string());
            claims.insert("exp", (now + ACCESS_TOKEN_TTL_SECS).to_string());

            deadpool_redis::redis::pipe()
              .cmd("DEL")
              .arg(otp_redis_key)
              .cmd("DEL")
              .arg(otp_attempts_key)
              .exec_async(&mut redis)
              .await
              .map_err(|error| {
                tracing::error!(?error, "failed to delete otp from redis");
                SharedError::RedisOperation
              })?;

            let token = claims
              .sign_with_key(&Arc::unwrap_or_clone(context.auth_key.clone()))
              .map_err(|error| {
                tracing::error!(?error, "claim sign error");
                AuthError::TokenSignFailed
              })?;

            Ok(VerifyOtpResponse {
              token,
              user,
              refresh_token: refresh_token_string,
              session_id,
            })
          })
          .await?;

        return Ok(response);
      }
    }

    Err(AuthError::InvalidOtp)
  }

  pub async fn refresh_token(
    context: &Context,
    raw_refresh_token: String,
  ) -> Result<RefreshTokenResponse, AuthError> {
    let token_bytes = STANDARD
      .decode(raw_refresh_token.trim())
      .map_err(|_| AuthError::InvalidRefreshToken)?;
    let token_hash = Sha256::digest(&token_bytes).to_vec();

    let now = Utc::now();

    let new_raw_token: [u8; 32] = rng().random();
    let new_refresh_token = STANDARD.encode(new_raw_token);
    let new_token_hash = Sha256::digest(&new_raw_token).to_vec();

    let new_expires_at = now + chrono::Duration::days(SESSION_DURATION_DAYS);

    let mut postgres = context.get_pg().await?;

    let candidate = AuthRepository::find_refresh_token_by_hash(&mut postgres, &token_hash)
      .await
      .map_err(|error| match error {
        diesel::result::Error::NotFound => AuthError::InvalidRefreshToken,
        _ => {
          tracing::error!(?error, "failed to query refresh token");
          AuthError::Shared(SharedError::DatabaseOperation)
        }
      })?;

    let unix_now = now.timestamp();
    let mut claims = BTreeMap::new();

    claims.insert("role", "user".into());
    claims.insert("iat", unix_now.to_string());
    claims.insert("sid", candidate.session.to_string());

    let session = AuthRepository::find_session_by_id(&mut postgres, candidate.session)
      .await
      .map_err(|error| match error {
        diesel::result::Error::NotFound => AuthError::SessionNotFound,
        _ => {
          tracing::error!(?error, "failed to query session");
          AuthError::Shared(SharedError::DatabaseOperation)
        }
      })?;

    claims.insert("sub", session.user.to_string());
    claims.insert("exp", (unix_now + ACCESS_TOKEN_TTL_SECS).to_string());

    let access_token = claims
      .sign_with_key(&Arc::unwrap_or_clone(context.auth_key.clone()))
      .map_err(|error| {
        tracing::error!(?error, "failed to sign access token");
        AuthError::TokenSignFailed
      })?;

    match AuthRepository::rotate_refresh_token(
      &mut postgres,
      &token_hash,
      now,
      RefreshToken {
        id: Uuid::new_v4(),
        token: new_token_hash,
        session: candidate.session,
        created_at: now,
        consumed_at: None,
        revoked_at: None,
        expires_at: new_expires_at,
      },
    )
    .await
    .map_err(|error| {
      tracing::error!(?error, "refresh-token rotation failed");
      AuthError::Shared(SharedError::DatabaseOperation)
    })? {
      RefreshRotationOutcome::Rotated => Ok(RefreshTokenResponse {
        token: access_token,
        refresh_token: new_refresh_token,
      }),
      RefreshRotationOutcome::Reused => {
        tracing::warn!(
            session = %candidate.session,
            "consumed refresh token reuse detected; session revoked"
        );
        Err(AuthError::SessionRevoked)
      }
      RefreshRotationOutcome::Invalid => Err(AuthError::InvalidRefreshToken),
      RefreshRotationOutcome::SessionRevoked => Err(AuthError::SessionRevoked),
    }
  }

  pub async fn get_sessions(
    context: &Context,
    user_id: Uuid,
    current_session_id: Option<Uuid>,
  ) -> Result<Vec<SessionResponse>, AuthError> {
    let mut postgres = context.get_pg().await?;

    let sessions = AuthRepository::find_sessions_by_user(&mut postgres, user_id)
      .await
      .map_err(|error| {
        tracing::error!(?error, "failed to query user sessions");
        AuthError::SessionFetchFailed
      })?;

    let result = sessions
      .into_iter()
      .map(|session: Session| SessionResponse {
        id: session.id,
        user_agent: session.user_agent,
        ip_address: session.ip_address.map(|ip| ip.to_string()),
        created_at: session.created_at,
        expires_at: session.expires_at,
        last_used_at: session.last_used_at,
        is_current: current_session_id == Some(session.id),
      })
      .collect();

    Ok(result)
  }

  pub async fn revoke_session(
    context: &Context,
    user_id: Uuid,
    session_id: Uuid,
  ) -> Result<RevokeSessionResponse, AuthError> {
    let mut postgres = context.get_pg().await?;
    let now = Utc::now();

    AuthRepository::revoke_session_and_tokens(&mut postgres, session_id, user_id, now)
      .await
      .map_err(|error| match error {
        diesel::result::Error::NotFound => AuthError::SessionNotFound,
        _ => {
          tracing::error!(?error, "failed to revoke session and tokens in transaction");
          AuthError::SessionRevokeFailed
        }
      })?;

    Ok(RevokeSessionResponse {
      message: "session revoked successfully".into(),
    })
  }

  pub async fn logout(
    context: &Context,
    user_id: Uuid,
    session_id: Option<Uuid>,
  ) -> Result<RevokeSessionResponse, AuthError> {
    if let Some(session_id) = session_id {
      Self::revoke_session(context, user_id, session_id).await
    } else {
      Ok(RevokeSessionResponse {
        message: "logged out".into(),
      })
    }
  }
}
