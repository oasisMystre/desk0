use std::{
  collections::HashSet,
  sync::{Arc, Mutex},
};

use ipc::decode;
use ipc_pdu::{
  Listener,
  outbound::{IpcDamageRect, IpcDisplayInfo, IpcUnpackedDisplayEvent},
};
use ironrdp::server::{
  DesktopSize, DisplayUpdate, RdpServerDisplay, RdpServerDisplayUpdates, ServerError,
  ServerErrorExt, ServerErrorKind, ServerResult,
};
use pinray::Rect;
use tokio::{
  sync::{
    mpsc::{Receiver, Sender, UnboundedReceiver, channel, error::TrySendError},
    watch,
  },
  task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use crate::media::{damage_updates, full_frame_update};

#[derive(Clone)]
pub struct DisplayHandler {
  updates_sender: Sender<DisplayUpdate>,
  updates_receiver: Arc<Mutex<Option<Receiver<DisplayUpdate>>>>,

  info: watch::Receiver<Option<IpcDisplayInfo>>,
  info_sender: watch::Sender<Option<IpcDisplayInfo>>,

  ipc: Arc<Listener>,
  inbound: Arc<Mutex<Option<UnboundedReceiver<IpcUnpackedDisplayEvent>>>>,

  handle: Arc<Mutex<Option<JoinHandle<()>>>>,
  cancellation_token: CancellationToken,
}

struct DisplayUpdates {
  receiver: Receiver<DisplayUpdate>,
}

impl DisplayHandler {
  pub fn new(
    ipc: Arc<Listener>,
    inbound: UnboundedReceiver<IpcUnpackedDisplayEvent>,
    cancellation_token: CancellationToken,
  ) -> Self {
    let (info_sender, info) = watch::channel(None);
    let (updates_sender, updates_receiver) = channel(8);

    Self {
      ipc,
      info,
      info_sender,
      updates_sender,
      cancellation_token,
      handle: Arc::new(Mutex::new(None)),
      inbound: Arc::new(Mutex::new(Some(inbound))),
      updates_receiver: Arc::new(Mutex::new(Some(updates_receiver))),
    }
  }

  pub fn start(&self) {
    let ipc = self.ipc.clone();
    let info_sender = self.info_sender.clone();
    let updates_sender = self.updates_sender.clone();
    let cancellation_token = self.cancellation_token.clone();

    let Some(mut inbound) = self.inbound.clone().lock().unwrap().take() else {
      return;
    };

    let handle = tokio::task::spawn(async move {
      let mut needs_full_frame: HashSet<String> = HashSet::new();

      'capture: loop {
        let event = tokio::select! {
            _ = cancellation_token.cancelled() => break,
            event = inbound.recv() => match event {
                Some(event) => event,
                None => break,
            },
            _ = updates_sender.closed() => break,
        };

        match event {
          IpcUnpackedDisplayEvent::DisplayInfo(size) => {
            info_sender.send_replace(Some(size));
          }
          IpcUnpackedDisplayEvent::BitmapUpdate(update) => {
            let source = update.source.clone();
            let result = {
              let ipc = Arc::clone(&ipc);

              async move {
                let pixels = ipc.copy_from_shared_buffer(&update.pixels).await?;
                let damages: Option<Vec<Rect>> = match update.damage {
                  Some(damage) => {
                    let bytes = ipc.copy_from_shared_buffer(&damage).await?;
                    Some(
                      decode::<Vec<IpcDamageRect>>(&bytes)?
                        .into_iter()
                        .map(|rect| rect.into())
                        .collect(),
                    )
                  }
                  None => None,
                };

                Ok::<_, anyhow::Error>((pixels, damages))
              }
              .await
            };

            match result {
              Ok((pixels, damage)) => {
                let width = update.width as usize;
                let height = update.height as usize;

                let full_frame =
                  update.first_frame || needs_full_frame.contains(&source) || damage.is_none();

                let updates = if full_frame {
                  full_frame_update(&pixels, width, height, update.x, update.y)
                    .into_iter()
                    .collect()
                } else {
                  damage_updates(
                    &pixels,
                    width,
                    height,
                    update.x,
                    update.y,
                    damage.as_deref(),
                  )
                };

                let mut sent_full_frame = full_frame;

                for update in updates {
                  match updates_sender.try_send(update) {
                    Ok(()) => {}
                    Err(TrySendError::Full(_)) => {
                      sent_full_frame = true;
                      needs_full_frame.insert(source.clone());

                      tracing::trace!(
                          monitor = ?source,
                          "dropping display update because encoder is behind"
                      );
                      break;
                    }
                    Err(TrySendError::Closed(_)) => {
                      tracing::debug!("display update receiver disconnected; stopping capture");
                      break 'capture;
                    }
                  }
                }

                if sent_full_frame {
                  needs_full_frame.remove(&source);
                }
              }
              Err(error) => {
                tracing::error!(?error, "copying bitmap update from shared buffer failed")
              }
            }
          }
        }
      }
    });

    *self.handle.lock().unwrap() = Some(handle);
  }

  #[allow(dead_code)]
  pub async fn stop(&mut self) -> anyhow::Result<()> {
    self.cancellation_token.cancel();
    let handle = {
      let mut lock = self.handle.lock().unwrap();
      lock.take()
    };

    if let Some(handle) = handle {
      handle.abort();
      handle.await?;
    }

    Ok(())
  }

  async fn display_info(&self) -> IpcDisplayInfo {
    let mut info = self.info.clone();

    loop {
      if let Some(info) = *info.borrow() {
        return info;
      }

      if info.changed().await.is_err() {
        panic!("Display initialization channel closed");
      }
    }
  }
}

#[async_trait::async_trait]
impl RdpServerDisplay for DisplayHandler {
  async fn size(&mut self) -> DesktopSize {
    self.display_info().await.size.into()
  }

  async fn monitor_count(&mut self) -> u32 {
    self.display_info().await.monitors
  }

  async fn updates(&mut self) -> ServerResult<Box<dyn RdpServerDisplayUpdates>> {
    let receiver = self
      .updates_receiver
      .lock()
      .unwrap()
      .take()
      .ok_or_else(|| {
        ServerError::custom(
          "Display updates channel already taken",
          ServerErrorKind::Custom,
        )
      })?;

    self
      .ipc
      .send(ipc_pdu::inbound::IpcInboundEvent::Display(
        ipc_pdu::inbound::IpcDisplayEvent::Start(0),
      ))
      .await
      .map_err(|error| {
        ServerError::custom(
          "sending display start event to ipc",
          std::io::Error::other(error),
        )
      })?;

    Ok(Box::new(DisplayUpdates { receiver }))
  }
}

#[async_trait::async_trait]
impl RdpServerDisplayUpdates for DisplayUpdates {
  async fn next_update(&mut self) -> ServerResult<Option<DisplayUpdate>> {
    Ok(self.receiver.recv().await)
  }
}
