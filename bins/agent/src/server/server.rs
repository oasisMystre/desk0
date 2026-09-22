use std::net::SocketAddr;
use std::sync::Arc;

use copypaste::CliprdrHandler;
use enigo::Settings;
use ironrdp::server::{RdpServer, TransportTls};
use tokio::{
  io::{AsyncRead, AsyncWrite},
  net::TcpListener,
};
use tokio_util::sync::CancellationToken;
use tokio_rustls::{TlsAcceptor, rustls::{self, pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer}}};

use crate::{
  handlers::{display::DisplayHandler, input::InputHandler},
  server::traits::Server,
};

#[derive(Clone)]
pub struct AgentServer {
  config: AgentServerConfig,
}

#[derive(Clone, Default)]
pub struct AgentServerConfig {
  pub addr: Option<SocketAddr>,
}

impl AgentServer {
  pub fn new(addr: SocketAddr) -> Self {
    Self {
      config: AgentServerConfig { addr: Some(addr) },
    }
  }
}

#[async_trait::async_trait(?Send)]
impl Server for AgentServer {
  async fn run(&self, cancellation_token: CancellationToken) -> anyhow::Result<()> {
    let local = tokio::task::LocalSet::new();
    local
      .run_until(async move {
        let addr = self
          .config
          .addr
          .ok_or_else(|| anyhow::anyhow!("Server bind address is not configured"))?;
        let listener = TcpListener::bind(addr).await?;
        println!("RDP server listening on {addr}");

        loop {
          tokio::select! {
            _ = cancellation_token.cancelled() => {
              println!("RDP server shutting down...");
              break;
            }
            result = listener.accept() => {
              let (stream, peer_addr) = result?;
              println!("Accepted connection from {peer_addr}");

              let config = self.config.clone();
              let cancellation_token = cancellation_token.clone();

              tokio::task::spawn_local(async move {
                tracing::info!(%peer_addr, "RDP connection accepted");
                if let Err(error) = Self::handle(config, stream, cancellation_token).await {
                  eprintln!("Connection handler error for {peer_addr}: {error:?}");
                }
                tracing::info!(%peer_addr, "RDP connection handler stopped");
              });
            }
          }
        }

        Ok::<_, anyhow::Error>(())
      })
      .await?;

    Ok(())
  }
}

impl AgentServer {
  fn tls_acceptor() -> anyhow::Result<TlsAcceptor> {
    let identity = rcgen::generate_simple_self_signed(vec!["desk0-agent".to_owned()])?;
    let certificate = CertificateDer::from(identity.cert.der().to_vec());
    let private_key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
      identity.signing_key.serialize_der(),
    ));
    let config = rustls::ServerConfig::builder()
      .with_no_client_auth()
      .with_single_cert(vec![certificate], private_key)?;

    Ok(TlsAcceptor::from(Arc::new(config)))
  }

  pub async fn handle<S>(
    config: AgentServerConfig,
    stream: S,
    cancellation_token: CancellationToken,
  ) -> anyhow::Result<()>
  where
    S: AsyncRead + AsyncWrite + Sync + Send + Unpin + 'static,
  {
    let display_handler = DisplayHandler::new()?;
    let input_handler = InputHandler::new(&Settings::default())?;

    let mut handles = vec![];

    let addr = config
      .addr
      .unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 0)));

    let tls_acceptor = Self::tls_acceptor()?;
    let server = RdpServer::builder()
      .with_addr(addr)
      .with_tls(tls_acceptor)
      .with_input_handler(input_handler)
      .with_display_handler(display_handler.clone())
      .with_cliprdr_factory(Some(Box::new(CliprdrHandler::new())));

    let mut server = server.build();

    handles.push(tokio::task::spawn_local(async move {
      match display_handler.capture().await {
        Ok(()) => println!("Display handler shut down"),
        Err(error) => {
          eprintln!("Display handler error: {error:?}");
        }
      }
    }));

    handles.push(tokio::task::spawn_local(async move {
      if let Err(error) = server.run_connection_with(stream, TransportTls::AlreadyDone).await {
        eprintln!("Connection dropped: {error:?}");
      }
    }));

    tokio::select! {
      _ = cancellation_token.cancelled() => {
        for handle in &handles {
          handle.abort();
        }
      }
      _ = futures::future::join_all(&mut handles) => {}
    }

    Ok(())
  }
}
