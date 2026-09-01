use std::{collections::BTreeMap, time::Duration};

use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use sha2::Sha256;
use shared::env::FromEnv;
use tokio::{
  io::AsyncReadExt,
  net::{TcpListener, TcpStream},
  time::timeout,
};

use crate::env::Env;

#[derive(Debug)]
pub enum AuthError {
  Io(std::io::Error),
  Timeout,
  FrameTooLarge,
  InvalidJwt(jwt::error::Error),
}

impl From<std::io::Error> for AuthError {
  fn from(value: std::io::Error) -> Self {
    Self::Io(value)
  }
}

pub struct Auth {
  key: Hmac<Sha256>,
}

pub struct Claims {
  pub sub: String,
  pub device: String,
}

impl Auth {
  const MAX_TOKEN_LEN: usize = 4096;

  pub fn from_env() -> anyhow::Result<Self> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(Env::AuthToken.get()?.as_bytes())?;

    Ok(Self { key })
  }

  fn authenticate(&self, token: &str) -> anyhow::Result<Claims, jwt::Error> {
    let claims: BTreeMap<String, String> = token.verify_with_key(&self.key)?;
    let sub = &claims["sub"];
    let device = &claims["device"];

    Ok(Claims {
      sub: sub.clone(),
      device: device.clone(),
    })
  }

  pub async fn accept_authenticated(
    &self,
    listener: &TcpListener,
  ) -> anyhow::Result<(TcpStream, Claims), AuthError> {
    let (mut stream, peer) = listener.accept().await?;
    let result = async {
      let len = stream.read_u16().await? as usize;
      if len == 0 || len > Self::MAX_TOKEN_LEN {
        return Err(AuthError::FrameTooLarge);
      }

      let mut buffer = vec![0u8; len];
      stream.read_exact(&mut buffer).await?;

      Ok(buffer)
    };

    let token_bytes = timeout(Duration::from_secs(5), result)
      .await
      .map_err(|_| AuthError::Timeout)??;

    let token = String::from_utf8_lossy(&token_bytes);
    let claims = self.authenticate(&token).map_err(AuthError::InvalidJwt)?;

    tracing::debug!(
      "authenticated {} to connect to {} from {peer}",
      claims.sub,
      claims.device
    );

    Ok((stream, claims))
  }
}
