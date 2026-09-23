use std::sync::{Arc, Mutex};

use axum::extract::ws::Message;
use dashmap::DashMap;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

#[derive(Clone, Debug)]
pub struct DeviceInfo {
  pub org_id: String,
  pub device_id: String,
}

#[derive(Clone)]
pub struct AgentSession {
  pub id: uuid::Uuid,
  pub info: DeviceInfo,
  pub sender: UnboundedSender<Message>,
  inbound: Arc<Mutex<Vec<UnboundedSender<Message>>>>,
}

impl AgentSession {
  pub fn subscribe_inbound(&self) -> UnboundedReceiver<Message> {
    let (sender, receiver) = unbounded_channel();
    self.inbound.lock().unwrap().push(sender);
    receiver
  }

  pub fn publish_inbound(&self, message: Message) {
    let mut subscribers = self.inbound.lock().unwrap();
    subscribers.retain(|subscriber| match subscriber.send(message.clone()) {
      Ok(()) => true,
      Err(error) => {
        tracing::debug!(?error, "agent data subscriber disconnected");
        false
      }
    });
  }
}

#[derive(Clone)]
pub struct DeviceRegistry {
  sessions: DashMap<String, AgentSession>,
  presence: Arc<Mutex<Vec<UnboundedSender<DeviceInfo>>>>,
}

impl DeviceRegistry {
  fn session_key(org_id: &str, device_id: &str) -> String {
    format!("{org_id}:{device_id}")
  }

  pub fn new() -> Self {
    Self {
      sessions: DashMap::new(),
      presence: Arc::new(Mutex::new(Vec::new())),
    }
  }

  pub fn register(&self, info: DeviceInfo, sender: UnboundedSender<Message>) -> AgentSession {
    let session = AgentSession {
      id: uuid::Uuid::new_v4(),
      info,
      sender,
      inbound: Arc::new(Mutex::new(Vec::new())),
    };
    self.sessions.insert(
      Self::session_key(&session.info.org_id, &session.info.device_id),
      session.clone(),
    );
    let mut subscribers = self.presence.lock().unwrap();
    subscribers.retain(|subscriber| match subscriber.send(session.info.clone()) {
      Ok(()) => true,
      Err(error) => {
        tracing::debug!(?error, "presence subscriber disconnected");
        false
      }
    });
    session
  }

  pub fn subscribe_presence(&self) -> UnboundedReceiver<DeviceInfo> {
    let (sender, receiver) = unbounded_channel();
    self.presence.lock().unwrap().push(sender);
    receiver
  }

  pub fn unregister(&self, session: &AgentSession) {
    let key = Self::session_key(&session.info.org_id, &session.info.device_id);
    if self
      .sessions
      .get(&key)
      .is_some_and(|current| current.id == session.id)
    {
      self.sessions.remove(&key);
    }
  }

  pub fn get_by_org_and_device_id(&self, org_id: &str, device_id: &str) -> Option<AgentSession> {
    self
      .sessions
      .get(&Self::session_key(org_id, device_id))
      .map(|value| value.clone())
  }
}
