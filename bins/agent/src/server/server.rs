use std::net::SocketAddr;

use enigo::Settings;
use ironrdp::server::RdpServer;
use tokio::{
  io::{AsyncRead, AsyncWrite},
  net::TcpListener,
};
use tokio_util::sync::CancellationToken;

use crate::{
  handlers::{clipdr::CliprdrHandler, display::DisplayHandler, input::InputHandler},
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

impl Server for AgentServer {
  fn run(&self, cancellation_token: CancellationToken) {
    let _ = tokio::task::LocalSet::new().run_until(async move {
      let listener = TcpListener::bind(self.config.addr.unwrap()).await?;
      println!("RDP server listening on {}", self.config.addr.unwrap());

      loop {
        tokio::select! {
            _ = cancellation_token.cancelled() => {
                break;
            }
            result = listener.accept() => {
                let (stream, peer_addr) = result?;
                println!("Accepted connection from {peer_addr}");

                let config = self.config.clone();
                let cancellation_token = cancellation_token.clone();

                tokio::task::spawn_local(async move {
                    if let Err(error) = Self::handle(config, stream, cancellation_token).await {
                        eprintln!("Connection handler error for {peer_addr}: {error:?}");
                    }
                });
            }
        }
      }

      Ok::<_, anyhow::Error>(())
    });
  }
}

impl AgentServer {
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

    let server = RdpServer::builder()
      .with_addr(addr)
      .with_no_security()
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
      if let Err(error) = server.run_connection(stream).await {
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
