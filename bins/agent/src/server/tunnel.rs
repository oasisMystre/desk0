use std::time::Duration;

use futures::{SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tokio_util::sync::CancellationToken;

use crate::{
  api::Api,
  server::{
    identity::Identity,
    server::{AgentServer, AgentServerConfig},
    traits::Tunnel,
  },
};

pub struct AgentTunnel {
  api: Api,
  identity: Identity,
  config: AgentTunnelConfig,
}

#[derive(Clone)]
pub struct AgentTunnelConfig {
  addr: String,
}

impl AgentTunnel {
  pub fn new(addr: String, identity: Identity, api: Api) -> Self {
    Self {
      api,
      identity,
      config: AgentTunnelConfig { addr },
    }
  }

  fn websocket_url(&self, token: &str) -> String {
    let addr = self.config.addr.trim_end_matches('/');
    let addr = addr
      .strip_prefix("https://")
      .map(|host| format!("wss://{host}"))
      .or_else(|| {
        addr
          .strip_prefix("http://")
          .map(|host| format!("ws://{host}"))
      })
      .unwrap_or_else(|| addr.to_owned());

    format!("{addr}/ws?token={token}")
  }

  async fn connect(&self) -> anyhow::Result<()> {
    let token = self.api.device.get_token(&self.identity.device_id).await?;
    let url = self.websocket_url(&token);

    tracing::info!("connecting agent WebSocket");
    let (websocket, _) = connect_async(url).await?;
    tracing::info!("agent WebSocket connected");
    let (mut websocket_writer, mut websocket_reader) = websocket.split();

    let (mut rdp_peer, rdp_stream) = tokio::io::duplex(1024 * 1024);

    let rdp_cancellation = CancellationToken::new();
    let rdp = AgentServer::handle(
      AgentServerConfig::default(),
      rdp_stream,
      rdp_cancellation.clone(),
    );
    tokio::pin!(rdp);
    let mut bytes = [0; 16 * 1024];

    let disconnect_reason = loop {
      tokio::select! {
        result = websocket_reader.next() => match result {
          Some(Ok(Message::Binary(bytes))) => {
            tracing::debug!(bytes = bytes.len(), "server -> agent RDP bytes");
            rdp_peer.write_all(&bytes).await?
          },
          Some(Ok(Message::Ping(bytes))) => websocket_writer.send(Message::Pong(bytes)).await?,
          Some(Ok(Message::Close(_))) | None => break "server closed agent WebSocket",
          Some(Ok(Message::Pong(_) | Message::Text(_) | Message::Frame(_))) => {},
          Some(Err(error)) => {
            tracing::warn!(?error, "agent WebSocket read failed");
            break "agent WebSocket read failed";
          }
        },
        result = rdp_peer.read(&mut bytes) => {
          let count = result?;
          if count == 0 {
            break "local RDP stream closed";
          }
          tracing::debug!(bytes = count, "agent -> server RDP bytes");
          websocket_writer
            .send(Message::Binary(bytes[..count].to_vec().into()))
            .await?;
        },
        result = &mut rdp => {
          result?;
          break "RDP server stopped";
        },
      }
    };

    rdp_cancellation.cancel();
    tracing::info!(reason = disconnect_reason, "closing agent WebSocket");
    if let Err(error) = websocket_writer.close().await {
      tracing::debug!(?error, "agent WebSocket close handshake failed");
    }

    Ok(())
  }
}

#[async_trait::async_trait]
impl Tunnel for AgentTunnel {
  async fn run(&self, cancellation_token: CancellationToken) -> anyhow::Result<()> {
    while !cancellation_token.is_cancelled() {
      match self.connect().await {
        Ok(()) => eprintln!("Agent WebSocket closed; reconnecting in 5 seconds..."),
        Err(error) => eprintln!("Agent WebSocket connection failed: {error:#}"),
      }

      tokio::select! {
        _ = cancellation_token.cancelled() => break,
        _ = tokio::time::sleep(Duration::from_secs(5)) => {},
      }
    }

    Ok(())
  }
}
