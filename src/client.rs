use ironrdp::server::{
  DesktopSize, DisplayUpdate, KeyboardEvent, MouseEvent, RdpServer, RdpServerDisplay,
  RdpServerDisplayUpdates, RdpServerInputHandler, RdpsndServerHandler, RdpsndServerMessage,
  ServerEvent, SoundServerFactory,
};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::net::TcpListener;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_rustls::rustls::ServerConfig;

use crate::platform::traits::Display as _;
use crate::platform::traits::Keyboard as _;
use crate::platform::traits::Mouse as _;

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

#[allow(dead_code)]
struct InputHandler {
  width: u16,
  height: u16,
}

struct Display {
  width: u16,
  height: u16,
  rx: UnboundedReceiver<DisplayUpdate>,
}

impl InputHandler {
  fn new(width: u16, height: u16) -> Self {
    Self { width, height }
  }
}

impl RdpServerInputHandler for InputHandler {
  fn keyboard(&mut self, event: KeyboardEvent) {
    #[cfg(target_os = "windows")]
    crate::platform::windows::keyboard::Keyboard::event(event);
  }

  fn mouse(&mut self, event: MouseEvent) {
    #[cfg(target_os = "windows")]
    crate::platform::windows::mouse::Mouse::event(self.width, self.height, event);
  }
}

#[async_trait::async_trait]
impl RdpServerDisplayUpdates for Display {
  async fn next_update(&mut self) -> anyhow::Result<Option<DisplayUpdate>> {
    Ok(self.rx.recv().await)
  }
}

impl Display {
  fn new(width: u16, height: u16, rx: UnboundedReceiver<DisplayUpdate>) -> Self {
    Self { rx, width, height }
  }
}

#[async_trait::async_trait]
impl RdpServerDisplay for Display {
  async fn size(&mut self) -> DesktopSize {
    DesktopSize {
      width: self.width,
      height: self.height,
    }
  }

  async fn updates(&mut self) -> anyhow::Result<Box<dyn RdpServerDisplayUpdates>> {
    if let Some(rx) = self.rx.take() {
      Ok(Box::new(DisplayUpdates { rx }))
    } else {
      anyhow::bail!("Display updates channel already taken");
    }
  }
}

pub async fn run_server(addr: &str) -> anyhow::Result<()> {
  let socket_addr = addr.parse::<std::net::SocketAddr>()?;
  let listener = TcpListener::bind(socket_addr).await?;
  println!("RDP Server Listening on {}", socket_addr);

  let tls_config = create_tls_config()?;
  let tls_acceptor = tokio_rustls::TlsAcceptor::from(tls_config);

  tokio::task::LocalSet::new()
    .run_until(async move {
      loop {
        let (stream, peer_addr) = listener.accept().await?;
        println!("Accepted connection from {}", peer_addr);

        let tls_acceptor = tls_acceptor.clone();

        tokio::task::spawn_local(async move {
          if let Err(err) = handle_connection(stream, tls_acceptor).await {
            eprintln!("Connection handler error for {}: {:?}", peer_addr, err);
          }
        });
      }

      #[allow(unreachable_code)]
      Ok::<(), anyhow::Error>(())
    })
    .await?;

  Ok(())
}

async fn run_capture_loop(width: u16, height: u16, sender: UnboundedSender<DisplayUpdate>) {
  #[cfg(target_os = "windows")]
  crate::platform::windows::display::Display::capture(width, height, sender).await;
}

async fn handle_connection(
  stream: tokio::net::TcpStream,
  tls_acceptor: tokio_rustls::TlsAcceptor,
) -> anyhow::Result<()> {
  let peer_addr = stream.peer_addr()?;

  let (width, height) = crate::platform::windows::display::Display::size();

  let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<DisplayUpdate>();

  let input_handler = InputHandler::new(width, height);
  let display_handler = Display::new(width, height, rx);

  let mut server = RdpServer::builder()
    .with_addr(peer_addr)
    .with_tls(tls_acceptor)
    .with_input_handler(input_handler)
    .with_display_handler(display_handler)
    .build();

  let _capture_task = tokio::task::spawn_local(run_capture_loop(width, height, tx));

  println!("Running RDP connection for {}...", peer_addr);
  server.run_connection(stream).await?;
  println!("RDP connection for {} disconnected.", peer_addr);

  Ok(())
}
