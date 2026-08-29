pub mod proto {
  tonic::include_proto!("desk0");
}

use std::pin::Pin;

use futures::Stream;
pub use proto::{
  desk0_hub_client::Desk0HubClient,
  desk0_hub_server::{Desk0Hub, Desk0HubServer},
  *,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream, duplex, split};

/// Utility to bridge a bidirectional gRPC `Stream<Item = Result<RelayData, tonic::Status>>`
/// with a standard `DuplexStream` (which implements `AsyncRead + AsyncWrite`).
pub fn create_relay_bridge(
  session_id: String,
  device_id: String,
  org_id: String,
  buffer_size: usize,
) -> (
  DuplexStream,
  tokio::sync::mpsc::Sender<RelayData>,
  Pin<Box<dyn Stream<Item = RelayData> + Send>>,
) {
  let (client_io, server_io) = duplex(buffer_size);
  let (mut server_read, mut server_write) = split(server_io);

  let (tx_out, rx_out) = tokio::sync::mpsc::channel::<RelayData>(128);
  let (tx_in, mut rx_in) = tokio::sync::mpsc::channel::<RelayData>(128);

  // Task 1: Pump incoming gRPC RelayData -> DuplexStream (Writing to server_write)
  tokio::spawn(async move {
    while let Some(msg) = rx_in.recv().await {
      if !msg.data.is_empty() {
        if server_write.write_all(&msg.data).await.is_err() {
        break;
        }
      }
    }
  });

  // Task 2: Read from DuplexStream -> Outgoing gRPC RelayData
  let sess_id = session_id.clone();
  let dev_id = device_id.clone();
  let o_id = org_id.clone();
  tokio::spawn(async move {
    let mut buf = vec![0u8; 16384];
    loop {
      match server_read.read(&mut buf).await {
        Ok(0) => break, // EOF
        Ok(n) => {
          let chunk = RelayData {
            session_id: sess_id.clone(),
            device_id: dev_id.clone(),
            org_id: o_id.clone(),
            data: buf[..n].to_vec(),
            is_init: false,
          };
          if tx_out.send(chunk).await.is_err() {
            break;
          }
        }
        Err(_) => break,
      }
    }
  });

  let out_stream = Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx_out));
  (client_io, tx_in, out_stream)
}
