use std::{assert_matches, io::Cursor};

use ipc_pdu::{inbound::IpcInboundEvent, outbound::IpcOutboundEvent};
use tokio::io::BufReader;

use ipc::{IpcFrame, IpcPackMessage, IpcSharedBufferAck};

#[tokio::test]
pub async fn roundtrip_encode_decode_frame() -> anyhow::Result<()> {
  let ack = IpcSharedBufferAck {
    slot: 0,
    sequence: 0,
  };

  let frame = IpcFrame::<IpcPackMessage<IpcInboundEvent>>::Ack(vec![ack]);

  let bytes = frame.encode()?;
  let size = bytes.len() - 4;
  let mut buffer = Vec::with_capacity(size);

  let mut reader = BufReader::new(Cursor::new(bytes));
  let decoded =
    IpcFrame::<IpcPackMessage<IpcOutboundEvent>>::decode(&mut reader, &mut buffer, size).await?;

  assert_matches!(
      decoded,
      IpcFrame::Ack(acks)
          if acks.first().is_some_and(|value| value.slot == 0  && value.sequence == 0)
  );

  Ok(())
}
