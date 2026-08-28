use ironrdp::server::{
  DesktopSize, DisplayUpdate, KeyboardEvent, MouseEvent, RdpServer, RdpServerDisplay,
  RdpServerDisplayUpdates, RdpServerInputHandler, ServerEvent, SoundServerFactory,
  RdpsndServerHandler, RdpsndServerMessage,
};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc::{Sender, UnboundedSender};
use tokio::sync::Mutex;
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
  rx: Option<tokio::sync::mpsc::Receiver<DisplayUpdate>>,
}

struct DisplayUpdates {
  rx: tokio::sync::mpsc::Receiver<DisplayUpdate>,
}

#[derive(Clone, Debug)]
pub struct AudioBackend {
  sender: Arc<Mutex<Option<UnboundedSender<ServerEvent>>>>,
}

impl AudioBackend {
  pub fn new() -> Self {
    Self {
      sender: Arc::new(Mutex::new(None)),
    }
  }

  pub fn get_sender(&self) -> Option<UnboundedSender<ServerEvent>> {
    self.sender.blocking_lock().clone()
  }
}

impl ironrdp::server::ServerEventSender for AudioBackend {
  fn set_sender(&mut self, sender: UnboundedSender<ServerEvent>) {
    *self.sender.blocking_lock() = Some(sender);
  }
}

impl SoundServerFactory for AudioBackend {
  fn build_backend(&self) -> Box<dyn RdpsndServerHandler> {
    Box::new(AudioHandler::new(self.clone()))
  }
}

#[derive(Debug)]
struct AudioHandler {
  backend: AudioBackend,
  negotiated: Option<ironrdp_rdpsnd::server::NegotiatedFormat>,
}

impl AudioHandler {
  fn new(backend: AudioBackend) -> Self {
    Self {
      backend,
      negotiated: None,
    }
  }
}

impl RdpsndServerHandler for AudioHandler {
  fn get_formats(&self) -> &[ironrdp_rdpsnd::pdu::AudioFormat] {
    static FORMATS: [ironrdp_rdpsnd::pdu::AudioFormat; 2] = [
      ironrdp_rdpsnd::pdu::AudioFormat {
        format: ironrdp_rdpsnd::pdu::WaveFormat::PCM,
        n_channels: 2,
        n_samples_per_sec: 48000,
        n_avg_bytes_per_sec: 48000 * 2 * 2,
        n_block_align: 4,
        bits_per_sample: 16,
        data: None,
      },
      ironrdp_rdpsnd::pdu::AudioFormat {
        format: ironrdp_rdpsnd::pdu::WaveFormat::PCM,
        n_channels: 2,
        n_samples_per_sec: 44100,
        n_avg_bytes_per_sec: 44100 * 2 * 2,
        n_block_align: 4,
        bits_per_sample: 16,
        data: None,
      }
    ];
    &FORMATS
  }

  fn choose_format<'a>(&mut self, common: &'a [ironrdp_rdpsnd::server::NegotiatedFormat]) -> Option<&'a ironrdp_rdpsnd::server::NegotiatedFormat> {
    common.first()
  }

  fn start(&mut self, format: &ironrdp_rdpsnd::server::NegotiatedFormat) -> Result<(), Box<dyn ironrdp_rdpsnd::server::RdpsndError>> {
    self.negotiated = Some(format.clone());
    Ok(())
  }

  fn stop(&mut self) {
    self.negotiated = None;
  }
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
impl RdpServerDisplayUpdates for DisplayUpdates {
  async fn next_update(&mut self) -> anyhow::Result<Option<DisplayUpdate>> {
    Ok(self.rx.recv().await)
  }
}

impl Display {
  fn new(width: u16, height: u16, rx: tokio::sync::mpsc::Receiver<DisplayUpdate>) -> Self {
    Self {
      width,
      height,
      rx: Some(rx),
    }
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

async fn run_capture_loop(
  width: u16,
  height: u16,
  sender: Sender<DisplayUpdate>,
  audio_sender: Option<UnboundedSender<ServerEvent>>,
) {
  crate::platform::display::Display::capture(width, height, sender, audio_sender).await;
}

async fn handle_connection(
  stream: tokio::net::TcpStream,
  tls_acceptor: tokio_rustls::TlsAcceptor,
) -> anyhow::Result<()> {
  let peer_addr = stream.peer_addr()?;

  let (width, height) = crate::platform::display::Display::get_desktop_size();

  let (tx, rx) = tokio::sync::mpsc::channel::<DisplayUpdate>(16);

  let input_handler = InputHandler::new(width, height);
  let display_handler = Display::new(width, height, rx);
  let audio_backend = AudioBackend::new();

  let mut server = RdpServer::builder()
    .with_addr(peer_addr)
    .with_tls(tls_acceptor)
    .with_input_handler(input_handler)
    .with_display_handler(display_handler)
    .with_sound_factory(Some(Box::new(audio_backend.clone())))
    .build();

  let (audio_tx, mut audio_rx) = tokio::sync::mpsc::unbounded_channel::<ServerEvent>();
  
  let audio_backend_clone = audio_backend.clone();
  tokio::task::spawn_local(async move {
    while let Some(event) = audio_rx.recv().await {
      if let Some(sender) = audio_backend_clone.get_sender() {
        let _ = sender.send(event);
      }
    }
  });

  let _capture_task = tokio::task::spawn_local(run_capture_loop(width, height, tx, Some(audio_tx)));

  println!("Running RDP connection for {}...", peer_addr);
  server.run_connection(stream).await?;
  println!("RDP connection for {} disconnected.", peer_addr);

  Ok(())
}
