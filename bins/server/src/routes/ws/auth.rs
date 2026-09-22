use std::collections::BTreeMap;

use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use sha2::Sha256;
use shared::env::FromEnv;

use crate::env::Env;

#[derive(Clone)]
pub struct Auth {
  key: Hmac<Sha256>,
}

#[derive(Clone)]
pub struct Claims {
  pub org_id: String,
  pub device_id: String,
  pub role: Option<String>,
}

impl Auth {
  const MAX_TOKEN_LEN: usize = 4096;

  pub fn from_env() -> anyhow::Result<Self> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(Env::AuthToken.get()?.as_bytes())?;

    Ok(Self { key })
  }

  pub fn authenticate(&self, token: &str) -> anyhow::Result<Claims, jwt::Error> {
    if token.is_empty() || token.len() > Self::MAX_TOKEN_LEN {
      return Err(jwt::Error::Format);
    }

    let claims: BTreeMap<String, String> = token.verify_with_key(&self.key)?;
    let org_id = claims.get("sub").ok_or_else(|| jwt::Error::Format)?;
    let device_id = claims
      .get("device")
      .or_else(|| claims.get("device"))
      .ok_or_else(|| jwt::Error::Format)?;

    Ok(Claims {
      org_id: org_id.clone(),
      device_id: device_id.clone(),
      role: claims
        .get("role")
        .map(|role| role.trim_matches('"').to_ascii_lowercase()),
    })
  }
}
