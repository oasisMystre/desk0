use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures::StreamExt;
use tokio::{
  sync::mpsc::UnboundedReceiver,
  time::{Duration, timeout},
};

use crate::routes::ws::{
  auth::{Auth, Claims},
  registry::{AgentSession, DeviceInfo, DeviceRegistry},
};

const RESET_RDP_SESSION: &[u8] = b"desk0:reset-rdp-session";

pub async fn handle_viewer(
  mut socket: WebSocket,
  auth: Auth,
  registry: Arc<DeviceRegistry>,
  mut presence: UnboundedReceiver<DeviceInfo>,
  certificate: Arc<Vec<u8>>,
) {
  let (request, claims) = match read_rdcleanpath_request(&mut socket, &auth).await {
    Ok(value) => value,
    Err(error) => {
      tracing::debug!(?error, "RDCleanPath viewer authentication failed");
      return;
    }
  };

  if claims.role.as_deref() != Some("organization") {
    tracing::warn!(role = ?claims.role, "rejected non organization RDP token");
    return;
  }

  let agent = loop {
    if let Some(agent) = registry.get_by_org_and_device_id(&claims.org_id, &claims.device_id) {
      break agent;
    }

    tokio::select! {
      Some(info) = presence.recv() => {
        if info.org_id == claims.org_id && info.device_id == claims.device_id {
          if let Some(agent) = registry.get_by_org_and_device_id(&claims.org_id, &claims.device_id) {
            break agent;
          }
        }
      }
      result = socket.recv() => match result {
        Some(Ok(Message::Ping(bytes))) => { if socket.send(Message::Pong(bytes)).await.is_err() { return; } }
        Some(Ok(Message::Close(_))) | None | Some(Err(_)) => return,
        Some(Ok(_)) => {}
      }
    }
  };

  let mut agent_reader = agent.subscribe_inbound();
  if let Err(error) = negotiate_rdcleanpath(
    &mut socket,
    &agent,
    &mut agent_reader,
    &certificate,
    request,
  )
  .await
  {
    tracing::debug!(?error, "RDCleanPath negotiation failed");
    return;
  }

  loop {
    tokio::select! {
      result = socket.next() => match result {
        Some(Ok(Message::Binary(bytes))) => {
          tracing::debug!(bytes = bytes.len(), "viewer -> agent RDP bytes");
          if agent.sender.send(Message::Binary(bytes)).is_err() { break; }
        }
        Some(Ok(Message::Ping(bytes))) => { if socket.send(Message::Pong(bytes)).await.is_err() { break; } }
        Some(Ok(Message::Close(_)))  => {
            tracing::info!("rdp disconnected");
            break;
        },
        None => break,
        Some(Err(error)) => {
            tracing::error!(?error, "rdp disconnected");
            break;
        },
        Some(Ok(_)) => {}
      },
      result = agent_reader.recv() => match result {
        Some(message) => {
          if let Err(error) = socket.send(message).await {
              tracing::error!(?error, "rdp disconnected");
              break;
          }
        }
        None => break,
      }
    }
  }

  if let Err(error) = agent
    .sender
    .send(Message::Ping(RESET_RDP_SESSION.to_vec().into()))
  {
    tracing::warn!(?error, "resetting agent RDP session failed");
  }
}

async fn read_rdcleanpath_request(
  socket: &mut WebSocket,
  auth: &Auth,
) -> anyhow::Result<(Vec<u8>, Claims)> {
  loop {
    let Some(message) = socket.next().await else {
      anyhow::bail!("viewer closed during RDCleanPath authentication")
    };

    match message? {
      Message::Ping(bytes) => socket.send(Message::Pong(bytes)).await?,
      Message::Binary(bytes) => {
        let pdu = ironrdp_rdcleanpath::RDCleanPathPdu::from_der(&bytes)?;
        let ironrdp_rdcleanpath::RDCleanPath::Request {
          proxy_auth,
          x224_connection_request,
          ..
        } = pdu.into_enum()?
        else {
          anyhow::bail!("expected RDCleanPath request")
        };
        let claims = auth.authenticate(&proxy_auth)?;
        return Ok((x224_connection_request.as_bytes().to_vec(), claims));
      }
      Message::Pong(_) | Message::Text(_) => {}
      Message::Close(_) => anyhow::bail!("viewer closed during RDCleanPath authentication"),
    }
  }
}

async fn negotiate_rdcleanpath(
  socket: &mut WebSocket,
  agent: &AgentSession,
  agent_reader: &mut tokio::sync::mpsc::UnboundedReceiver<Message>,
  certificate: &[u8],
  request: Vec<u8>,
) -> anyhow::Result<()> {
  agent.sender.send(Message::Binary(request.into()))?;

  let response = timeout(Duration::from_secs(15), async {
    loop {
      let Some(message) = agent_reader.recv().await else {
        anyhow::bail!("agent closed during RDCleanPath negotiation")
      };
      if let Message::Binary(bytes) = message {
        break Ok::<_, anyhow::Error>(bytes);
      }
    }
  })
  .await
  .map_err(|_| anyhow::anyhow!("timed out waiting 15s for agent RDCleanPath response"))??;

  let response = ironrdp_rdcleanpath::RDCleanPath::Response {
    x224_connection_response: ironrdp_rdcleanpath::der::asn1::OctetString::new(response)
      .map_err(|error| anyhow::anyhow!("encode X.224 response: {error:?}"))?,
    server_cert_chain: vec![
      ironrdp_rdcleanpath::der::asn1::OctetString::new(certificate)
        .map_err(|error| anyhow::anyhow!("encode server certificate: {error:?}"))?,
    ],
    server_addr: "127.0.0.1".to_owned(),
  }
  .into_pdu()
  .to_der()
  .map_err(|error| anyhow::anyhow!("encode RDCleanPath response: {error:?}"))?;

  socket.send(Message::Binary(response.into())).await?;

  Ok(())
}
