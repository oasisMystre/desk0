use std::sync::Arc;

use crate::routes::ws::{auth::Auth, registry::DeviceRegistry};

#[derive(Clone)]
pub struct WsContext {
  pub auth: Auth,
  pub registry: Arc<DeviceRegistry>,
  pub certificate: Arc<Vec<u8>>,
}

impl WsContext {
  pub fn from_env() -> anyhow::Result<Self> {
    let certificate = rcgen::generate_simple_self_signed(vec!["desk0".to_owned()])?;
    let certificate = Arc::new(certificate.cert.der().to_vec());

    Ok(Self {
      certificate,
      auth: Auth::from_env()?,
      registry: Arc::new(DeviceRegistry::new()),
    })
  }
}
