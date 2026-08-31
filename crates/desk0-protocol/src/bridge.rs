use std::pin::Pin;

use futures::Stream;
use tokio::{
  io::{AsyncReadExt, AsyncWriteExt, DuplexStream, duplex, split},
  sync::mpsc::{UnboundedSender, unbounded_channel},
};

use crate::RelayData;

#[derive(Clone)]
pub struct Session {
  pub org_id: String,
  pub device_id: String,
  pub session_id: String,
}

pub struct Relay {
  pub stream: DuplexStream,
  pub inbound: UnboundedSender<RelayData>,
  pub outbound: Pin<Box<dyn Stream<Item = RelayData> + Send>>,
}

/// max bytes that can be read on wire
/// 16kib
pub const MAX_RELAY_DATA: usize = 16384;

pub fn create_relay_bridge(session: Session, max_buffer_size: usize) -> Relay {
  let (client_stream, server_stream) = duplex(max_buffer_size);
  let (mut server_read, mut server_write) = split(server_stream);

  let (outbound_tx, outbound_rx) = unbounded_channel::<RelayData>();
  let (inbound_tx, mut inbound_rx) = unbounded_channel::<RelayData>();

  tokio::spawn(async move {
    while let Some(message) = inbound_rx.recv().await {
      if !message.data.is_empty() {
        if server_write.write_all(&message.data).await.is_err() {
          break;
        }
      }
    }
  });

  tokio::spawn(async move {
    let mut buffer = vec![0u8; MAX_RELAY_DATA];

    let session = session.clone();

    loop {
      match server_read.read(&mut buffer).await {
        Ok(0) => break,
        Ok(n) => {
          if outbound_tx
            .send(RelayData {
              is_init: false,
              data: buffer[..n].to_vec(),
              org_id: session.org_id.clone(),
              device_id: session.device_id.clone(),
              session_id: session.session_id.clone(),
            })
            .is_err()
          {
            break;
          }
        }
        Err(_) => break,
      }
    }
  });

  Relay {
    stream: client_stream,
    inbound: inbound_tx,
    outbound: Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(
      outbound_rx,
    )),
  }
}
