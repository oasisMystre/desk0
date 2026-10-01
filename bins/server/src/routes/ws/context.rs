use std::sync::Arc;

use crate::{
  context::Context,
  routes::ws::{auth::Auth, hub::Hub},
};

#[derive(Clone)]
pub struct WsContext {
  pub auth: Arc<Auth>,
  pub hub: Arc<Hub>,
  pub certificate: Arc<Vec<u8>>,
}

impl WsContext {
  pub fn from_env(context: Arc<Context>) -> anyhow::Result<Self> {
    let certificate = rcgen::generate_simple_self_signed(vec!["desk0".to_owned()])?;
    let certificate = Arc::new(certificate.cert.der().to_vec());

    Ok(Self {
      certificate,
      auth: Arc::new(Auth::from_env()?),
      hub: Arc::new(Hub::new(context, "desk0:hub")),
    })
  }
}
