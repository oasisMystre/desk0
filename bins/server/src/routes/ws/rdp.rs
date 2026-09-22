use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures::StreamExt;
use tokio::{
  sync::mpsc::UnboundedReceiver,
  time::{Duration, timeout},
};

use crate::routes::ws::{
  auth::Claims,
  registry::{AgentSession, DeviceInfo, DeviceRegistry},
};

pub async fn handle_viewer(
  mut socket: WebSocket,
  claims: Claims,
  registry: Arc<DeviceRegistry>,
  mut presence: UnboundedReceiver<DeviceInfo>,
  certificate: Arc<Vec<u8>>,
) {
  tracing::info!(org = %claims.org_id, device = %claims.device_id, "viewer RDP WebSocket connected");
  let agent = match registry.get_by_org_and_device_id(&claims.org_id, &claims.device_id) {
    Some(agent) => agent,
    None => loop {
      tokio::select! {
        result = socket.recv() => match result {
          Some(Ok(Message::Ping(bytes))) => {
            if let Err(error) = socket.send(Message::Pong(bytes)).await {
              tracing::debug!(?error, "viewer pong failed while waiting for agent");
              return;
            }
          }
          Some(Ok(Message::Close(close))) => {
            tracing::debug!(?close, "viewer closed while waiting for agent");
            return;
          }
          Some(Ok(message)) => tracing::debug!(?message, "ignored viewer message while waiting for agent"),
          Some(Err(error)) => {
            tracing::debug!(?error, "viewer websocket closed while waiting for agent");
            return;
          }
          None => return,
        },
        result = presence.recv() => match result {
          Some(info) if info.org_id == claims.org_id && info.device_id == claims.device_id => {
            if let Some(agent) = registry.get_by_org_and_device_id(&claims.org_id, &claims.device_id) {
              break agent;
            }
          }
          Some(info) => tracing::debug!(org = %info.org_id, device = %info.device_id, "ignored presence update"),
          None => return,
        }
      }
    },
  };
  tracing::info!("agent found");
  let mut agent_reader = agent.subscribe_inbound();
  if let Err(error) =
    negotiate_rdcleanpath(&mut socket, &agent, &mut agent_reader, &certificate).await
  {
    tracing::debug!(?error, "RDCleanPath negotiation failed");
    return;
  }

  loop {
    tokio::select! {
      result = socket.next() => match result {
        Some(Ok(Message::Binary(bytes))) => {
          tracing::debug!(bytes = bytes.len(), "viewer -> agent RDP bytes");
          if let Err(error) = agent.sender.send(Message::Binary(bytes)) {
            tracing::debug!(?error, "agent websocket closed while sending viewer data");
            break;
          }
        }
        Some(Ok(Message::Ping(bytes))) => {
          if let Err(error) = socket.send(Message::Pong(bytes)).await {
            tracing::debug!(?error, "viewer pong failed");
            break;
          }
        }
        Some(Ok(Message::Close(close))) => {
          tracing::debug!(?close, "viewer closed websocket");
          break;
        }
        Some(Ok(message)) => tracing::debug!(?message, "ignored viewer message"),
        Some(Err(error)) => {
          tracing::debug!(?error, "viewer websocket closed");
          break;
        }
        None => break,
      },
      result = agent_reader.recv() => match result {
        Some(message) => {
          if let Message::Binary(ref bytes) = message {
            tracing::debug!(bytes = bytes.len(), "agent -> viewer RDP bytes");
          }
          if let Err(error) = socket.send(message).await {
            tracing::debug!(?error, "viewer write failed");
            break;
          }
        }
        None => break,
      },
      _ = presence.recv() => {}
    }
  }
}

async fn negotiate_rdcleanpath(
  socket: &mut WebSocket,
  agent: &AgentSession,
  agent_reader: &mut tokio::sync::mpsc::UnboundedReceiver<Message>,
  certificate: &[u8],
) -> anyhow::Result<()> {
  let request = loop {
    let Some(message) = socket.next().await else {
      anyhow::bail!("viewer closed during RDCleanPath negotiation")
    };
    match message? {
      Message::Binary(bytes) => match ironrdp_rdcleanpath::RDCleanPathPdu::detect(&bytes) {
        ironrdp_rdcleanpath::DetectionResult::Detected { total_length, .. }
          if total_length == bytes.len() =>
        {
          break ironrdp_rdcleanpath::RDCleanPathPdu::from_der(&bytes)
            .map_err(|error| anyhow::anyhow!("decode RDCleanPath request: {error:?}"))?;
        }
        _ => anyhow::bail!("invalid RDCleanPath request"),
      },
      Message::Ping(bytes) => socket.send(Message::Pong(bytes)).await?,
      Message::Close(_) => anyhow::bail!("viewer closed during RDCleanPath negotiation"),
      Message::Pong(_) | Message::Text(_) => {}
    }
  };

  tracing::info!("sending back");

  let request = match request.into_enum()? {
    ironrdp_rdcleanpath::RDCleanPath::Request {
      x224_connection_request,
      ..
    } => x224_connection_request.as_bytes().to_vec(),
    _ => anyhow::bail!("expected RDCleanPath request"),
  };

  agent.sender.send(Message::Binary(request.into()))?;
  tracing::info!("RDCleanPath request forwarded to agent; waiting for response");
  
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
  tracing::debug!(
    bytes = response.len(),
    "RDCleanPath response received from agent"
  );

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
