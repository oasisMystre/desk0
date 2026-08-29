use std::{collections::HashMap, sync::Arc, time::Instant};

use desk0_protocol::ServerMessage;
use tokio::{
  io::DuplexStream,
  sync::{RwLock, mpsc::UnboundedSender, oneshot},
};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct DeviceInfo {
  pub org_id: String,
  pub device_id: String,
  pub hostname: Option<String>,
  pub os: String,
  pub username: Option<String>,
  pub version: String,
  pub last_seen: Instant,
}

pub struct AgentSession {
  pub info: DeviceInfo,
  pub sender: UnboundedSender<ServerMessage>,
}

#[derive(Clone, Default)]
pub struct DeviceRegistry {
  agents: Arc<RwLock<HashMap<String, AgentSession>>>,
  pending_relays: Arc<RwLock<HashMap<String, oneshot::Sender<DuplexStream>>>>,
}

impl DeviceRegistry {
  pub fn new() -> Self {
    Self::default()
  }

  pub async fn register_agent(&self, info: DeviceInfo, sender: UnboundedSender<ServerMessage>) {
    let mut lock = self.agents.write().await;
    lock.insert(info.device_id.clone(), AgentSession { info, sender });
  }

  pub async fn update_heartbeat(&self, device_id: &str) {
    let mut lock = self.agents.write().await;
    if let Some(agent) = lock.get_mut(device_id) {
      agent.info.last_seen = Instant::now();
    }
  }

  pub async fn unregister_agent(&self, device_id: &str) {
    let mut lock = self.agents.write().await;
    lock.remove(device_id);
  }

  pub async fn get_agent_sender(&self, device_id: &str) -> Option<UnboundedSender<ServerMessage>> {
    let lock = self.agents.read().await;
    lock.get(device_id).map(|s| s.sender.clone())
  }

  pub async fn list_devices_for_org(&self, org_id: &str) -> Vec<DeviceInfo> {
    let lock = self.agents.read().await;
    lock
      .values()
      .filter(|a| org_id.is_empty() || a.info.org_id == org_id)
      .map(|a| a.info.clone())
      .collect()
  }

  pub async fn register_pending_relay(
    &self,
    session_id: String,
    sender: oneshot::Sender<DuplexStream>,
  ) {
    let mut lock = self.pending_relays.write().await;
    lock.insert(session_id, sender);
  }

  pub async fn fulfill_pending_relay(&self, session_id: &str, stream: DuplexStream) -> bool {
    let mut lock = self.pending_relays.write().await;
    if let Some(sender) = lock.remove(session_id) {
      sender.send(stream).is_ok()
    } else {
      false
    }
  }
}
