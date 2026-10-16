use std::{fmt::Debug, marker::PhantomData, sync::Arc};

use interprocess::local_socket::{
  Name,
  tokio::SendHalf,
  traits::tokio::{Listener, Stream},
};

use tokio::{
  io::AsyncWriteExt,
  sync::mpsc::{Receiver, Sender},
  task::{JoinHandle, JoinSet},
};
use tokio_util::sync::CancellationToken;

use super::shared_memory::IpcMessage;
use crate::{IpcSharedMemory, RkyvDecode};
use crate::{RkyvEncode, pdu::IpcPackMessage};

pub struct Ipc<T, U> {
  shared_memory: Arc<IpcSharedMemory<T, U>>,

  cancellation_token: CancellationToken,
  handle: Option<JoinHandle<Result<(), anyhow::Error>>>,

  _marker: PhantomData<(T, U)>,
}

impl<T, U> Debug for Ipc<T, U> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Ipc")
      .field("shared_memory", &self.shared_memory)
      .finish()
  }
}

impl<T, U> Ipc<T, U>
where
  T: RkyvDecode,
  T::Archived: for<'a> rkyv::bytecheck::CheckBytes<rkyv::api::high::HighValidator<'a, rkyv::rancor::Error>>
    + rkyv::Deserialize<T, rkyv::rancor::Strategy<rkyv::de::Pool, rkyv::rancor::Error>>,
  U: RkyvEncode,
{
  pub fn new(shared_memory: Arc<IpcSharedMemory<T, U>>) -> Self {
    Self {
      shared_memory,
      handle: None,
      cancellation_token: CancellationToken::new(),
      _marker: PhantomData,
    }
  }

  pub async fn as_listener<'a>(
    &mut self,
    path: Name<'a>,
    outbound: Sender<IpcPackMessage<T>>,
  ) -> anyhow::Result<()> {
    let shared_memory = Arc::clone(&self.shared_memory);
    let cancellation_token = self.cancellation_token.clone();
    let listener = interprocess::local_socket::ListenerOptions::new()
      .name(path)
      .create_tokio()?;

    self.handle = Some(tokio::spawn(async move {
      loop {
        let accepted = tokio::select! {
            _ = cancellation_token.cancelled() => break,
            result = listener.accept() => Some(result),
        };

        let Some(result) = accepted else { break };
        let stream = result?;

        let mut tasks = JoinSet::new();
        let cancellation_token = cancellation_token.child_token();

        let (reader, writer) = stream.split();
        shared_memory.producer(reader, outbound.clone(), &mut tasks);

        let mut receiver = shared_memory
          .receiver
          .lock()
          .await
          .take()
          .ok_or_else(|| anyhow::anyhow!("a task already has ownership of shared_memory"))?;

        tasks.spawn({
          let shared_memory = Arc::clone(&shared_memory);
          let cancellation_token = cancellation_token.child_token();

          async move {
            if let Err(error) = Self::run(writer, &mut receiver, cancellation_token).await {
              tracing::error!(?error, "run task filed")
            }

            *shared_memory.receiver.lock().await = Some(receiver);
          }
        });

        if let Some(result) = tasks.join_next().await {
          cancellation_token.cancel();
          match result {
            Ok(()) => tracing::debug!("IPC task exited"),
            Err(error) if error.is_cancelled() => {}
            Err(error) => tracing::error!(?error, "IPC task failed"),
          }

          while tasks.join_next().await.is_some() {}
        }
      }

      Ok::<_, anyhow::Error>(())
    }));

    Ok(())
  }

  pub async fn as_connector<'a>(
    &mut self,
    path: Name<'a>,
    outbound: Sender<IpcPackMessage<T>>,
  ) -> anyhow::Result<()> {
    let shared_memory = Arc::clone(&self.shared_memory);
    let cancellation_token = self.cancellation_token.clone();

    let connector = tokio::select! {
        _ = cancellation_token.cancelled() => return Ok(()),
        result = async {
          interprocess::local_socket::ConnectOptions::new()
                .name(path)
                .connect_tokio().await
        } => result?
    };

    self.handle = Some(tokio::spawn(async move {
      let (reader, writer) = connector.split();

      let mut tasks = JoinSet::new();
      shared_memory.producer(reader, outbound.clone(), &mut tasks);

      let mut receiver = shared_memory
        .receiver
        .lock()
        .await
        .take()
        .ok_or_else(|| anyhow::anyhow!("a task already has ownership of shared_memory"))?;

      let result = Self::run(writer, &mut receiver, cancellation_token.child_token()).await;

      *shared_memory.receiver.lock().await = Some(receiver);

      result?;

      Ok::<_, anyhow::Error>(())
    }));

    Ok(())
  }

  async fn run(
    mut writer: SendHalf,
    receiver: &mut Receiver<IpcMessage>,
    cancellation_token: CancellationToken,
  ) -> anyhow::Result<()> {
    loop {
      let result = tokio::select! {
          _ = cancellation_token.cancelled() => break,
          result = receiver.recv() => result
      };

      let Some(mut data) = result else {
        break;
      };

      if cancellation_token.is_cancelled() {
        break;
      }

      let result = async {
        writer.write_all(&data.bytes).await?;
        writer.flush().await?;

        Ok::<_, anyhow::Error>(())
      }
      .await;

      if let Err(error) = result {
        if let Some(reservation) = data.reservation.as_mut() {
          reservation.quarantine();
        }

        tracing::error!(?error, "IPC write failed");
        break;
      }

      if let Some(reservation) = data.reservation {
        reservation.commit();
      }
    }

    if let Err(error) = writer.shutdown().await {
      tracing::error!(?error, "IPC writer shutdown failed");
    };

    Ok(())
  }

  pub async fn stop(&mut self) -> anyhow::Result<()> {
    self.cancellation_token.cancel();

    if let Some(handle) = self.handle.take() {
      match handle.await {
        Ok(result) => result?,
        Err(error) if error.is_cancelled() => {}
        Err(error) => return Err(error.into()),
      }
    }

    Ok(())
  }
}
