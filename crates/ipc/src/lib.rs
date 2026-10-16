mod pdu;
mod shared_memory;

pub use pdu::*;
pub use shared_memory::*;

pub use interprocess::local_socket::{
  GenericFilePath, GenericNamespaced, Name, ToFsName, ToNsName,
};

pub fn encode<T: RkyvEncode>(value: T) -> anyhow::Result<rkyv::util::AlignedVec> {
  let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&value)?;
  Ok(bytes)
}

pub fn decode<T: RkyvDecode>(bytes: &[u8]) -> anyhow::Result<T>
where
  T::Archived: for<'a> rkyv::bytecheck::CheckBytes<rkyv::api::high::HighValidator<'a, rkyv::rancor::Error>>
    + rkyv::Deserialize<T, rkyv::rancor::Strategy<rkyv::de::Pool, rkyv::rancor::Error>>,
{
  let data = rkyv::from_bytes::<T, rkyv::rancor::Error>(bytes)?;
  Ok(data)
}
