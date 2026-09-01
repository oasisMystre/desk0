use std::{collections::HashMap, sync::Arc, time::Instant};

use dashmap::DashMap;
use desk0_protocol::ConnectTunnelResponse;
use tokio::{
  io::DuplexStream,
  sync::{RwLock, mpsc::UnboundedSender, oneshot::Sender},
};

#[derive(Clone)]
pub struct DeviceInfo {
  pub os: String,
  pub org_id: String,
  pub device_id: String,
  pub version: String,
  pub last_seen: Instant,
  pub hostname: Option<String>,
  pub username: Option<String>,
}

#[derive(Clone)]
pub struct AgentSession {
  pub info: DeviceInfo,
  pub sender: UnboundedSender<ConnectTunnelResponse>,
}

#[derive(Clone)]
pub struct DeviceRegistry {
  sessions: DashMap<String, AgentSession>,
  relays: Arc<RwLock<HashMap<String, Sender<DuplexStream>>>>,
}

impl DeviceRegistry {
  pub fn new() -> Self {
    Self {
      sessions: DashMap::new(),
      relays: Arc::new(RwLock::new(HashMap::new())),
    }
  }
  pub fn register(&self, info: DeviceInfo, sender: UnboundedSender<ConnectTunnelResponse>) {
    self
      .sessions
      .insert(info.device_id.clone(), AgentSession { info, sender });
  }

  pub fn heartbeat(&self, device_id: &str) -> Option<Instant> {
    if let Some(mut agent) = self.sessions.get_mut(device_id) {
      let now = Instant::now();
      agent.info.last_seen = now;
      return Some(now);
    }

    None
  }

  pub fn unregister(&self, device_id: &str) {
    self.sessions.remove(device_id);
  }

  pub fn get(&self, device_id: &str) -> Option<AgentSession> {
    self
      .sessions
      .get(device_id)
      .map(|value| value.value().clone())
  }

  pub fn get_by_org_and_device_id(&self, org_id: &str, device_id: &str) -> Option<AgentSession> {
    return self
      .sessions
      .iter()
      .find(|session| session.info.org_id == org_id && session.info.device_id == device_id)
      .map(|value| value.clone());
  }

  pub async fn queue(&self, session_id: String, sender: Sender<DuplexStream>) {
    let mut lock = self.relays.write().await;
    lock.insert(session_id, sender);
  }

  pub async fn dequeue(&self, session_id: &str, stream: DuplexStream) -> bool {
    let mut lock = self.relays.write().await;
    if let Some(sender) = lock.remove(session_id) {
      sender.send(stream).is_ok()
    } else {
      false
    }
  }
}
