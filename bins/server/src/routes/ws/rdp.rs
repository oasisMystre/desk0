use std::sync::Arc;

use anyhow::Context;
use axum::extract::ws::{Message, WebSocket};
use futures::StreamExt;
use tokio::{
  sync::mpsc::UnboundedReceiver,
  time::{Duration, Instant, MissedTickBehavior, interval, timeout},
};
use tokio_util::sync::CancellationToken;

use crate::routes::ws::{
  auth::{Auth, Claims},
  hub::{Hub, HubConnectionState},
};

pub const RESET_RDP_SESSION: &[u8] = b"desk0:reset-rdp-session";

pub struct RdpHandler {
  hub: Arc<Hub>,
  auth: Arc<Auth>,
  certificate: Arc<Vec<u8>>,
}

impl RdpHandler {
  pub fn new(hub: Arc<Hub>, auth: Arc<Auth>, certificate: Arc<Vec<u8>>) -> Self {
    Self {
      hub,
      auth,
      certificate,
    }
  }

  /// skip negotiation if agent is still connected
  /// reconnect if agent is in connecting state
  pub async fn handle(&mut self, mut socket: WebSocket) -> anyhow::Result<()> {
    let (request, claims) = self
      .read_rdcleanpath_request(&mut socket)
      .await
      .context("RDCleanPath viewer authentication failed")?;

    if claims.role.as_deref() != Some("organization") {
      anyhow::bail!("rejected non organization RDP token {:?}", claims.role);
    }

    // read all message from rdp and forward to agent
    let upstream = &format!("rdp:{}:{}", claims.org_id, claims.device_id);
    // read all message from agent and forward to rdp
    let inbound = &format!("agent:{}:{}", claims.org_id, claims.device_id);

    // this first to prevent missing message from agent
    let mut inbound_receiver = self.hub.subscribe(inbound).await;

    // mark rdp as connected immediately
    self
      .hub
      .register(upstream, HubConnectionState::Connected)
      .await?;

    let state = self.wait_for_agent(&mut socket, inbound).await?;
    tracing::info!(?state, "rdp state");
    if !matches!(
      state,
      Some(HubConnectionState::Connected | HubConnectionState::Connecting)
    ) {
      self.hub.unsubscribe(inbound).await;
      self.hub.unregister(upstream).await?;
      anyhow::bail!("Agent not connected");
    }

    if let Err(error) = self
      .negotiate_rdcleanpath(&mut socket, request, upstream, &mut inbound_receiver)
      .await
    {
      self.hub.unsubscribe(&inbound).await;
      self.hub.unregister(&upstream).await?;
      anyhow::bail!("RDCleanPath negotiation failed {error}");
    }

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
        result = socket.next() => match result {
          None => break,
          Some(Ok(Message::Binary(bytes))) => {
            if let Err(error) =  self.hub.send(&upstream, Message::Binary(bytes)).await {
                tracing::error!(?error, "rdp upstream send failed");
            }
          }
          Some(Ok(Message::Ping(bytes))) => {
            if socket.send(Message::Pong(bytes)).await.is_err() {
              break;
            }
          }
          Some(Ok(Message::Close(_))) => {
            tracing::info!("rdp disconnected gracefully");
            break;
          },
          Some(Err(error))  => {
              tracing::error!(?error, "rdp disconnected due to error");
          }
          Some(Ok(_)) => {}
        },
        result = inbound_receiver.recv() => match result {
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

    cancellation_token.cancel();
    if let Err(error) = presence_task.await {
      tracing::warn!(?error, "RDP presence task failed");
    }

    self.hub.unsubscribe(inbound).await;

    if let Err(error) = self.hub.unregister(upstream).await {
      tracing::warn!(?error, "removing RDP presence failed");
    }

    if let Err(error) = self
      .hub
      .send(&upstream, Message::Ping(RESET_RDP_SESSION.into()))
      .await
    {
      tracing::warn!(?error, "resetting agent RDP session failed");
    }

    tracing::info!("rdp disconnected");

    Ok(())
  }

  /// if agent is idle, give a grace period of 10s
  /// if agent is disconnected, give a grace period of 10s
  /// if agent is connecting, still wait but timeout 10s
  /// if agent is connected return
  async fn wait_for_agent(
    &self,
    socket: &mut WebSocket,
    inbound: &str,
  ) -> anyhow::Result<Option<HubConnectionState>> {
    let deadline = Instant::now() + Duration::from_secs(10);

    let mut last_state = None;

    loop {
      if Instant::now() > deadline {
        break Ok(last_state);
      }

      let state = self.hub.is_registered(inbound).await?;
      if let Some(state) = state {
        match state {
          HubConnectionState::Connected => return Ok(Some(state)),
          HubConnectionState::Connecting => return Ok(Some(state)),
          HubConnectionState::Idle | HubConnectionState::Disconnected => last_state = Some(state),
        }
      }

      let remaining = deadline.saturating_duration_since(Instant::now());
      let wait = remaining.min(Duration::from_millis(250));

      tokio::select! {
        result = tokio::time::sleep(wait) => result,
        result = socket.recv() => match result {
          Some(Ok( Message::Close(_))) | None => return Ok(None),
          Some(Ok(Message::Ping(bytes))) => {
            socket.send(Message::Pong(bytes)).await?;
          }
           Some(Err(error)) => return Err(error.into()),
          Some(Ok(_)) => {}
        }
      }
    }
  }

  async fn read_rdcleanpath_request(
    &self,
    socket: &mut WebSocket,
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
          let claims = self.auth.authenticate(&proxy_auth)?;
          return Ok((x224_connection_request.as_bytes().to_vec(), claims));
        }
        Message::Pong(_) | Message::Text(_) => {}
        Message::Close(_) => anyhow::bail!("viewer closed during RDCleanPath authentication"),
      }
    }
  }

  async fn negotiate_rdcleanpath(
    &mut self,
    socket: &mut WebSocket,
    request: Vec<u8>,
    inbound: &str,
    inbound_reader: &mut UnboundedReceiver<Message>,
  ) -> anyhow::Result<()> {
    let response = {
      self
        .hub
        .send(inbound, Message::Binary(request.into()))
        .await?;

      let response = timeout(Duration::from_secs(5), async {
        loop {
          let Some(message) = inbound_reader.recv().await else {
            anyhow::bail!("agent closed during RDCleanPath negotiation")
          };
          if let Message::Binary(bytes) = message {
            break Ok::<_, anyhow::Error>(bytes);
          }
        }
      })
      .await
      .map_err(|_| anyhow::anyhow!("timed out waiting 15s for agent RDCleanPath response"));

      match response {
        Ok(Ok(value)) => value.to_vec(),
        Ok(Err(error)) | Err(error) => {
          tracing::error!(?error, "negotiate error");

          let response = ironrdp_rdcleanpath::RDCleanPathPdu::new_tls_error(100)
            .to_der()
            .map_err(|error| anyhow::anyhow!("encode RDCleanPath response: {error:?}"))?;
          self
            .hub
            .send(inbound, Message::Binary(response.into()))
            .await?;

          return Ok(());
        }
      }
    };

    let response = ironrdp_rdcleanpath::RDCleanPath::Response {
      x224_connection_response: ironrdp_rdcleanpath::der::asn1::OctetString::new(
        response.into_boxed_slice(),
      )
      .map_err(|error| anyhow::anyhow!("encode X.224 response: {error:?}"))?,
      server_cert_chain: vec![
        ironrdp_rdcleanpath::der::asn1::OctetString::new(Arc::unwrap_or_clone(
          self.certificate.clone(),
        ))
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
}
