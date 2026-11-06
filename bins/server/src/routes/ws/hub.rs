use std::{collections::HashMap, sync::Arc};

use axum::extract::ws::Message;
use deadpool_redis::redis::AsyncCommands;
use futures_util::StreamExt;
use tokio::{
  sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
  },
  task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use crate::context::Context;

#[repr(u8)]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum HubConnectionState {
  Idle = 0,
  Connected = 1,
  Connecting = 2,
  Disconnected = 3,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum HubMessageType {
  Text,
  Binary,
  Ping,
  Pong,
  Close,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct HubMessage {
  pub message: Vec<u8>,
  pub session: String,
  pub r#type: HubMessageType,
}
pub struct HubHandle {
  handle: Option<JoinHandle<()>>,
  cancellation_token: CancellationToken,
}

#[derive(Clone)]
pub struct Hub {
  channel: String,
  connection_states: Arc<Mutex<HashMap<String, HubConnectionState>>>,
  subscriptions: Arc<Mutex<HashMap<String, UnboundedSender<Message>>>>,

  pub context: Arc<Context>,
}

impl TryFrom<u8> for HubConnectionState {
  type Error = ();

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      0 => Ok(Self::Idle),
      1 => Ok(Self::Connected),
      2 => Ok(Self::Connecting),
      3 => Ok(Self::Disconnected),
      _ => Err(()),
    }
  }
}

impl HubMessage {
  #[allow(dead_code)]
  pub fn is_broadcast(&self) -> bool {
    self.session == "*"
  }

  pub fn into_message(self) -> Message {
    match self.r#type {
      // todo add frame
      HubMessageType::Close => Message::Close(None),
      HubMessageType::Binary => Message::binary(self.message),
      HubMessageType::Ping => Message::Ping(self.message.into()),
      HubMessageType::Pong => Message::Pong(self.message.into()),
      HubMessageType::Text => Message::text(String::from_utf8(self.message).unwrap()),
    }
  }
}

impl HubHandle {
  pub async fn run(&mut self) -> anyhow::Result<()> {
    if let Some(handle) = self.handle.take() {
      handle.await?;
    }

    Ok(())
  }

  #[allow(dead_code)]
  pub fn abort(&mut self) {
    self.cancellation_token.cancel();
  }
}

impl Hub {
  pub fn new(context: Arc<Context>, channel: &str) -> Self {
    Self {
      context,
      channel: channel.into(),
      subscriptions: Arc::new(Mutex::new(HashMap::new())),
      connection_states: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  pub async fn subscribe(&self, session: impl Into<String>) -> UnboundedReceiver<Message> {
    let (sender, receiver) = unbounded_channel();

    self
      .subscriptions
      .lock()
      .await
      .insert(session.into(), sender);

    receiver
  }

  pub async fn unsubscribe(&self, session: &str) {
    self.subscriptions.lock().await.remove(session);
  }

  pub async fn connect(&self) -> anyhow::Result<HubHandle> {
    let mut pubsub = self.context.redis.get_async_pubsub().await?;

    pubsub.subscribe(&self.channel).await?;

    let cancellation_token = CancellationToken::new();
    let handle = tokio::spawn({
      let subscriptions = self.subscriptions.clone();
      let cancellation_token = cancellation_token.clone();

      async move {
        let mut stream = pubsub.on_message();

        loop {
          tokio::select! {
              _ = cancellation_token.cancelled() => {
                  break;
              }
              result = stream.next() => {
                  match result {
                    Some(message) => {
                        match rmp_serde::from_slice::<HubMessage>(message.get_payload_bytes()){
                         Ok(message) => {
                             let sender = {
                                 let subscriptions = subscriptions.lock().await;
                                 subscriptions.get(&message.session).cloned()
                             };

                             if let Some(sender) = sender {
                              let _ =   sender.send(message.into_message());
                             }
                         },
                         Err(error) => {
                             tracing::error!(?error, "deserializing hub message");
                         },
                     }
                    },
                    None => break,
                }
              }
          }
        }
      }
    });

    Ok(HubHandle {
      cancellation_token,
      handle: Some(handle),
    })
  }

  pub async fn send(&self, session: &str, message: Message) -> anyhow::Result<()> {
    self.publish(session, message).await
  }

  #[allow(dead_code)]
  pub async fn broadcast(&self, message: Message) -> anyhow::Result<()> {
    self.publish("*", message).await
  }

  async fn publish(&self, session: &str, message: Message) -> anyhow::Result<()> {
    let mut connection = self.context.redis_pool.get().await?;
    let r#type = match message {
      Message::Text(_) => HubMessageType::Text,
      Message::Ping(_) => HubMessageType::Ping,
      Message::Pong(_) => HubMessageType::Pong,
      Message::Close(_) => HubMessageType::Close,
      Message::Binary(_) => HubMessageType::Binary,
    };

    let message = message.into_data();

    let message = HubMessage {
      r#type,
      session: session.into(),
      message: message.to_vec(),
    };

    let payload = rmp_serde::to_vec(&message)?;

    connection
      .publish::<_, _, ()>(&self.channel, payload)
      .await?;

    Ok(())
  }

  fn subscriber_key(&self, session: &str) -> String {
    format!("desk0:{}:presence:{session}", self.channel)
  }

  pub async fn register(&self, session: &str, state: HubConnectionState) -> anyhow::Result<()> {
    let mut redis = self.context.redis_pool.get().await?;
    let () = redis
      .set_ex(self.subscriber_key(session), state.clone() as u8, 10)
      .await?;

    self
      .connection_states
      .lock()
      .await
      .insert(session.into(), state);

    Ok(())
  }

  pub async fn unregister(&self, session: &str) -> anyhow::Result<()> {
    self.connection_states.lock().await.remove(session);

    let mut redis = self.context.redis_pool.get().await?;
    let () = redis.del(self.subscriber_key(session)).await?;

    Ok(())
  }

  pub async fn is_registered(&self, session: &str) -> anyhow::Result<Option<HubConnectionState>> {
    if let Some(state) = {
      let state = self.connection_states.lock().await;
      state.get(session).cloned()
    } {
      return Ok(Some(state));
    }

    let mut redis = self.context.redis_pool.get().await?;
    let result: Option<u8> = redis.get(self.subscriber_key(session)).await?;

    Ok(result.and_then(|value| HubConnectionState::try_from(value).ok()))
  }
}
