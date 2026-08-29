use std::{
  num::{NonZeroU16, NonZeroUsize},
  sync::Arc,
};

use display_info::DisplayInfo;
use ironrdp::server::{
  BitmapUpdate, DesktopSize, DisplayUpdate, PixelFormat, RdpServerDisplay, RdpServerDisplayUpdates,
};
use rs_capture::{
  CaptureBackend, FrameMetadata, PixelFormat as CapturePixelFormat, ScreenCapture,
  ScreenCaptureConfig,
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

#[derive(Clone)]
pub struct DisplayHandler {
  width: u16,
  height: u16,
  sender: UnboundedSender<DisplayUpdate>,
  receiver: Arc<tokio::sync::Mutex<Option<UnboundedReceiver<DisplayUpdate>>>>,
}

struct DisplayUpdates {
  receiver: UnboundedReceiver<DisplayUpdate>,
}

impl DisplayHandler {
  pub fn new() -> anyhow::Result<Self> {
    let displays = DisplayInfo::all()?;

    let to_u16 =
      |value: u32| -> Option<u16> { u16::try_from(value).ok().filter(|&value| value > 0) };

    let (width, height) = displays
      .iter()
      .find(|display| display.is_primary)
      .or_else(|| displays.first())
      .and_then(|display| Some((to_u16(display.width)?, to_u16(display.height)?)))
      .unwrap_or((1920, 1080));

    let (sender, receiver) = unbounded_channel();

    Ok(Self {
      width,
      height,
      sender,
      receiver: Arc::new(tokio::sync::Mutex::new(Some(receiver))),
    })
  }
}

#[async_trait::async_trait]
impl RdpServerDisplay for DisplayHandler {
  async fn size(&mut self) -> DesktopSize {
    DesktopSize {
      width: self.width,
      height: self.height,
    }
  }

  async fn updates(&mut self) -> anyhow::Result<Box<dyn RdpServerDisplayUpdates>> {
    let mut guard = self.receiver.lock().await;
    if let Some(receiver) = guard.take() {
      Ok(Box::new(DisplayUpdates { receiver }))
    } else {
      anyhow::bail!("Display updates channel already taken");
    }
  }
}

#[async_trait::async_trait]
impl RdpServerDisplayUpdates for DisplayUpdates {
  async fn next_update(&mut self) -> anyhow::Result<Option<DisplayUpdate>> {
    Ok(self.receiver.recv().await)
  }
}

impl DisplayHandler {
  pub async fn capture(&self) -> anyhow::Result<()> {
    let sender = self.sender.clone();
    let first_frame = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let first_frame_clone = first_frame.clone();

    let capture = ScreenCapture::with_callback_arc(
      ScreenCaptureConfig {
        fps: Some(60),
        backend: Some(CaptureBackend::Auto),
        pixel_format: Some(CapturePixelFormat::Bgra),
      },
      Arc::new(move |frame: rs_capture::FrameData, this: &ScreenCapture| {
        let width = frame.width as usize;
        let height = frame.height as usize;
        let is_first = first_frame_clone.swap(false, std::sync::atomic::Ordering::Relaxed);

        let updates = if is_first {
          match Self::get_full_frame_update(&frame.data, width, height) {
            Some(update) => vec![update],
            None => return,
          }
        } else {
          Self::get_frame_updates_from_metadata(&frame.data, width, height, &frame.metadata)
        };

        for update in updates {
          if sender.send(update).is_err() {
            println!("Display update receiver disconnected. Stopping capture.");
            let _ = this.stop();
            break;
          }
        }
      }),
    )?;

    capture.start().await?;

    Ok(())
  }

  fn get_full_frame_update(pixels: &[u8], width: usize, height: usize) -> Option<DisplayUpdate> {
    Some(DisplayUpdate::Bitmap(BitmapUpdate {
      x: 0,
      y: 0,
      format: PixelFormat::BgrA32,
      width: NonZeroU16::new(width as u16)?,
      height: NonZeroU16::new(height as u16)?,
      stride: NonZeroUsize::new(width * 4)?,
      data: bytes::Bytes::copy_from_slice(pixels),
    }))
  }

  fn get_frame_updates_from_metadata(
    pixels: &[u8],
    width: usize,
    height: usize,
    metadata: &FrameMetadata,
  ) -> Vec<DisplayUpdate> {
    let stride = width * 4;
    let rects = metadata
      .dirty_rects
      .iter()
      .copied()
      .chain(metadata.move_rects.iter().map(|rect| rect.destination_rect));

    let mut updates = vec![];
    for (left, top, right, bottom) in rects {
      let x = left as usize;
      let y = top as usize;
      let w = (right - left) as usize;
      let h = (bottom - top) as usize;

      if x + w > width || y + h > height {
        eprintln!("rect ({left},{top},{right},{bottom}) out of frame bounds, skipping");
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
        x: x as u16,
        y: y as u16,
        width: nz_w,
        height: nz_h,
        stride: nz_stride,
        format: PixelFormat::BgrA32,
        data: bytes::Bytes::from(cropped),
      }));
    }

    updates
  }
}
