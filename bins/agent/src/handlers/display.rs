use std::{collections::HashSet, sync::Arc};

use ironrdp::server::{
  DesktopSize, DisplayUpdate, RdpServerDisplay, RdpServerDisplayUpdates, ServerError,
  ServerErrorExt, ServerErrorKind, ServerResult,
};
use pinray::SourceId;
use tokio::{
  sync::{
    Mutex,
    mpsc::{Receiver, Sender, UnboundedReceiver, channel, error::TrySendError},
  },
  task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use crate::media::{MediaHandler, WrapVideoFrame, damage_updates, full_frame_update};

#[derive(Clone)]
pub struct DisplayHandler {
  media: Arc<MediaHandler>,
  display_updates_sender: Sender<DisplayUpdate>,
  display_updates_receiver: Arc<Mutex<Option<Receiver<DisplayUpdate>>>>,
}

struct DisplayUpdates {
  receiver: Receiver<DisplayUpdate>,
}

impl DisplayHandler {
  pub fn new(media: Arc<MediaHandler>) -> Self {
    let (display_updates_sender, display_updates_receiver) = channel(8);

    Self {
      media,
      display_updates_sender,
      display_updates_receiver: Arc::new(Mutex::new(Some(display_updates_receiver))),
    }
  }
}

#[async_trait::async_trait]
impl RdpServerDisplay for DisplayHandler {
  async fn size(&mut self) -> DesktopSize {
    let monitors = self.media.monitors();

    let width = monitors
      .iter()
      .map(|monitor| monitor.x as u32 + monitor.width as u32)
      .max()
      .unwrap_or(1282);

    let height = monitors
      .iter()
      .map(|monitor| monitor.y as u32 + monitor.height as u32)
      .max()
      .unwrap_or(720);

    DesktopSize {
      width: width.clamp(1, u16::MAX as u32) as u16,
      height: height.clamp(1, u16::MAX as u32) as u16,
    }
  }

  async fn updates(&mut self) -> ServerResult<Box<dyn RdpServerDisplayUpdates>> {
    let receiver = self
      .display_updates_receiver
      .lock()
      .await
      .take()
      .ok_or_else(|| {
        ServerError::custom(
          "Display updates channel already taken",
          ServerErrorKind::Custom,
        )
      })?;

    Ok(Box::new(DisplayUpdates { receiver }))
  }

  async fn monitor_count(&mut self) -> u32 {
    self.media.monitor_count() as u32
  }
}

#[async_trait::async_trait]
impl RdpServerDisplayUpdates for DisplayUpdates {
  async fn next_update(&mut self) -> ServerResult<Option<DisplayUpdate>> {
    Ok(self.receiver.recv().await)
  }
}

impl DisplayHandler {
  pub fn capture(
    &self,
    mut video_frame_receiver: UnboundedReceiver<WrapVideoFrame>,
    cancellation_token: CancellationToken,
  ) -> anyhow::Result<JoinHandle<()>> {
    let display_updates_sender = self.display_updates_sender.clone();

    let handle = tokio::spawn(async move {
      let mut needs_full_frame: HashSet<SourceId> = HashSet::new();

      'capture: loop {
        let frame = tokio::select! {
            _ = cancellation_token.cancelled() => break 'capture,
            result = video_frame_receiver.recv() => match result {
                Some(frame) => frame,
                None => break 'capture,
            },
            _ = display_updates_sender.closed() => break 'capture,
        };

        let Some(pixels) = frame.inner.to_tight_bytes() else {
          tracing::trace!("dropping frame with unsupported pixel data");
          continue;
        };

        let width = frame.inner.width as usize;
        let height = frame.inner.height as usize;

        let full_frame = frame.first_frame
          || needs_full_frame.contains(&frame.monitor.source)
          || frame.inner.damage.is_none();

        let updates = if full_frame {
          full_frame_update(&pixels, width, height, frame.monitor.x, frame.monitor.y)
            .into_iter()
            .collect()
        } else {
          damage_updates(
            &pixels,
            width,
            height,
            frame.monitor.x,
            frame.monitor.y,
            frame.inner.damage.as_deref(),
          )
        };

        let mut sent_full_frame = full_frame;

        for update in updates {
          match display_updates_sender.try_send(update) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
              sent_full_frame = true;
              needs_full_frame.insert(frame.monitor.source.clone());

              tracing::trace!(
                  monitor = ?frame.monitor.source,
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
          needs_full_frame.remove(&frame.monitor.source);
        }
      }
    });

    Ok(handle)
  }
}
