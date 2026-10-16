use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Archive, Serialize, Deserialize)]
pub enum IpcAudioEvent {
  Start = 0,
  Stop = 1,
}
