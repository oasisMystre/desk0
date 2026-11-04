use std::time::Duration;

use base64::{Engine, engine::general_purpose::STANDARD};
use ed25519_dalek::Signer;
use futures::{SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tokio_util::sync::CancellationToken;

use crate::{
  api::{Api, device::CreateConnectionTokenRequest},
  server::{
    identity::Identity,
    server::{AgentServer, AgentServerConfig},
    traits::Tunnel,
  },
};

const RESET_RDP_SESSION: &[u8] = b"desk0:reset-rdp-session";

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

  async fn connect(&self, cancellation_token: CancellationToken) -> anyhow::Result<()> {
    let challenge = self
      .api
      .device
      .get_challenge(&self.identity.device_id)
      .await?;
    let signature = self
      .identity
      .signing_key()
      .sign(challenge.challenge.as_bytes());
    let token = self
      .api
      .device
      .get_token(
        &self.identity.device_id,
        CreateConnectionTokenRequest {
          challenge: challenge.challenge,
          signature: STANDARD.encode(signature.to_bytes()),
        },
      )
      .await?;
    let url = self.websocket_url(&token);

    tracing::info!("connecting agent WebSocket");
    let (websocket, _) = connect_async(url).await?;
    tracing::info!("agent WebSocket connected");
    let (mut websocket_writer, mut websocket_reader) = websocket.split();

    let mut bytes = [0; 16 * 1024];
    loop {
      let rdp_cancellation_token = cancellation_token.child_token();
      let (mut rdp_peer, rdp_stream) = tokio::io::duplex(1024 * 1024);

      let rdp = AgentServer::handle(
        AgentServerConfig {
          tunnel: true,
          ..Default::default()
        },
        rdp_stream,
        rdp_cancellation_token.clone(),
      );
      tokio::pin!(rdp);

      let reset = loop {
        tokio::select! {
          result = websocket_reader.next() => match result {
            Some(Ok(Message::Binary(bytes))) => {
              rdp_peer.write_all(&bytes).await?
            },
            Some(Ok(Message::Ping(bytes))) if bytes.as_ref() == RESET_RDP_SESSION => break true,
            Some(Ok(Message::Ping(bytes))) => websocket_writer.send(Message::Pong(bytes)).await?,
            Some(Ok(Message::Close(_))) | None => break false,
            Some(Ok(Message::Pong(_) | Message::Text(_) | Message::Frame(_))) => {},
            Some(Err(error)) => {
              tracing::warn!(?error, "agent WebSocket read failed");
              break false;
            }
          },
          result = rdp_peer.read(&mut bytes) => {
            let count = result?;
            if count == 0 {
              break false;
            }
            websocket_writer
              .send(Message::Binary(bytes[..count].to_vec().into()))
              .await?;
          },
          result = &mut rdp => {
            result?;
            break false;
          },
        }
      };

      rdp_cancellation_token.cancel();
      if reset {
        if let Err(error) = rdp.await {
          tracing::debug!(?error, "previous embedded RDP session stopped during reset");
        }
        tracing::info!("resetting embedded RDP session while keeping agent WebSocket");
        continue;
      }

      tracing::info!("closing agent WebSocket");
      if let Err(error) = websocket_writer.close().await {
        tracing::error!(?error, "agent WebSocket close handshake failed");
      }
      break;
    }

    Ok(())
  }
}

#[async_trait::async_trait]
impl Tunnel for AgentTunnel {
  async fn run(&self, cancellation_token: CancellationToken) -> anyhow::Result<()> {
    loop {
      let result = tokio::select! {
        _ = cancellation_token.cancelled() => break,
        result =  self.connect(cancellation_token.clone()) => result,
      };

      if cancellation_token.is_cancelled() {
        break;
      }

      match result {
        Ok(()) => tracing::info!("Agent WebSocket closed; reconnecting in 5 seconds..."),
        Err(error) => tracing::error!(?error, "Agent WebSocket connection failed"),
      }

      tokio::select! {
          _ = cancellation_token.cancelled() => break,
          _ = tokio::time::sleep(Duration::from_secs(5)) => {}
      }
    }

    Ok(())
  }
}
