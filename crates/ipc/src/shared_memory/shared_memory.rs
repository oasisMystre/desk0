use std::{
  fmt::Debug,
  marker::PhantomData,
  sync::{Arc, atomic::AtomicU64},
  time::Duration,
};

use shared_memory::{Shmem, ShmemConf, ShmemError};
use tokio::{
  io::AsyncRead,
  sync::{
    Mutex,
    mpsc::{Receiver, Sender, channel},
  },
  task::JoinSet,
};

use super::{config::IpcConfig, pool::Pool, reservation::Reservation, role::Role};
use crate::{RkyvDecode, pdu::IpcFrame};
use crate::{
  RkyvEncode,
  pdu::{IpcPackMessage, IpcSharedBuffer, IpcSharedBufferAck},
};

struct Mapping(Shmem);

unsafe impl Send for Mapping {}
unsafe impl Sync for Mapping {}

pub(crate) struct IpcMessage {
  pub(super) bytes: Vec<u8>,
  pub(super) reservation: Option<Reservation>,
}

pub struct IpcSharedMemory<T, U> {
  memory: Mapping,
  pub(super) role: Role,
  pub(super) pool: Arc<Pool>,
  pub(super) sequence: AtomicU64,
  pub(super) config: IpcConfig,

  pub(super) sender: Sender<IpcMessage>,
  pub(crate) receiver: Arc<Mutex<Option<Receiver<IpcMessage>>>>,

  _marker: PhantomData<(T, U)>,
}

impl<T, U> Debug for IpcSharedMemory<T, U> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Ipc")
      .field("role", &self.role)
      .field("sequence", &self.sequence)
      .field("config", &self.config)
      .finish()
  }
}

impl<T, U> IpcSharedMemory<T, U>
where
  T: RkyvDecode,
  T::Archived: for<'a> rkyv::bytecheck::CheckBytes<rkyv::api::high::HighValidator<'a, rkyv::rancor::Error>>
    + rkyv::Deserialize<T, rkyv::rancor::Strategy<rkyv::de::Pool, rkyv::rancor::Error>>,
  U: RkyvEncode,
{
  pub fn new(path: &str, create: bool, config: IpcConfig) -> anyhow::Result<Self> {
    assert!(
      config.max_slot >= 2 && config.max_slot.is_multiple_of(2),
      "invalid max slot must be multiple of 2"
    );

    let (shmem, role) = if create {
      std::fs::remove_file(path).ok();

      let shmem = match ShmemConf::new()
        .size(config.memory_size())
        .flink(path)
        .create()
      {
        Ok(it) => it,
        Err(ShmemError::LinkExists) => ShmemConf::new()
          .size(config.memory_size())
          .flink(path)
          .open()?,
        Err(err) => return Err(err.into()),
      };

      (shmem, Role::Host)
    } else {
      (
        ShmemConf::new()
          .size(config.memory_size())
          .flink(path)
          .open()?,
        Role::Guest,
      )
    };

    anyhow::ensure!(
      shmem.len() >= config.memory_size(),
      "shared memory {path:?} is {} bytes, need at least {}",
      shmem.len(),
      config.memory_size()
    );

    let (sender, receiver) = channel::<IpcMessage>(100);

    Ok(Self {
      pool: Arc::new(Pool::new(
        role.send_slots(config.slots_per_direction() as u32),
      )),
      role,
      config,
      memory: Mapping(shmem),
      sequence: AtomicU64::new(0),
      sender,
      receiver: Arc::new(Mutex::new(Some(receiver))),
      _marker: PhantomData,
    })
  }

  pub fn role(&self) -> &Role {
    &self.role
  }

  pub fn free_slots(&self) -> usize {
    self.pool.lock().free.len()
  }

  pub(super) fn slot_ptr(&self, slot: u32) -> *mut u8 {
    debug_assert!((slot as usize) < self.config.max_slot);

    unsafe {
      self
        .memory
        .0
        .as_ptr()
        .add(slot as usize * self.config.slot_size)
    }
  }

  pub fn producer<R>(
    &self,
    mut reader: R,
    sender: Sender<IpcPackMessage<T>>,
    tasks: &mut JoinSet<()>,
  ) where
    R: AsyncRead + Unpin + Send + 'static,
  {
    let timeout = self.config.used_slot_timeout;

    let mut stop_reclaimer = None;
    if let Some(timeout) = timeout {
      let tick = (timeout / 4).clamp(Duration::from_millis(10), Duration::from_secs(1));

      let reclaimer = tasks.spawn({
        let pool = Arc::clone(&self.pool);

        async move {
          let mut interval = tokio::time::interval(tick);
          loop {
            interval.tick().await;
            let reclaimed = pool.reclaim_expired(timeout);
            if reclaimed > 0 {
              tracing::warn!(reclaimed, "reclaimed unacked ipc slots after timeout");
            }
          }
        }
      });

      stop_reclaimer = Some(reclaimer);
    }

    tasks.spawn({
      let pool = Arc::clone(&self.pool);
      let max_message_size = self.config.max_message_size;
      let mut buffer = Vec::with_capacity(max_message_size);

      async move {
        loop {
          let message = IpcFrame::<T>::decode(&mut reader, &mut buffer, max_message_size).await;
          match message {
            Ok(IpcFrame::Ack(acks)) => {
              for ack in acks {
                if !pool.ack(ack.slot, ack.sequence) {
                  tracing::debug!(
                    slot = ack.slot,
                    sequence = ack.sequence,
                    "ignoring ack that matches not inflight slot"
                  );
                }
              }
            }
            Ok(IpcFrame::Message(message)) => {
              if sender.send(message).await.is_err() {
                tracing::info!("sender broken");

                break;
              }
            }
            Err(error) => {
              let closed = error
                .downcast_ref::<std::io::Error>()
                .is_some_and(|error| error.kind() == std::io::ErrorKind::UnexpectedEof);

              if closed {
                tracing::debug!("ipc peer closed the stream");
              } else {
                tracing::warn!(?error, "decoding ipc packed message error");
              }

              break;
            }
          };
        }

        if let Some(stop_reclaimer) = stop_reclaimer {
          stop_reclaimer.abort();
        }
      }
    });
  }

  pub fn read_from_shared_buffer(
    &self,
    shared_buffer: &IpcSharedBuffer,
  ) -> anyhow::Result<(&[u8], IpcSharedBufferAck)> {
    let peer_slots = self
      .role
      .peer()
      .send_slots(self.config.slots_per_direction() as u32);
    anyhow::ensure!(
      peer_slots.contains(&shared_buffer.slot),
      "slot {} is not one of the peer's slots",
      shared_buffer.slot
    );

    let len = shared_buffer.len as usize;
    let offset = shared_buffer.offset as usize;
    anyhow::ensure!(offset <= self.config.slot_size, "invalid offset");
    anyhow::ensure!(
      len <= self.config.slot_size - offset,
      "buffer range exceeds slot"
    );

    let buffer =
      unsafe { std::slice::from_raw_parts(self.slot_ptr(shared_buffer.slot).add(offset), len) };

    let ack = IpcSharedBufferAck {
      slot: shared_buffer.slot,
      sequence: shared_buffer.sequence,
    };

    Ok((buffer, ack))
  }

  pub fn release_slots_after_peer_stopped(&self) -> usize {
    self.pool.release_all_after_peer_stopped()
  }
}
