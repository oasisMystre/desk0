use std::{pin::Pin, time::Instant};

use desk0_protocol::{
  AgentMessage, Desk0Hub, RegisterAck, RelayData, ServerMessage, agent_message,
  create_relay_bridge, server_message,
};
use futures::{Stream, StreamExt as _};
use tokio::sync::mpsc;
use tonic::{Request, Response, Status, Streaming};
use uuid::Uuid;

use crate::registry::{DeviceInfo, DeviceRegistry};

pub struct Desk0HubService {
  registry: DeviceRegistry,
}

impl Desk0HubService {
  pub fn new(registry: DeviceRegistry) -> Self {
    Self { registry }
  }
}

#[tonic::async_trait]
impl Desk0Hub for Desk0HubService {
  type ConnectTunnelStream = Pin<Box<dyn Stream<Item = Result<ServerMessage, Status>> + Send>>;
  type RelayStreamStream = Pin<Box<dyn Stream<Item = Result<RelayData, Status>> + Send>>;

  async fn connect_tunnel(
    &self,
    request: Request<Streaming<AgentMessage>>,
  ) -> Result<Response<Self::ConnectTunnelStream>, Status> {
    let mut in_stream = request.into_inner();
    let registry = self.registry.clone();

    let first_msg = match in_stream.next().await {
      Some(Ok(msg)) => msg,
      Some(Err(e)) => return Err(Status::internal(format!("Stream error: {e:?}"))),
      None => return Err(Status::invalid_argument("Empty stream")),
    };

    let register_info = match first_msg.msg {
      Some(agent_message::Msg::Register(r)) => r,
      other => {
        return Err(Status::invalid_argument(format!(
          "Expected Register message as first stream element, got: {other:?}"
        )));
      }
    };

    println!(
      "gRPC Agent registered: device={}, org={}, host={:?}, user={:?}",
      register_info.device_id, register_info.org_id, register_info.hostname, register_info.username
    );

    let (tx_server, rx_server) = mpsc::unbounded_channel::<ServerMessage>();

    let device_info = DeviceInfo {
      org_id: register_info.org_id.clone(),
      device_id: register_info.device_id.clone(),
      hostname: register_info.hostname,
      os: register_info.os,
      username: register_info.username,
      version: register_info.version,
      last_seen: Instant::now(),
    };

    registry
      .register_agent(device_info, tx_server.clone())
      .await;

    let ack_msg = ServerMessage {
      msg: Some(server_message::Msg::RegisterAck(RegisterAck {
        success: true,
        session_token: Uuid::new_v4().to_string(),
        error: String::new(),
      })),
    };
    let _ = tx_server.send(ack_msg);

    let dev_id = register_info.device_id.clone();
    let reg_clone = registry.clone();

    tokio::spawn(async move {
      while let Some(item) = in_stream.next().await {
        match item {
          Ok(agent_msg) => match agent_msg.msg {
            Some(agent_message::Msg::Heartbeat(hb)) => {
              reg_clone.update_heartbeat(&hb.device_id).await;
              let _ = tx_server.send(ServerMessage {
                msg: Some(server_message::Msg::HeartbeatAck(
                  desk0_protocol::HeartbeatAck {},
                )),
              });
            }
            Some(agent_message::Msg::SessionStatus(st)) => {
              println!(
                "Agent session status update: session={}, ready={}, closed={}",
                st.session_id, st.ready, st.closed
              );
            }
            _ => {}
          },
          Err(e) => {
            eprintln!("gRPC agent tunnel error ({dev_id}): {e:?}");
            break;
          }
        }
      }

      println!("gRPC agent tunnel disconnected: {dev_id}");
      reg_clone.unregister_agent(&dev_id).await;
    });

    let out_stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx_server).map(Ok);
    Ok(Response::new(Box::pin(out_stream)))
  }

  async fn relay_stream(
    &self,
    request: Request<Streaming<RelayData>>,
  ) -> Result<Response<Self::RelayStreamStream>, Status> {
    let mut in_stream = request.into_inner();
    let registry = self.registry.clone();

    let first_chunk = match in_stream.next().await {
      Some(Ok(chunk)) => chunk,
      Some(Err(e)) => return Err(Status::internal(format!("Stream error: {e:?}"))),
      None => return Err(Status::invalid_argument("Empty relay stream")),
    };

    let session_id = first_chunk.session_id.clone();
    let device_id = first_chunk.device_id.clone();
    let org_id = first_chunk.org_id.clone();

    println!("gRPC Relay stream attached for session: {session_id} on device: {device_id}");

    let (duplex_stream, tx_to_duplex, out_grpc_stream) =
      create_relay_bridge(session_id.clone(), device_id, org_id, 65536);

    if !first_chunk.data.is_empty() {
      let _ = tx_to_duplex.send(first_chunk).await;
    }

    if !registry
      .fulfill_pending_relay(&session_id, duplex_stream)
      .await
    {
      eprintln!("Failed to fulfill pending relay for session {session_id} (expired or not found)");
      return Err(Status::not_found("Pending relay not found or expired"));
    }

    tokio::spawn(async move {
      while let Some(item) = in_stream.next().await {
        match item {
          Ok(chunk) => {
            if tx_to_duplex.send(chunk).await.is_err() {
              break;
            }
          }
          Err(_) => break,
        }
      }
    });

    let mapped_out = out_grpc_stream.map(Ok);
    Ok(Response::new(Box::pin(mapped_out)))
  }
}
