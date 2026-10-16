use std::{sync::Arc, time::Instant};

use super::pool::{Pool, SlotState};

pub(super) struct Reservation {
  pub armed: bool,
  pub pool: Arc<Pool>,
  pub items: Vec<(u32, u64)>,
}

impl Reservation {
  fn set_state(&self, state: SlotState) {
    let mut pool = self.pool.lock();
    for &(slot, sequence) in &self.items {
      if let Some(used) = pool.used.get_mut(&slot)
        && used.sequence == sequence
      {
        used.state = state;
      }
    }
  }

  pub(super) fn queue(&mut self) {
    self.set_state(SlotState::Queued);
  }

  pub(super) fn quarantine(&mut self) {
    self.set_state(SlotState::Quarantined);
    self.armed = false;
  }

  pub(super) fn commit(mut self) {
    self.set_state(SlotState::Sent(Instant::now()));
    self.armed = false;
  }
}

impl Drop for Reservation {
  fn drop(&mut self) {
    if self.armed {
      for &(slot, sequence) in &self.items {
        self.pool.release_reserved(slot, sequence);
      }
    }
  }
}
