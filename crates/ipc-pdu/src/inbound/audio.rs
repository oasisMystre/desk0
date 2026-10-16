use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Archive, Serialize, Deserialize)]
pub enum IpcAudioEvent {
  Stop,
  Start(usize),
}
