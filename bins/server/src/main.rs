use std::net::SocketAddr;

use clap::Parser;
use desk0_protocol::Desk0HubServiceServer;
use tokio::net::TcpListener;

use crate::{auth::Auth, context::Context, grpc::Desk0Hub, registry::DeviceRegistry};

mod auth;
mod context;
mod env;
mod grpc;
mod registry;

#[derive(Parser, Debug)]
#[command(name = "desk0-server", about = "Desk0 Relay Server & Device Broker")]
struct Args {
  #[arg(long, default_value = "0.0.0.0")]
  bind_addr: String,

  /// Port for agents to connect outbound gRPC control and relay streams
  #[arg(long, default_value = "8080")]
  grpc_port: u16,

  /// Port for RDP clients / viewers to connect
  #[arg(long, default_value = "3389")]
  rdp_port: u16,

  /// Default target device ID to route incoming RDP connections to
  #[arg(long)]
  default_target_device: Option<String>,
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
  let args = Args::parse();
  let registry = DeviceRegistry::new();
  let context = Context::from_env().await?;
  let auth = Auth::from_env()?;

  let mut handles = vec![];

  let grpc_listen_addr = format!("{}:{}", args.bind_addr, args.grpc_port).parse::<SocketAddr>()?;
  let rdp_listen_addr = format!("{}:{}", args.bind_addr, args.rdp_port).parse::<SocketAddr>()?;

  let rdp_listener = TcpListener::bind(&rdp_listen_addr).await?;
  tracing::info!("Desk0 RDP Viewer Proxy running on {}", rdp_listen_addr);

  let service = Desk0Hub::new(context, registry.clone());
  {
    let service = service.clone();
    let handle = tokio::spawn(async move {
      if let Err(error) = tonic::transport::Server::builder()
        .add_service(Desk0HubServiceServer::new(service))
        .serve(grpc_listen_addr)
        .await
      {
        tracing::error!(?error, "gRPC Server error {error:?}");
      }
    });

    handles.push(handle);
  }

  {
    let handle = tokio::spawn(async move {
      loop {
        let (viewer_stream, claims) = match auth.accept_authenticated(&rdp_listener).await {
          Ok(value) => value,
          Err(error) => {
            tracing::error!(?error, "Rdp failed to accept connection");
            continue;
          }
        };

        let agent = registry.get_by_org_and_device_id(&claims.sub, &claims.device);

        match agent {
          Some(agent) => {
            let service = service.clone();
            tokio::spawn(async move {
              if let Err(error) = service
                .bridge_client(viewer_stream, agent.info.device_id)
                .await
              {
                tracing::error!(
                  org = claims.sub,
                  device = claims.device,
                  ?error,
                  "RDP proxy session error"
                );
              }
            });
          }
          None => {
            tracing::error!(
              org = claims.sub,
              device = claims.device,
              "No agent for target device connected"
            );
          }
        }
      }
    });

    handles.push(handle);
  }

  Ok(())
}
