use std::{
  num::{NonZeroU16, NonZeroUsize},
  sync::Arc,
};

use ironrdp::server::{
  BitmapUpdate, DesktopSize, DisplayUpdate, PixelFormat, RdpServerDisplay, RdpServerDisplayUpdates,
  ServerError, ServerErrorExt, ServerErrorKind, ServerResult,
};
use pinray::Rect;
use tokio::sync::{
  Mutex,
  mpsc::{Receiver, Sender, UnboundedReceiver, channel, error::TrySendError},
};
use tokio_util::sync::CancellationToken;

use crate::handlers::media::{MediaHandler, WrapVideoFrame};

#[derive(Clone)]
pub struct DisplayHandler {
  media: Arc<MediaHandler>,
  display_updates_sender: Arc<Sender<DisplayUpdate>>,
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
      display_updates_sender: Arc::new(display_updates_sender),
      display_updates_receiver: Arc::new(Mutex::new(Some(display_updates_receiver))),
    }
  }
}

#[async_trait::async_trait]
impl RdpServerDisplay for DisplayHandler {
  async fn size(&mut self) -> DesktopSize {
    let monitors = self.media.monitors();

    monitors
      .iter()
      .find(|monitor| monitor.primary)
      .or_else(|| monitors.first())
      .map(|monitor| DesktopSize {
        width: monitor.width,
        height: monitor.height,
      })
      .expect("primary display")
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
  pub async fn capture(
    &self,
    mut video_frame_receiver: UnboundedReceiver<WrapVideoFrame>,
    cancellation_token: CancellationToken,
  ) -> anyhow::Result<()> {
    let display_updates_sender = self.display_updates_sender.clone();
    tokio::spawn(async move {
      loop {
        if cancellation_token.is_cancelled() {
          break;
        }

        let frame = match video_frame_receiver.recv().await {
          Some(frame) => frame,
          None => break,
        };

        let Some(pixels) = frame.inner.to_tight_bytes() else {
          tracing::trace!("dropping frame with unsupported pixel data");
          continue;
        };

        let width = frame.inner.width as usize;
        let height = frame.inner.height as usize;

        // let updates = if frame.first_frame {
        //   Self::full_frame_update(
        //     &pixels,
        //     width,
        //     height,
        //     frame.monitor.left,
        //     frame.monitor.top,
        //   )
        //   .into_iter()
        //   .collect()
        // } else {
        //   Self::damage_updates(
        //     &pixels,
        //     width,
        //     height,
        //     frame.monitor.left,
        //     frame.monitor.top,
        //     frame.inner.damage.as_deref(),
        //   )
        // };
        //
        let updates: Vec<DisplayUpdate> = Self::full_frame_update(
          &pixels,
          width,
          height,
          frame.monitor.left,
          frame.monitor.top,
        )
        .into_iter()
        .collect();

        for update in updates {
          match display_updates_sender.try_send(update) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
              tracing::trace!("dropping display update because encoder is behind");
            }
            Err(TrySendError::Closed(_)) => {
              tracing::debug!("display update receiver disconnected; stopping capture");
              break;
            }
          }
        }
      }
    });

    Ok(())
  }

  fn full_frame_update(
    pixels: &[u8],
    width: usize,
    height: usize,
    offset_x: i32,
    offset_y: i32,
  ) -> Option<DisplayUpdate> {
    let x = u16::try_from(offset_x).ok()?;
    let y = u16::try_from(offset_y).ok()?;
    let w = u16::try_from(width).ok()?;
    let h = u16::try_from(height).ok()?;

    Some(DisplayUpdate::Bitmap(BitmapUpdate {
      x,
      y,
      width: NonZeroU16::new(w)?,
      height: NonZeroU16::new(h)?,
      format: PixelFormat::BgrA32,
      data: bytes::Bytes::copy_from_slice(pixels),
      stride: NonZeroUsize::new(width.checked_mul(4)?)?,
    }))
  }

  fn damage_updates(
    pixels: &[u8],
    width: usize,
    height: usize,
    offset_x: i32,
    offset_y: i32,
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
      let (Ok(x), Ok(y), Ok(ox), Ok(oy), Ok(w), Ok(h)) = (
        u16::try_from(rect.x.max(0)),
        u16::try_from(rect.y.max(0)),
        u16::try_from(offset_x.max(0)),
        u16::try_from(offset_y.max(0)),
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
        x: ox.saturating_add(x as u16),
        y: oy.saturating_add(y as u16),
      }));
    }

    updates
  }
}
