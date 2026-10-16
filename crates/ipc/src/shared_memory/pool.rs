use std::{
  collections::HashMap,
  ops::Range,
  sync::{Arc, PoisonError},
  time::{Duration, Instant},
};

use tokio::sync::Semaphore;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SlotState {
  Reserved,
  Queued,
  Quarantined,
  Sent(Instant),
}

pub(super) struct UsedSlot {
  pub sequence: u64,
  pub state: SlotState,
}

pub(super) struct PoolState {
  pub free: Vec<u32>,
  pub used: HashMap<u32, UsedSlot>,
}

pub(super) struct Pool {
  pub permits: Arc<Semaphore>,
  pub state: std::sync::Mutex<PoolState>,
}

impl Pool {
  pub(super) fn new(slots: Range<u32>) -> Self {
    let free: Vec<u32> = slots.collect();

    Self {
      permits: Arc::new(Semaphore::new(free.len())),
      state: std::sync::Mutex::new(PoolState {
        free,
        used: HashMap::new(),
      }),
    }
  }

  pub(super) fn lock(&self) -> std::sync::MutexGuard<'_, PoolState> {
    self.state.lock().unwrap_or_else(PoisonError::into_inner)
  }

  fn free_where(&self, mut predicate: impl FnMut(&UsedSlot) -> bool) -> usize {
    let freed = {
      let mut state = self.lock();
      let PoolState { free, used } = &mut *state;
      let mut freed = 0;

      used.retain(|&slot, entry| {
        if predicate(entry) {
          free.push(slot);
          freed += 1;
          false
        } else {
          true
        }
      });

      freed
    };

    if freed > 0 {
      self.permits.add_permits(freed);
    }
    freed
  }

  pub(super) fn ack(&self, slot: u32, sequence: u64) -> bool {
    let released = {
      let mut state = self.lock();
      match state.used.get(&slot) {
        Some(used) if used.sequence == sequence && used.state != SlotState::Reserved => {
          state.used.remove(&slot);
          state.free.push(slot);
          true
        }
        _ => false,
      }
    };

    if released {
      self.permits.add_permits(1);
    }

    released
  }

  pub(super) fn release_reserved(&self, slot: u32, sequence: u64) -> bool {
    let released = {
      let mut state = self.lock();
      match state.used.get(&slot) {
        Some(used)
          if used.sequence == sequence
            && matches!(used.state, SlotState::Reserved | SlotState::Queued) =>
        {
          state.used.remove(&slot);
          state.free.push(slot);
          true
        }
        _ => false,
      }
    };

    if released {
      self.permits.add_permits(1);
    }

    released
  }

  pub(super) fn reclaim_expired(&self, timeout: Duration) -> usize {
    let now = Instant::now();
    self.free_where(|used| match used.state {
      SlotState::Sent(at) => now.duration_since(at) >= timeout,
      _ => false,
    })
  }

  pub(super) fn release_all_after_peer_stopped(&self) -> usize {
    self.free_where(|used| used.state != SlotState::Reserved)
  }
}
