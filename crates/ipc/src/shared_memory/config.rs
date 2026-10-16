use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct IpcConfig {
  pub max_slot: usize,
  pub slot_size: usize,
  pub max_message_size: usize,
  pub used_slot_timeout: Option<Duration>,
}

impl IpcConfig {
  const DEFAULT_MAX_SLOT: usize = 8;
  const DEFAULT_SLOT_SIZE: usize = 10 * 1024 * 1024;
  const DEFAULT_MAX_MESSAGE_SIZE: usize = 1024 * 1024;

  pub fn slots_per_direction(&self) -> usize {
    self.max_slot / 2
  }

  pub fn memory_size(&self) -> usize {
    self.max_slot * self.slot_size
  }
}

impl Default for IpcConfig {
  fn default() -> Self {
    Self {
      used_slot_timeout: None,
      max_slot: Self::DEFAULT_MAX_SLOT,
      slot_size: Self::DEFAULT_SLOT_SIZE,
      max_message_size: Self::DEFAULT_MAX_MESSAGE_SIZE,
    }
  }
}
