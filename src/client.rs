use std::sync::Arc;

use enigo::Settings;
use ironrdp::server::RdpServer;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use tokio::net::TcpListener;
use tokio_rustls::rustls::ServerConfig;
use tokio_util::sync::CancellationToken;

use crate::handlers::{display::DisplayHandler, input::InputHandler};

pub fn create_tls_config() -> anyhow::Result<Arc<ServerConfig>> {
  let subject_alt_names: Vec<String> = vec!["localhost".into(), "127.0.0.1".into()];
  let cert = rcgen::generate_simple_self_signed(subject_alt_names)?;

  let cert_der = CertificateDer::from(cert.cert.der().to_vec());
  let key_der = PrivateKeyDer::Pkcs8(cert.signing_key.serialize_der().into());

  let server_config = ServerConfig::builder()
    .with_no_client_auth()
    .with_single_cert(vec![cert_der], key_der)?;

  Ok(Arc::new(server_config))
}
pub async fn run_server(addr: &str, cancellation_token: CancellationToken) -> anyhow::Result<()> {
  let socket_addr = addr.parse::<std::net::SocketAddr>()?;
  let listener = TcpListener::bind(socket_addr).await?;
  println!("RDP Server Listening on {socket_addr}");

  let tls_config = create_tls_config()?;
  let tls_acceptor = tokio_rustls::TlsAcceptor::from(tls_config);

  tokio::task::LocalSet::new()
    .run_until(async move {
      loop {
        let (stream, peer_addr) = listener.accept().await?;
        println!("Accepted connection from {}", peer_addr);

        let tls_acceptor = tls_acceptor.clone();
        let cancellation_token = cancellation_token.clone();

        tokio::task::spawn_local(async move {
          if let Err(error) =
            handle_connection(stream, tls_acceptor, cancellation_token.clone()).await
          {
            eprintln!("Connection handler error for {peer_addr}: {error:?}");
          }
        });
      }

      #[allow(unreachable_code)]
      Ok::<(), anyhow::Error>(())
    })
    .await?;

  Ok(())
}

async fn handle_connection(
  stream: tokio::net::TcpStream,
  tls_acceptor: tokio_rustls::TlsAcceptor,
  cancellation_token: CancellationToken,
) -> anyhow::Result<()> {
  let peer_addr = stream.peer_addr()?;

  let display_handler = DisplayHandler::new()?;
  let input_handler = InputHandler::new(&Settings::default())?;

  let mut server = RdpServer::builder()
    .with_addr(peer_addr)
    .with_tls(tls_acceptor)
    .with_input_handler(input_handler)
    .with_display_handler(display_handler.clone())
    .build();

  let capture_handle = tokio::task::spawn_local(async move {
    match display_handler.capture().await {
      Ok(()) => println!("Display handler shut down"),
      Err(error) => {
        eprintln!("Display handler error: {error:?}");
      }
    }
  });

  tokio::select! {
      _ = cancellation_token.cancelled() => {
          capture_handle.abort();
      },
      result = server.run_connection(stream) => {
          if let Err(error) = result {
              eprintln!("Connection dropped: {error:?}");
          }
      }
  }

  Ok(())
}
