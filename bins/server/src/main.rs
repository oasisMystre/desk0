use clap::Parser;
use desk0_protocol::Desk0HubServer;
use tokio::net::TcpListener;

mod grpc;
mod registry;
mod relay;

use grpc::Desk0HubService;
use registry::DeviceRegistry;

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

  let grpc_listen_addr = format!("{}:{}", args.bind_addr, args.grpc_port).parse()?;
  println!("Desk0 gRPC Hub running on {}", grpc_listen_addr);

  let rdp_listen_addr = format!("{}:{}", args.bind_addr, args.rdp_port);
  let rdp_listener = TcpListener::bind(&rdp_listen_addr).await?;
  println!("Desk0 RDP Viewer Proxy running on {}", rdp_listen_addr);

  // Task 1: Start gRPC Service Server
  let service = Desk0HubService::new(registry.clone());
  tokio::spawn(async move {
    if let Err(e) = tonic::transport::Server::builder()
      .add_service(Desk0HubServer::new(service))
      .serve(grpc_listen_addr)
      .await
    {
      eprintln!("gRPC Server error: {e:?}");
    }
  });

  // Task 2: Accept RDP Viewer Connections
  let rdp_reg = registry.clone();
  let default_target = args.default_target_device;

  loop {
    let (viewer_stream, peer) = rdp_listener.accept().await?;
    println!("Accepted RDP viewer connection from {}", peer);

    let reg = rdp_reg.clone();
    let target = default_target.clone();

    tokio::spawn(async move {
      let target_id = match target {
        Some(id) => id,
        None => {
          let devices = reg.list_devices_for_org("").await;
          if let Some(dev) = devices.first() {
            dev.device_id.clone()
          } else {
            eprintln!("No target device specified and no agents currently connected");
            return;
          }
        }
      };

      if let Err(e) = relay::bridge_rdp_viewer(viewer_stream, target_id, reg).await {
        eprintln!("RDP proxy session error: {:?}", e);
      }
    });
  }
}
