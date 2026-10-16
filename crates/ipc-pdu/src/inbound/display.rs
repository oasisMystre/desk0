use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Archive, Serialize, Deserialize)]
pub enum IpcDisplayEvent {
  Stop,
}
