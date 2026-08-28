use std::num::{NonZeroU16, NonZeroUsize};

use dxgi_capture_rs::{CaptureError, FrameMetadata};
use ironrdp::server::{BitmapUpdate, DisplayUpdate, PixelFormat};
use tokio::sync::mpsc::UnboundedSender;
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

pub struct Display;

impl crate::platform::traits::Display for Display {
  async fn capture(_width: u16, _height: u16, sender: UnboundedSender<DisplayUpdate>) {
    tokio::task::spawn_blocking(move || {
      let mut manager = match dxgi_capture_rs::DXGIManager::new(33) {
        Ok(manager) => manager,
        Err(error) => {
          eprintln!("Failed to initialize DXGI capture manager: {:?}", error);
          return;
        }
      };

      let mut first_frame = true;

      'outer: loop {
        match manager.capture_frame_components_with_metadata() {
          Ok((pixels, (width, height), metadata)) => {
            let updates = if first_frame {
              // first_frame = false;
              match Self::get_full_frame_update(&pixels, width, height) {
                Some(update) => vec![update],
                None => continue,
              }
            } else {
              Self::get_frame_updates_from_metadata(&pixels, width, height, &metadata)
            };

            if updates.is_empty() {
              std::thread::sleep(std::time::Duration::from_millis(10));
              continue;
            }

            for update in updates {
              if sender.send(update).is_err() {
                println!("display receiver dropped, stopped capturing");
                break 'outer;
              }
            }
          }
          Err(CaptureError::Timeout) => {
            std::thread::sleep(std::time::Duration::from_millis(16));
          }
          Err(error) => {
            eprintln!("Capture failed: {:?}", error);
            std::thread::sleep(std::time::Duration::from_millis(100));
          }
        }
      }
    });
  }
}

impl Display {
  pub fn size() -> (u16, u16) {
    unsafe {
      let width = GetSystemMetrics(SM_CXSCREEN) as u16;
      let height = GetSystemMetrics(SM_CYSCREEN) as u16;
      (width, height)
    }
  }

  fn get_full_frame_update(pixels: &[u8], width: usize, height: usize) -> Option<DisplayUpdate> {
    Some(DisplayUpdate::Bitmap(BitmapUpdate {
      x: 0,
      y: 0,
      format: PixelFormat::BgrX32,
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
        cropped.extend_from_slice(&pixels[source_start..source_end]);
      }

      let (Some(nz_w), Some(nz_h), Some(nz_stride)) = (
        NonZeroU16::new(w as u16),
        NonZeroU16::new(h as u16),
        NonZeroUsize::new(w * 4),
      ) else {
        continue;
      };

      updates.push(DisplayUpdate::Bitmap(ironrdp::server::BitmapUpdate {
        x: x as u16,
        y: y as u16,
        width: nz_w,
        height: nz_h,
        stride: nz_stride,
        format: PixelFormat::BgrX32,
        data: bytes::Bytes::from(cropped),
      }));
    }

    updates
  }
}
