use std::ops::Range;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Role {
  Host,
  Guest,
}

impl Role {
  pub(super) fn send_slots(&self, half: u32) -> Range<u32> {
    match self {
      Role::Host => 0..half,
      Role::Guest => half..2 * half,
    }
  }

  pub(super) fn peer(&self) -> Role {
    match self {
      Role::Host => Role::Guest,
      Role::Guest => Role::Host,
    }
  }
}
