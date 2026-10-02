use std::{
  num::{NonZeroU16, NonZeroUsize},
  sync::{Arc, atomic::Ordering},
};

use display_info::DisplayInfo;
use ironrdp::{
  pdu::pointer::PointerPositionAttribute,
  server::{
    BitmapUpdate, ColorPointer, DesktopSize, DisplayUpdate, PixelFormat, RGBAPointer,
    RdpServerDisplay, RdpServerDisplayUpdates, ServerError, ServerErrorExt, ServerErrorKind,
    ServerResult,
  },
};
use rs_capture::{
  CaptureBackend, FrameMetadata, PixelFormat as CapturePixelFormat, ScreenCapture,
  ScreenCaptureConfig, backend::PointerShape,
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio_util::sync::CancellationToken;

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
    let (sender, receiver) = unbounded_channel();
    let (width, height) = Self::get_display_size();

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

  async fn updates(&mut self) -> ServerResult<Box<dyn RdpServerDisplayUpdates>> {
    let mut guard = self.receiver.lock().await;
    if let Some(receiver) = guard.take() {
      Ok(Box::new(DisplayUpdates { receiver }))
    } else {
      return Err(ServerError::custom(
        "Display updates channel already taken",
        ServerErrorKind::Custom,
      ));
    }
  }
}

#[async_trait::async_trait]
impl RdpServerDisplayUpdates for DisplayUpdates {
  async fn next_update(&mut self) -> ServerResult<Option<DisplayUpdate>> {
    Ok(self.receiver.recv().await)
  }
}

impl DisplayHandler {
  pub async fn capture(&self, cancellation_token: CancellationToken) -> anyhow::Result<()> {
    let sender = self.sender.clone();
    let first_frame = Arc::new(std::sync::atomic::AtomicBool::new(true));

    let capture = ScreenCapture::with_callback_arc(
      ScreenCaptureConfig {
        fps: Some(60),
        backend: Some(CaptureBackend::Auto),
        pixel_format: Some(CapturePixelFormat::Bgra),
      },
      Arc::new({
        let first_frame = first_frame.clone();

        move |frame: rs_capture::FrameData, this: &ScreenCapture| {
          let width = frame.width as usize;
          let height = frame.height as usize;
          let is_first = first_frame.swap(false, Ordering::Relaxed);

          let mut updates = if is_first {
            match Self::get_full_frame_update(&frame.data, width, height) {
              Some(update) => vec![update],
              None => return,
            }
          } else {
            Self::get_frame_updates_from_metadata(&frame.data, width, height, &frame.metadata)
          };

          Self::append_pointer_updates(&mut updates, &frame.metadata);

          for update in updates {
            if let Err(error) = sender.send(update) {
              tracing::debug!(
                ?error,
                "Display update receiver disconnected; stopping capture"
              );

              if let Err(error) = this.stop() {
                tracing::error!(?error, "Failed to gracefully exit screen capture");
              }
              break;
            }
          }
        }
      }),
    )?;

    tokio::select! {
      result = capture.start() => result?,
      _ = cancellation_token.cancelled() => {
        if let Err(error) = capture.stop() {
          tracing::warn!(?error, "failed to stop screen capture during session reset");
        }
      }
    }

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

    let full_frame_fallback = metadata.dirty_rects.len() == 1
      && metadata.dirty_rects[0] == (0, 0, width as i32, height as i32);
    let pointer_only_frame = metadata.move_rects.is_empty()
      && metadata.pointer_shape.is_some()
      && (metadata.dirty_rects.is_empty() || full_frame_fallback);

    if pointer_only_frame {
      return Vec::new();
    }

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
        tracing::warn!(
          left,
          top,
          right,
          bottom,
          "rect out of frame bounds, skipping"
        );
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

  fn append_pointer_updates(updates: &mut Vec<DisplayUpdate>, metadata: &FrameMetadata) {
    if metadata.pointer_visible {
      if let Some((x, y)) = metadata.pointer_position {
        if let (Ok(x), Ok(y)) = (u16::try_from(x), u16::try_from(y)) {
          updates.push(DisplayUpdate::PointerPosition(PointerPositionAttribute {
            x,
            y,
          }));
        } else {
          tracing::warn!(x, y, "pointer position is outside RDP coordinate range");
        }
      }
    } else {
      updates.push(DisplayUpdate::HidePointer);
    }

    if let Some(pointer) = &metadata.pointer_shape {
      match pointer {
        PointerShape::Hidden => {
          updates.push(DisplayUpdate::HidePointer);
        }
        PointerShape::Default => {
          updates.push(DisplayUpdate::DefaultPointer);
        }
        PointerShape::Color(pointer) => {
          updates.push(DisplayUpdate::ColorPointer(ColorPointer {
            width: pointer.width,
            height: pointer.height,
            hot_x: pointer.hot_x,
            hot_y: pointer.hot_y,
            cache_index: pointer.cache_index,
            and_mask: pointer.and_mask.clone(),
            xor_mask: pointer.xor_mask.clone(),
          }));
        }
        PointerShape::Rgba(pointer) => {
          updates.push(DisplayUpdate::RGBAPointer(RGBAPointer {
            width: pointer.width,
            height: pointer.height,
            hot_x: pointer.hot_x,
            hot_y: pointer.hot_y,
            data: pointer.data.clone(),
            cache_index: pointer.cache_index,
          }));
        }
      }
    }
  }

  pub fn get_display_size() -> (u16, u16) {
    let display = DisplayInfo::all().ok().and_then(|displays| {
      let to_u16 =
        |value: u32| -> Option<u16> { u16::try_from(value).ok().filter(|&value| value > 0) };

      let display = displays
        .iter()
        .find(|display| display.is_primary)
        .or_else(|| displays.first())
        .and_then(|display| Some((to_u16(display.width)?, to_u16(display.height)?)));

      display
    });

    display.unwrap_or((1920, 1080))
  }
}
