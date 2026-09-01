use std::{
  pin::Pin,
  time::{Duration, Instant},
};

use desk0_protocol::{
  ConnectTunnelRequest, ConnectTunnelResponse, Desk0HubService, HeartbeatAck, RegisterAck,
  RegisterAckSuccess, Relay, RelayData, Session, StartSession, connect_tunnel_request,
  connect_tunnel_response, create_relay_bridge, register_ack::Ack,
};
use futures::{Stream, StreamExt};
use tokio::{
  io::{DuplexStream, copy_bidirectional},
  net::TcpStream,
  sync::{mpsc::unbounded_channel, oneshot::channel},
  time::timeout,
};
use tonic::{Request, Response, Status, Streaming};

use crate::{
  context::Context,
  registry::{DeviceInfo, DeviceRegistry},
};

#[derive(Clone)]
pub struct Desk0Hub {
  context: Context,
  registry: DeviceRegistry,
}

impl Desk0Hub {
  pub fn new(context: Context, registry: DeviceRegistry) -> Self {
    Self { context, registry }
  }

  pub async fn bridge_client(
    &self,
    mut client_stream: TcpStream,
    target_device_id: String,
  ) -> anyhow::Result<()> {
    let client_peer = client_stream.peer_addr()?;
    let agent = match self.registry.get(&target_device_id) {
      Some(sender) => sender,
      None => {
        anyhow::bail!("Target device {target_device_id} is offline or not found");
      }
    };

    let session_id = uuid::Uuid::new_v4().to_string();
    let (downstream_tx, downstream_rx) = channel::<DuplexStream>();

    self.registry.queue(session_id.clone(), downstream_tx).await;

    tracing::info!(session_id, ?client_peer, "Initiating gRPC RDP session");

    agent.sender.send(ConnectTunnelResponse {
      msg: Some(connect_tunnel_response::Msg::StartSession(StartSession {
        session_id: session_id.clone(),
      })),
    })?;

    let mut duplex_stream = timeout(Duration::from_secs(30), downstream_rx).await??;

    let (from_client, from_agent) =
      copy_bidirectional(&mut client_stream, &mut duplex_stream).await?;

    tracing::debug!(
      "Session {session_id} finished: {from_client} bytes from viewer, {from_agent} bytes from agent"
    );

    Ok(())
  }
}

#[tonic::async_trait]
impl Desk0HubService for Desk0Hub {
  type ConnectTunnelStream =
    Pin<Box<dyn Stream<Item = Result<ConnectTunnelResponse, Status>> + Send>>;
  type RelayStreamStream = Pin<Box<dyn Stream<Item = Result<RelayData, Status>> + Send>>;

  async fn connect_tunnel(
    &self,
    request: Request<Streaming<ConnectTunnelRequest>>,
  ) -> Result<Response<Self::ConnectTunnelStream>, Status> {
    let registry = self.registry.clone();
    let mut inbound_stream = request.into_inner();

    let first_message = match inbound_stream.next().await {
      Some(Ok(message)) => message,
      Some(Err(error)) => {
        tracing::error!(?error, "Stream error");
        return Err(Status::internal(format!("Stream error: {error:?}")));
      }
      None => return Err(Status::invalid_argument("Empty stream")),
    };

    let register_info = match first_message.msg {
      Some(connect_tunnel_request::Msg::Register(info)) => info,
      other => {
        tracing::warn!(?other, "not register message");
        return Err(Status::invalid_argument(format!(
          "Expected Register message as first stream element, got {other:?}"
        )));
      }
    };

    tracing::info!(
      register_info.device_id,
      register_info.org_id,
      register_info.hostname,
      register_info.username,
      "gRPC agent registered"
    );

    let (outbound_tx, outbound_rx) = unbounded_channel::<ConnectTunnelResponse>();
    registry.register(
      DeviceInfo {
        os: register_info.os,
        last_seen: Instant::now(),
        org_id: register_info.org_id,
        version: register_info.version,
        hostname: register_info.hostname,
        username: register_info.username,
        device_id: register_info.device_id.clone(),
      },
      outbound_tx.clone(),
    );

    if let Err(error) = outbound_tx.send(ConnectTunnelResponse {
      msg: Some(connect_tunnel_response::Msg::RegisterAck(RegisterAck {
        ack: Some(Ack::Success(RegisterAckSuccess {
          session_token: uuid::Uuid::new_v4().to_string(),
        })),
      })),
    }) {
      tracing::error!(?error, "send register ack error");
      return Err(Status::internal(error.to_string()));
    };

    let device_id = register_info.device_id.clone();
    tokio::spawn(async move {
      while let Some(message) = inbound_stream.next().await {
        match message {
          Err(error) => {
            tracing::error!(device_id, ?error, "gRpc agent tunnel error",);
            break;
          }
          Ok(message) => match message.msg {
            Some(desk0_protocol::connect_tunnel_request::Msg::Heartbeat(data)) => {
              registry.heartbeat(&data.device_id);
              outbound_tx
                .send(ConnectTunnelResponse {
                  msg: Some(connect_tunnel_response::Msg::HeartbeatAck(HeartbeatAck {})),
                })
                .ok();
            }
            Some(desk0_protocol::connect_tunnel_request::Msg::SessionStatus(data)) => {}
            _ => todo!(),
          },
        }
      }

      tracing::info!(device_id, "gRPC agent tunnel disconnected");

      registry.unregister(&device_id);
    });

    let outbound_stream = tokio_stream::wrappers::UnboundedReceiverStream::new(outbound_rx).map(Ok);
    Ok(Response::new(Box::pin(outbound_stream)))
  }

  async fn relay_stream(
    &self,
    request: Request<Streaming<RelayData>>,
  ) -> Result<Response<Self::RelayStreamStream>, Status> {
    let registry = self.registry.clone();
    let mut inbound_stream = request.into_inner();

    let first_chunk = match inbound_stream.next().await {
      Some(Ok(chunk)) => chunk,
      Some(Err(error)) => {
        tracing::error!(?error, "Stream error");
        return Err(Status::internal(format!("Stream error: {error:?}")));
      }
      None => return Err(Status::invalid_argument("Empty relay stream")),
    };

    tracing::info!(
      first_chunk.org_id,
      first_chunk.device_id,
      first_chunk.session_id,
      "gRPC Relay stream attached"
    );

    let Relay {
      stream,
      inbound,
      outbound,
    } = create_relay_bridge(
      Session {
        org_id: first_chunk.org_id.clone(),
        device_id: first_chunk.device_id.clone(),
        session_id: first_chunk.session_id.clone(),
      },
      65536,
    );

    let session_id = first_chunk.session_id.clone();
    if !first_chunk.data.is_empty() {
      inbound.send(first_chunk).ok();
    }

    if !registry.dequeue(&session_id, stream).await {
      tracing::error!(
        session_id,
        "Failed to fulfill pending relay for session (expired or not found)"
      );
      return Err(Status::not_found("Pending relay not found or expired"));
    }

    tokio::spawn(async move {
      while let Some(item) = inbound_stream.next().await {
        match item {
          Ok(chunk) => {
            if inbound.send(chunk).is_err() {
              tracing::info!(session_id, "inbound stream channel closed");
              break;
            }
          }
          Err(error) => {
            tracing::error!(session_id, ?error, "inbound stream error");
            break;
          }
        }
      }
    });

    let mapped_out = outbound.map(Ok);
    Ok(Response::new(Box::pin(mapped_out)))
  }
}
