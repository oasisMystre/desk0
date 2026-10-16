use std::num::{NonZeroU16, NonZeroUsize};

use ironrdp::server::{BitmapUpdate, DisplayUpdate, PixelFormat};

pub fn full_frame_update(
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

pub fn damage_updates(
  pixels: &[u8],
  width: usize,
  height: usize,
  offset_x: u16,
  offset_y: u16,
  damage: Option<&[pinray::Rect]>,
) -> Vec<DisplayUpdate> {
  let Some(damage) = damage else {
    return full_frame_update(pixels, width, height, offset_x, offset_y)
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
