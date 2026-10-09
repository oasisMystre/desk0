use std::{
  collections::HashSet,
  num::{NonZeroU16, NonZeroUsize},
  sync::Arc,
};

use ironrdp::server::{
  BitmapUpdate, DesktopSize, DisplayUpdate, PixelFormat, RdpServerDisplay, RdpServerDisplayUpdates,
  ServerError, ServerErrorExt, ServerErrorKind, ServerResult,
};
use pinray::{Rect, SourceId};
use tokio::{
  sync::{
    Mutex,
    mpsc::{Receiver, Sender, UnboundedReceiver, channel, error::TrySendError},
  },
  task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use crate::handlers::media::{MediaHandler, WrapVideoFrame};

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
          Self::full_frame_update(&pixels, width, height, frame.monitor.x, frame.monitor.y)
            .into_iter()
            .collect()
        } else {
          Self::damage_updates(
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

  fn full_frame_update(
    pixels: &[u8],
    width: usize,
    height: usize,
    x: u16,
    y: u16,
  ) -> Option<DisplayUpdate> {
    let w = u16::try_from(width).ok()?;
    let h = u16::try_from(height).ok()?;
    let stride = width.checked_mul(4)?;

    Some(DisplayUpdate::Bitmap(BitmapUpdate {
      x,
      y,
      width: NonZeroU16::new(w)?,
      height: NonZeroU16::new(h)?,
      format: PixelFormat::BgrA32,
      stride: NonZeroUsize::new(stride)?,
      data: bytes::Bytes::copy_from_slice(pixels),
    }))
  }

  fn damage_updates(
    pixels: &[u8],
    width: usize,
    height: usize,
    offset_x: u16,
    offset_y: u16,
    damage: Option<&[Rect]>,
  ) -> Vec<DisplayUpdate> {
    let Some(damage) = damage else {
      return Self::full_frame_update(pixels, width, height, offset_x, offset_y)
        .into_iter()
        .collect();
    };

    if damage.is_empty() {
      return vec![];
    }

    let stride = width * 4;
    let mut updates = Vec::with_capacity(damage.len());

    for rect in damage {
      let (Ok(x), Ok(y), Ok(w), Ok(h)) = (
        u16::try_from(rect.x.max(0)),
        u16::try_from(rect.y.max(0)),
        u16::try_from(rect.width),
        u16::try_from(rect.height),
      ) else {
        tracing::warn!(?rect, "damage rect out of RDP coordinate range skipping");
        continue;
      };

      let (x, y, w, h) = (x as usize, y as usize, w as usize, h as usize);

      if x + w > width || y + h > height {
        tracing::warn!(x, y, w, h, "damage rect out of frame bounds, skipping");
        continue;
      }

      let mut cropped = Vec::with_capacity(w * h * 4);
      for row in 0..h {
        let source_start = (y + row) * stride + x * 4;
        let source_end = source_start + w * 4;
        if source_end <= pixels.len() {
          cropped.extend_from_slice(&pixels[source_start..source_end]);
        }
      }

      let (Some(nz_w), Some(nz_h), Some(nz_stride)) = (
        NonZeroU16::new(w as u16),
        NonZeroU16::new(h as u16),
        NonZeroUsize::new(w * 4),
      ) else {
        continue;
      };

      updates.push(DisplayUpdate::Bitmap(BitmapUpdate {
        width: nz_w,
        height: nz_h,
        stride: nz_stride,
        data: cropped.into(),
        format: PixelFormat::BgrA32,
        x: offset_x.saturating_add(x as u16),
        y: offset_y.saturating_add(y as u16),
      }));
    }

    updates
  }
}
