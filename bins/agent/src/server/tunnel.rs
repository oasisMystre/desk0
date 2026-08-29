use std::time::Duration;

use dashmap::DashMap;
use desk0_protocol::{
  AgentMessage, AgentRegister, Desk0HubClient, Heartbeat, RelayData, agent_message,
  create_relay_bridge, server_message,
};
use futures::StreamExt;
use tokio::{
  sync::mpsc::unbounded_channel,
  time::{interval, sleep},
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_util::sync::CancellationToken;
use tonic::Request;

use crate::server::{
  identity::Identity,
  server::{AgentServer, AgentServerConfig},
  traits::Tunnel,
};

#[derive(Clone)]
pub struct AgentTunnel {
  identity: Identity,
  config: AgentTunnelConfig,
  sessions: DashMap<String, CancellationToken>,
}

#[derive(Clone)]
pub struct AgentTunnelConfig {
  addr: String,
}

impl AgentTunnel {
  pub fn new(addr: String, identity: Identity) -> Self {
    Self {
      identity,
      sessions: DashMap::new(),
      config: AgentTunnelConfig { addr },
    }
  }
}

#[async_trait::async_trait]
impl Tunnel for AgentTunnel {
  /// Connect to grpc and create a session and inbound stream
  async fn run(&self, cancellation_token: CancellationToken) -> anyhow::Result<()> {
    while !cancellation_token.is_cancelled() {
      let endpoint = tonic::transport::Endpoint::from_shared(self.config.addr.to_string())?;
      let channel = match endpoint.connect().await {
        Ok(channel) => channel,
        Err(error) => {
          eprintln!("Failed to connect to gRPC server. Retrying in 5 seconds...:{error:?}");
          tokio::select! {
              _ = cancellation_token.cancelled() => break,
              _ = tokio::time::sleep(Duration::from_secs(5)) => continue,
          }
        }
      };

      let mut client = Desk0HubClient::new(channel);
      let (outbound_tx, outbound_rx) = unbounded_channel::<AgentMessage>();

      if let Err(error) = outbound_tx.send(AgentMessage {
        msg: Some(agent_message::Msg::Register(AgentRegister {
          org_id: self.identity.org_id.clone(),
          device_id: self.identity.device_id.clone(),
          os: self.identity.os(),
          hostname: self.identity.hostname(),
          version: self.identity.version(),
          username: self.identity.username(),
        })),
      }) {
        eprintln!("Failed to enqueue registration: {error:?}");
        continue;
      };

      let request = Request::new(UnboundedReceiverStream::new(outbound_rx));
      // send agent info and get tunnel session
      let mut inbound_stream = match client.connect_tunnel(request).await {
        Ok(response) => response.into_inner(),
        Err(status) => {
          eprintln!("gRPC connect_tunnel failed: {status:?}. Retrying in 5 seconds...");
          sleep(Duration::from_secs(5)).await;
          continue;
        }
      };

      println!("Connected to Desk0 gRPC Hub! Control tunnel active.");

      let mut heartbeat_interval = interval(Duration::from_secs(15));

      'session: loop {
        tokio::select! {
            _ = cancellation_token.cancelled() => {
                println!("Agent tunnel cancelled, Shutting down...");
                break 'session;
            }
            _ = heartbeat_interval.tick() => {
                if outbound_tx.send(
                    AgentMessage{
                        msg: Some(agent_message::Msg::Heartbeat(Heartbeat{ device_id: self.identity.device_id.clone() }))
                    }
                ).is_err() {
                    eprintln!("Failed to send heartbeat. Tunnel disconnected.");
                    break 'session;
                }
            }
            result = inbound_stream.message() => {
                match result {
                    Err(status) => {
                        eprintln!("gRPC stream error: {status:?}, Reconnecting...");
                        break 'session;
                    }
                    Ok(None) => {
                        println!("grpc Server closed stream, Reconnecting in 3 seconds...");
                        break 'session;
                    }
                    Ok(Some(message)) => {
                        match message.msg {
                            None => {},
                            Some(server_message::Msg::RegisterAck(ack)) => {
                                if ack.success {
                                    println!("Registration confirmed by gRPC server. Token: {}", ack.session_token);
                                } else {
                                    eprintln!("Registration rejected by server: {}", ack.error);
                                    break 'session;
                                }
                            },
                            Some(server_message::Msg::StartSession(start)) => {
                                let cancellation_token = CancellationToken::new();

                                self.sessions.insert(start.session_id.clone(), cancellation_token.clone());

                                let this = self.clone();
                                let session_id = start.session_id.clone();

                                std::thread::spawn(move || {
                                    let runtime = tokio::runtime::Builder::new_current_thread()
                                        .enable_all()
                                        .build();

                                    if let Ok(runtime) = runtime {
                                        runtime.block_on(async move {
                                            if let Err(error) = this.relay(session_id.clone(), cancellation_token).await {
                                                eprintln!("gRPC relay session error {}: {error:?}", session_id)
                                            }

                                            this.sessions.remove(&session_id);
                                        });
                                    }
                                });
                            }
                            Some(server_message::Msg::CloseSession(close)) => {
                                println!("Received CloseSession request for session: {}", close.session_id);
                                if let Some((_, token)) = self.sessions.remove(&close.session_id) {
                                    token.cancel();
                                }
                            }
                            Some(server_message::Msg::HeartbeatAck(_)) => {},
                        }
                    }
                }
            }
        }
      }
    }

    Ok(())
  }
}
impl AgentTunnel {
  /// Create a outbound stream after getting session
  async fn relay(
    &self,
    session_id: String,
    cancellation_token: CancellationToken,
  ) -> anyhow::Result<()> {
    println!(
      "Establishing dedicated gRPC Relay stream to {} for session {}",
      self.config.addr, session_id
    );

    let channel = tonic::transport::Endpoint::from_shared(self.config.addr.clone())?
      .connect()
      .await?;
    let mut client = Desk0HubClient::new(channel);

    // create relay bridge which intercept request from ironrdp stream
    // send to outbound stream / tunnel
    let (duplex_io, tx_in, out_stream) = create_relay_bridge(
      session_id.clone(),
      self.identity.device_id.clone(),
      self.identity.org_id.clone(),
      u16::MAX as usize + 1,
    );

    let (outbound_tx, outbound_rx) = unbounded_channel::<RelayData>();
    outbound_tx.send(RelayData {
      session_id: session_id.clone(),
      data: vec![],
      is_init: true,
      org_id: self.identity.org_id.clone(),
      device_id: self.identity.device_id.clone(),
    })?;

    tokio::spawn(async move {
      let mut out_stream = out_stream;
      while let Some(chunk) = out_stream.next().await {
        if outbound_tx.send(chunk).is_err() {
          break;
        }
      }
    });

    let request = Request::new(UnboundedReceiverStream::new(outbound_rx));
    let mut inbound_stream = client.relay_stream(request).await?.into_inner();

    tokio::spawn(async move {
      while let Ok(Some(chunk)) = inbound_stream.message().await {
        if tx_in.send(chunk).await.is_err() {
          break;
        }
      }
    });

    println!("gRPC Relay stream attached. Running RDP connection handler for session {session_id}");

    tokio::task::LocalSet::new()
      .run_until(async move {
        if let Err(error) =
          AgentServer::handle(AgentServerConfig::default(), duplex_io, cancellation_token).await
        {
          eprintln!("Agent server error: {error:?}")
        };
      })
      .await;

    println!("grpc Relay session {session_id} finished.");
    Ok(())
  }
}
