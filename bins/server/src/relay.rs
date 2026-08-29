use std::time::Duration;

use desk0_protocol::{ServerMessage, StartSession, server_message};
use tokio::net::TcpStream;
use uuid::Uuid;

use crate::registry::DeviceRegistry;

pub async fn bridge_rdp_viewer(
  mut viewer_stream: TcpStream,
  target_device_id: String,
  registry: DeviceRegistry,
) -> anyhow::Result<()> {
  let viewer_peer = viewer_stream.peer_addr()?;
  let agent_sender = match registry.get_agent_sender(&target_device_id).await {
    Some(sender) => sender,
    None => {
      anyhow::bail!("Target device {target_device_id} is offline or not found");
    }
  };

  let session_id = Uuid::new_v4().to_string();
  let (relay_tx, relay_rx) = tokio::sync::oneshot::channel::<tokio::io::DuplexStream>();

  registry
    .register_pending_relay(session_id.clone(), relay_tx)
    .await;

  println!(
    "Initiating gRPC RDP session {session_id} to device {target_device_id} for viewer {viewer_peer}..."
  );
  agent_sender.send(ServerMessage {
    msg: Some(server_message::Msg::StartSession(StartSession {
      session_id: session_id.clone(),
    })),
  })?;

  let mut duplex_stream = match tokio::time::timeout(Duration::from_secs(10), relay_rx).await {
    Ok(Ok(stream)) => stream,
    Ok(Err(_)) => anyhow::bail!("Relay channel was dropped for session {session_id}"),
    Err(_) => anyhow::bail!("Timeout waiting for agent {target_device_id} to connect relay stream"),
  };

  println!("Relay stream established for session {session_id}. Proxying RDP traffic...");
  let (from_client, from_agent) =
    tokio::io::copy_bidirectional(&mut viewer_stream, &mut duplex_stream).await?;

  println!(
    "Session {session_id} finished: {from_client} bytes from viewer, {from_agent} bytes from agent"
  );
  Ok(())
}
