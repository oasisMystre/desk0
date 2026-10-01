use std::{sync::Arc, time::Duration};

use axum::extract::ws::{Message, WebSocket};
use futures::StreamExt;
use tokio::time::{MissedTickBehavior, interval};
use tokio_util::sync::CancellationToken;

use crate::routes::ws::{
  auth::Claims,
  hub::{Hub, HubConnectionState},
  rdp::RESET_RDP_SESSION,
};

pub struct AgentHandler {
  hub: Arc<Hub>,
}

impl AgentHandler {
  pub fn new(hub: Arc<Hub>) -> Self {
    Self { hub }
  }

  pub async fn handle(&mut self, mut socket: WebSocket, claims: Claims) -> anyhow::Result<()> {
    let upstream = &format!("agent:{}:{}", claims.org_id, claims.device_id);
    let inbound = &format!("rdp:{}:{}", claims.org_id, claims.device_id);

    // this first to prevent missing message from rdp
    let mut inbound_receiver = self.hub.subscribe(inbound).await;

    self
      .hub
      .register(&upstream, HubConnectionState::Connecting)
      .await?;

    if !self.wait_for_rdp(&mut socket, inbound).await? {
      self.hub.unsubscribe(inbound).await;
      self.hub.unregister(upstream).await?;

      return Ok(());
    }

    tracing::info!("agent waited for rdp");

    let cancellation_token = CancellationToken::new();
    let presence_task = tokio::spawn({
      let hub = self.hub.clone();
      let session = upstream.clone();
      let cancellation_token = cancellation_token.clone();

      async move {
        let mut ticker = interval(Duration::from_secs(5));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
          tokio::select! {
              _ = cancellation_token.cancelled() => break,
              _ = ticker.tick() => {
                if let Err(error) = hub.register(&session, HubConnectionState::Connected).await {
                    tracing::warn!(?error, session = %session, "refreshing RDP presence failed");
                }
              }
          }
        }
      }
    });

    loop {
      tokio::select! {
          result = socket.next() => {
              match result {
                  Some(Ok(Message::Binary(bytes))) => {
                      if let Err(error) = self
                          .hub
                          .send(&upstream, Message::Binary(bytes))
                          .await
                      {
                          tracing::error!(?error, "agent upstream send failed");
                          break;
                      }
                  }
                  Some(Ok(Message::Ping(bytes))) => {
                      if socket.send(Message::Pong(bytes)).await.is_err() {
                          break;
                      }
                  }
                  Some(Ok(Message::Close(_))) => {
                      tracing::info!("agent disconnected");
                      break;
                  }
                  Some(Err(error)) => {
                      tracing::error!(?error, "agent disconnected");
                      break;
                  }
                  Some(Ok(_)) => {}
                  None => break,
              }
          }
          result = inbound_receiver.recv() => {
              match result {
                  Some(message) => {
                      if let Message::Ping(bytes) = &message {
                          tracing::info!(?bytes, ?RESET_RDP_SESSION, "resetting session");
                      }
                      if let Err(error) = socket.send(message).await {
                          tracing::error!(?error, "agent disconnected");
                          break;
                      }
                  }
                  None => break,
              }
          }
      }
    }

    cancellation_token.cancel();
    if let Err(error) = presence_task.await {
      tracing::warn!(?error, "agent presence task failed");
    }

    self.hub.unsubscribe(inbound).await;
    if let Err(error) = self.hub.unregister(upstream).await {
      tracing::warn!(?error, "removing agent presence failed");
    }

    Ok(())
  }

  async fn wait_for_rdp(&self, socket: &mut WebSocket, inbound: &str) -> anyhow::Result<bool> {
    loop {
      tokio::select! {
          result = self.hub.is_registered(inbound) => {
              let state = result?;
              if matches!(state, Some(HubConnectionState::Connected)) {
                  return Ok(true);
              }
          }
          result = socket.recv() => {
              match result {
                  Some(Ok(Message::Ping(bytes))) => {
                      socket.send(Message::Pong(bytes)).await?;
                  }
                  Some(Ok(Message::Close(_))) | None => {
                      return Ok(false);
                  }
                  Some(Err(error)) => {
                      tracing::debug!(?error, "agent websocket disconnected");
                      return Ok(false);
                  }
                  Some(Ok(_)) => {}
              }
          }
      }
    }
  }
}
