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

#[derive(Archive, Serialize, Deserialize)]
pub struct IpcSharedBufferAck {
  pub slot: u32,
  pub sequence: u64,
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct IpcSharedBufferField(pub u8);

#[derive(Default, Archive, Serialize, Deserialize)]
pub struct IpcPackMessage<T> {
  pub data: T,
  pub buffers: Option<Vec<IpcSharedBuffer>>,
}

#[derive(Default, Archive, Serialize, Deserialize)]
pub struct IpcRawPackMessage<T> {
  pub data: T,
  pub buffers: Option<Vec<Vec<u8>>>,
}

#[derive(Archive, Serialize, Deserialize)]
pub enum IpcFrame<T> {
  Ack(Vec<IpcSharedBufferAck>),
  Message(IpcPackMessage<T>),
}

pub trait RkyvEncode
where
  Self: Archive + Send + Sync + Sized + 'static,
  Self: for<'a> Serialize<HighSerializer<AlignedVec, ArenaHandle<'a>, rkyv::rancor::Error>>,
{
}
impl<T> RkyvEncode for T
where
  T: Archive + Send + Sync + Sized + 'static,
  T: for<'a> Serialize<HighSerializer<AlignedVec, ArenaHandle<'a>, rkyv::rancor::Error>>,
{
}

pub trait RkyvDecode
where
  Self: Archive + Send + Sync + Sized + 'static,
  Self::Archived: for<'a> CheckBytes<HighValidator<'a, rkyv::rancor::Error>>
    + Deserialize<Self, Strategy<rkyv::de::Pool, rkyv::rancor::Error>>,
{
}

impl<T> RkyvDecode for T
where
  T: Archive + Send + Sync + Sized + 'static,
  T::Archived: for<'a> CheckBytes<HighValidator<'a, rkyv::rancor::Error>>
    + Deserialize<T, Strategy<rkyv::de::Pool, rkyv::rancor::Error>>,
{
}

impl<T> IpcFrame<T>
where
  T: RkyvEncode,
{
  pub fn encode(&self) -> anyhow::Result<Vec<u8>> {
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(self)?;
    let len = u32::try_from(bytes.len())?;

    let mut output = Vec::with_capacity(4 + bytes.len());
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
    read: &mut R,
    buffer: &mut Vec<u8>,
    max: usize,
  ) -> anyhow::Result<Self> {
    let mut header = [0u8; 4];
    read.read_exact(&mut header).await?;
    let len = u32::from_le_bytes(header) as usize;

    anyhow::ensure!(
      len > 0 && len <= max,
      "invalid IPC message length={len}, max={max}"
    );

    buffer.resize(len, 0);
    read.read_exact(buffer.as_mut_slice()).await?;

    Ok(rkyv::from_bytes::<Self, rkyv::rancor::Error>(
      buffer.as_slice(),
    )?)
  }
}
