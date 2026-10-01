use std::{sync::Arc, time::Duration};

use axum::extract::ws::{Message, WebSocket};
use futures::StreamExt;
use tokio::time::{MissedTickBehavior, interval};
use tokio_util::sync::CancellationToken;

use crate::routes::ws::{auth::Claims, hub::Hub};

pub struct AgentHandler {
  hub: Arc<Hub>,
}

impl AgentHandler {
  pub fn new(hub: Arc<Hub>) -> Self {
    Self { hub }
  }

  pub async fn handle(&mut self, mut socket: WebSocket, claims: Claims) -> anyhow::Result<()> {
    // read all message from agent and forward to rdp
    let upstream = &format!("agent:{}:{}", claims.org_id, claims.device_id);
    // read all message from rdp and forward to agent
    let inbound = &format!("rdp:{}:{}", claims.org_id, claims.device_id);

    self.hub.register(upstream).await?;
    let mut inbound_receiver = self.hub.subscribe(inbound).await;
    
    self.wait_for_rdp(&mut socket, inbound).await?;

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
                if let Err(error) = hub.register(&session).await {
                    tracing::warn!(?error, session = %session, "refreshing agent presence failed");
                }
              }
          }
        }
      }
    });


    loop {
      tokio::select! {
        result = socket.next() => match result {
          None => break,
          Some(Ok(Message::Binary(bytes))) => {
            if let Err(error) =  self.hub.send(&upstream, Message::Binary(bytes)).await {
                tracing::error!(?error, "upstream send failed");
            }
          }
          Some(Ok(Message::Ping(bytes))) => {
            if socket.send(Message::Pong(bytes)).await.is_err() {
              break;
            }
          }
          Some(Ok(Message::Close(_)))  => {
            tracing::info!("agent disconnected");
            break;
          },
          Some(Err(error)) => {
              tracing::error!(?error, "agent disconnected");
              break;
          },
          Some(Ok(_)) => {}
        },
        result = inbound_receiver.recv() => match result {
          Some(message) => {
            if let Err(error) = socket.send(message).await {
                tracing::error!(?error, "agent disconnected");
                break;
            }
          }
          None => break,
        }
      }
    }

    cancellation_token.cancel();

    if let Err(error) = presence_task.await {
      tracing::warn!(?error, "RDP presence task failed");
    }

    self.hub.unsubscribe(inbound).await;

    if let Err(error) = self.hub.unregister(upstream).await {
      tracing::warn!(?error, "removing RDP presence failed");
    }

    Ok(())
  }

  pub async fn wait_for_rdp(&self, socket: &mut WebSocket, inbound: &str) -> anyhow::Result<()> {
    let mut interval = tokio::time::interval(Duration::from_secs(1));

    loop {
      tokio::select! {
          _ = interval.tick() => {
              if self.hub.is_registered(inbound).await? {
                  return Ok(());
              }
          }
          result = socket.recv() => {
              match result {
                  Some(Ok(Message::Ping(bytes))) => {
                      socket.send(Message::Pong(bytes)).await?;
                  }
                  Some(Ok(Message::Close(_))) | None => {
                      return Ok(());
                  }
                  Some(Err(error)) => {
                      return Err(error.into());
                  }
                  Some(Ok(_)) => {}
              }
          }
      }
    }
  }
}
