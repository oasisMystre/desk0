use std::sync::{Arc, atomic::Ordering};

use super::{
  pool::{SlotState, UsedSlot},
  reservation::Reservation,
};
use crate::{
  IpcFrame, IpcPackMessage, IpcSharedBuffer, IpcSharedBufferAck, IpcSharedMemory, RkyvDecode,
  RkyvEncode, shared_memory::shared_memory::IpcMessage,
};

impl<T, U> IpcSharedMemory<T, U>
where
  T: RkyvDecode,
  T::Archived: for<'a> rkyv::bytecheck::CheckBytes<rkyv::api::high::HighValidator<'a, rkyv::rancor::Error>>
    + rkyv::Deserialize<T, rkyv::rancor::Strategy<rkyv::de::Pool, rkyv::rancor::Error>>,
  U: RkyvEncode,
{
  pub fn blocking_ack_shared_buffers(&self, acks: Vec<IpcSharedBufferAck>) -> anyhow::Result<()> {
    let ack: IpcFrame<u8> = IpcFrame::Ack(acks);
    let bytes = ack.encode()?;

    self.sender.blocking_send(IpcMessage {
      bytes,
      reservation: None,
    })?;

    Ok(())
  }

  pub fn blocking_copy_from_shared_buffers(
    &self,
    shared_buffers: &[IpcSharedBuffer],
  ) -> anyhow::Result<Vec<Vec<u8>>> {
    let mut acks = Vec::with_capacity(shared_buffers.len());
    let mut slices = Vec::with_capacity(shared_buffers.len());

    for shared_buffer in shared_buffers {
      let (slice, ack) = self.read_from_shared_buffer(shared_buffer)?;
      acks.push(ack);
      slices.push(slice);
    }

    let owned: Vec<Vec<u8>> = slices.into_iter().map(<[u8]>::to_vec).collect();
    self.blocking_ack_shared_buffers(acks)?;

    Ok(owned)
  }

  pub fn blocking_copy_from_shared_buffer(
    &self,
    shared_buffer: IpcSharedBuffer,
  ) -> anyhow::Result<Vec<u8>> {
    let (slice, ack) = self.read_from_shared_buffer(&shared_buffer)?;
    let owned: Vec<u8> = slice.to_vec();
    self.blocking_ack_shared_buffers(vec![ack])?;

    Ok(owned)
  }

  fn blocking_reserve(&self, count: usize) -> anyhow::Result<Reservation> {
    let slot_per_direction = self.config.slots_per_direction();
    anyhow::ensure!(
      count <= slot_per_direction,
      "{count} shared buffers requested, at most {slot_per_direction} fit in flight",
    );

    if count == 0 {
      return Ok(Reservation {
        armed: true,
        items: vec![],
        pool: Arc::clone(&self.pool),
      });
    }

    let countu32 = u32::try_from(count)?;

    tokio::task::block_in_place(|| {
      tokio::runtime::Handle::current().block_on(async {
        self.pool.permits.acquire_many(countu32).await?.forget();

        Ok::<_, anyhow::Error>(())
      })
    })?;

    let mut reservation = Reservation {
      armed: true,
      pool: Arc::clone(&self.pool),
      items: Vec::with_capacity(count),
    };

    {
      let mut state = self.pool.lock();

      if state.free.len() < count {
        self.pool.permits.add_permits(count);
        anyhow::bail!("slot pool out of sync with its semaphore");
      }

      for _ in 0..count {
        let slot = state
          .free
          .pop()
          .expect("slot pool out of sync with its semaphore");
        // todo: unsafe
        let sequence = self.sequence.fetch_add(1, Ordering::Relaxed);
        state.used.insert(
          slot,
          UsedSlot {
            sequence,
            state: SlotState::Reserved,
          },
        );
        reservation.items.push((slot, sequence));
      }
    }

    Ok(reservation)
  }

  pub fn blocking_send(&self, data: U) -> anyhow::Result<()> {
    let frame = IpcFrame::Message(crate::IpcPackMessage {
      data,
      buffers: None,
    });

    let bytes = frame.encode()?;
    anyhow::ensure!(
      bytes.len().saturating_sub(4) <= self.config.max_message_size,
      "encoded message exceeds maximum message size"
    );

    self.sender.blocking_send(IpcMessage {
      bytes,
      reservation: None,
    })?;

    Ok(())
  }

  pub fn blocking_send_with_shared_buffers<B>(
    &self,
    data: U,
    shared_buffers: &[B],
  ) -> anyhow::Result<()>
  where
    B: AsRef<[u8]> + Sync,
  {
    let max_slot_per_direction = self.config.slots_per_direction();
    anyhow::ensure!(
      shared_buffers.len() <= max_slot_per_direction,
      "{} shared buffers requested, at most {max_slot_per_direction} fit in flight",
      shared_buffers.len(),
    );

    for (index, buffer) in shared_buffers.iter().enumerate() {
      anyhow::ensure!(
        buffer.as_ref().len() <= self.config.slot_size,
        "shared buffer {index} exceeds slot size"
      );
    }

    let mut reservation = self.blocking_reserve(shared_buffers.len())?;

    let half = u32::try_from(max_slot_per_direction)?;
    let owned_slots = self.role.send_slots(half);

    let mut descriptors = Vec::with_capacity(shared_buffers.len());

    for (buffer, &(slot, sequence)) in shared_buffers.iter().zip(&reservation.items) {
      anyhow::ensure!(
        owned_slots.contains(&slot),
        "reserved slot {slot} is outside this side's range"
      );

      let buffer = buffer.as_ref();

      unsafe {
        std::ptr::copy_nonoverlapping(buffer.as_ptr(), self.slot_ptr(slot), buffer.len());
      }

      descriptors.push(IpcSharedBuffer {
        slot,
        sequence,
        offset: 0,
        len: u32::try_from(buffer.len())?,
      });
    }

    let message = IpcPackMessage {
      data,
      buffers: (!descriptors.is_empty()).then_some(descriptors),
    };

    let frame = IpcFrame::Message(message);
    let bytes = frame.encode()?;
    drop(frame);

    let payload_len = bytes
      .len()
      .checked_sub(4)
      .ok_or_else(|| anyhow::anyhow!("encoded IPC frame is missing its length prefix"))?;
    anyhow::ensure!(
      payload_len <= self.config.max_message_size,
      "encoded message is {payload_len} bytes, max is {}",
      self.config.max_message_size
    );

    reservation.queue();
    self.sender.blocking_send(IpcMessage {
      bytes,
      reservation: Some(reservation),
    })?;

    Ok(())
  }
}
