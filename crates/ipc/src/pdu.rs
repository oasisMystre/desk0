use std::fmt::Debug;

use rkyv::{
  Archive, Deserialize, Serialize,
  api::high::{HighSerializer, HighValidator},
  bytecheck::CheckBytes,
  rancor::Strategy,
  ser::allocator::ArenaHandle,
  util::AlignedVec,
};
use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Archive, Serialize, Deserialize, Debug, Clone)]
pub struct IpcSharedBuffer {
  pub slot: u32,
  pub len: u32,
  pub offset: u32,
  pub sequence: u64,
}

#[derive(Debug, Archive, Serialize, Deserialize)]
pub struct IpcSharedBufferAck {
  pub slot: u32,
  pub sequence: u64,
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct IpcSharedBufferField(pub u8);

#[derive(Debug, Default, Archive, Serialize, Deserialize)]
pub struct IpcPackMessage<T> {
  pub data: T,
  pub buffers: Option<Vec<IpcSharedBuffer>>,
}

#[derive(Default, Archive, Serialize, Deserialize)]
pub struct IpcRawPackMessage<T> {
  pub data: T,
  pub buffers: Option<Vec<Vec<u8>>>,
}

#[derive(Debug)]
pub enum IpcFrame<T> {
  Ack(Vec<IpcSharedBufferAck>),
  Message(IpcPackMessage<T>),
}

pub trait RkyvEncode
where
  Self: Debug,
  Self: Archive + Send + Sync + Sized + 'static,
  Self: for<'a> Serialize<HighSerializer<AlignedVec, ArenaHandle<'a>, rkyv::rancor::Error>>,
{
}
impl<T> RkyvEncode for T
where
  T: Debug,
  T: Archive + Send + Sync + Sized + 'static,
  T: for<'a> Serialize<HighSerializer<AlignedVec, ArenaHandle<'a>, rkyv::rancor::Error>>,
{
}

pub trait RkyvDecode
where
  Self: Debug,
  Self: Archive + Send + Sync + Sized + 'static,
  Self::Archived: for<'a> CheckBytes<HighValidator<'a, rkyv::rancor::Error>>
    + Deserialize<Self, Strategy<rkyv::de::Pool, rkyv::rancor::Error>>,
{
}

impl<T> RkyvDecode for T
where
  T: Debug,
  T: Archive + Send + Sync + Sized + 'static,
  T::Archived: for<'a> CheckBytes<HighValidator<'a, rkyv::rancor::Error>>
    + Deserialize<T, Strategy<rkyv::de::Pool, rkyv::rancor::Error>>,
{
}

impl From<IpcSharedBuffer> for IpcSharedBufferAck {
  fn from(value: IpcSharedBuffer) -> Self {
    Self {
      slot: value.slot,
      sequence: value.sequence,
    }
  }
}

pub const IPC_FRAME_HEADER_SIZE: usize = 5;

impl<T> IpcFrame<T> {
  const ACK: u8 = 0;
  const MESSAGE: u8 = 1;
}

impl<T> IpcFrame<T>
where
  T: RkyvEncode,
{
  pub fn encode(&self) -> anyhow::Result<Vec<u8>> {
    let (discriminator, bytes) = match self {
      IpcFrame::Ack(acks) => (Self::ACK, rkyv::to_bytes::<rkyv::rancor::Error>(acks)?),
      IpcFrame::Message(message) => (
        Self::MESSAGE,
        rkyv::to_bytes::<rkyv::rancor::Error>(message)?,
      ),
    };

    let len = u32::try_from(bytes.len())?;
    let mut output = Vec::with_capacity(1 + 4 + bytes.len());

    output.extend_from_slice(&discriminator.to_le_bytes());
    output.extend_from_slice(&len.to_le_bytes());
    output.extend_from_slice(&bytes);

    Ok(output)
  }
}

impl<T> IpcFrame<T>
where
  T: RkyvDecode,
  T::Archived: for<'a> CheckBytes<HighValidator<'a, rkyv::rancor::Error>>
    + Deserialize<T, Strategy<rkyv::de::Pool, rkyv::rancor::Error>>,
{
  pub async fn decode<R: AsyncRead + Unpin>(
    reader: &mut R,
    buffer: &mut Vec<u8>,
    max: usize,
  ) -> anyhow::Result<Self> {
    let discriminator = reader.read_u8().await?;
    let len = reader.read_u32_le().await? as usize;

    anyhow::ensure!(
      len > 0 && len <= max,
      "invalid IPC message length={len}, max={max}"
    );

    buffer.resize(len, 0);
    reader.read_exact(buffer.as_mut_slice()).await?;

    let value = match discriminator {
      Self::ACK => Self::Ack(rkyv::from_bytes::<
        Vec<IpcSharedBufferAck>,
        rkyv::rancor::Error,
      >(buffer.as_slice())?),
      Self::MESSAGE => Self::Message(rkyv::from_bytes::<IpcPackMessage<T>, rkyv::rancor::Error>(
        buffer.as_slice(),
      )?),
      _ => anyhow::bail!("invalid frame discriminatore"),
    };

    Ok(value)
  }
}
