use clap::Parser;

mod context;
mod env;
mod grpc;
mod init;
mod registry;
mod types;

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
pub async fn main() {}
