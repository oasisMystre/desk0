use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};

use crate::routes::ws::{
  auth::Claims,
  registry::{DeviceInfo, DeviceRegistry},
};

pub async fn handle_agent(mut socket: WebSocket, claims: Claims, registry: Arc<DeviceRegistry>) {
  let (agent_sender, mut agent_receiver) = tokio::sync::mpsc::unbounded_channel();
  let agent_session = registry.register(
    DeviceInfo {
      org_id: claims.org_id.clone(),
      device_id: claims.device_id.clone(),
    },
    agent_sender,
  );

  loop {
    tokio::select! {
      Some(message) = agent_receiver.recv() => {
        if let Err(error) = socket.send(message).await {
          tracing::error!(?error, "agent WebSocket write failed");
          break;
        }
      }
      result = socket.recv() => match result {
        Some(Ok(Message::Binary(bytes))) => {
          tracing::debug!(bytes = bytes.len(), prefix = ?&bytes[..bytes.len().min(8)], "agent -> relay RDP bytes");
          agent_session.publish_inbound(Message::Binary(bytes));
        }
        Some(Ok(Message::Ping(bytes))) => {
          if let Err(error) = socket.send(Message::Pong(bytes)).await {
            tracing::debug!(?error, "agent pong failed");
            break;
          }
        }
        Some(Ok(Message::Close(close))) => {
          tracing::debug!(?close, "agent closed websocket");
          break;
        }
        Some(Ok(message)) => tracing::debug!(?message, "ignored agent WebSocket message"),
        Some(Err(error)) => {
          tracing::error!(?error, "agent WebSocket error");
          break;
        }
        None => break,
      },
      else => break,
    }
  }

  tracing::info!(org = %claims.org_id, device = %claims.device_id, "agent WebSocket disconnected");
  registry.unregister(&claims.org_id, &claims.device_id);
}
