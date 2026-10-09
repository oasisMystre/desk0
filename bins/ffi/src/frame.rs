use std::sync::{Arc, Mutex};

use irondash_texture::{
  BoxedPixelData, PayloadProvider, PixelData, PixelDataProvider, SendableTexture, Texture,
};
use ironrdp::{client::rdp::DesktopUpdate, pdu::geometry::Rectangle};
use rayon::{
  iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator},
  slice::{ParallelSlice, ParallelSliceMut},
};

pub struct Frame {
  width: u16,
  height: u16,
  pixels: Arc<Vec<u8>>,
}

pub struct FrameSource(Arc<Mutex<Frame>>);

pub struct TextureHandle {
  frame: Arc<Mutex<Frame>>,
  texture: Mutex<Option<Arc<SendableTexture<BoxedPixelData>>>>,
}

impl PayloadProvider<BoxedPixelData> for FrameSource {
  fn get_payload(&self) -> BoxedPixelData {
    let frame = self.0.lock().unwrap();
    frame.boxed()
  }
}

impl Frame {
  fn boxed(&self) -> BoxedPixelData {
    Box::new(Frame {
      width: self.width,
      height: self.height,
      pixels: self.pixels.clone(),
    })
  }
}

impl PixelDataProvider for Frame {
  fn get(&self) -> PixelData<'_> {
    PixelData {
      data: &self.pixels,
      width: self.width as i32,
      height: self.height as i32,
    }
  }
}

impl TextureHandle {
  pub fn new(width: u16, height: u16) -> Self {
    Self {
      frame: Arc::new(Mutex::new(Frame {
        width,
        height,
        pixels: Arc::new(vec![0; width as usize * height as usize * 4]),
      })),
      texture: Mutex::new(None),
    }
  }

  pub fn create(&self, engine_handle: i64) -> anyhow::Result<i64> {
    let source = Arc::new(FrameSource(self.frame.clone()));
    let texture = Texture::new_with_provider(engine_handle, source)?;
    let id = texture.id();
    *self.texture.lock().unwrap() = Some(texture.into_sendable_texture());

    Ok(id)
  }

  pub fn apply_desktop_update(&self, update: DesktopUpdate) {
    let region = update.region();
    let width = update.width().get();
    let height = update.height().get();

    let mut frame = self.frame.lock().unwrap();

    Self::ensure_size(&mut frame, width, height);

    let fwidth = frame.width as usize;
    let rtop = region.top as usize;
    let rleft = region.left as usize;
    let rwidth = region.width() as usize;
    let rheight = region.height() as usize;

    let source = update.buffer();
    let pixels = Arc::make_mut(&mut frame.pixels);

    let start_index = rtop * fwidth * 4;
    let end_index = (rtop + rheight) * fwidth * 4;
    let target_rows = &mut pixels[start_index..end_index];

    target_rows
      .par_chunks_exact_mut(fwidth * 4)
      .zip(source.par_chunks_exact(rwidth))
      .for_each(|(destination_frame_row, source_row)| {
        let destination_segment = &mut destination_frame_row[(rleft * 4)..((rleft + rwidth) * 4)];
        for (destination, &pixel) in destination_segment.chunks_exact_mut(4).zip(source_row) {
          write_pixel(destination, pixel);
        }
      });

    drop(frame);

    self.update();
  }

  pub fn apply_image(&self, width: u16, height: u16, buffer: &[u32]) {
    let mut frame = self.frame.lock().unwrap();
    Self::ensure_size(&mut frame, width, height);

    let width = width as usize;
    let height = height as usize;
    let frame_width = frame.width as usize;

    let pixels = Arc::make_mut(&mut frame.pixels);

    (0..height).into_par_iter().for_each(|y| {
      let destination_start = y * frame_width * 4;
      let source_start = y * width;

      let source_row = &buffer[source_start..source_start + width];
      let destination_row = unsafe {
        let ptr = pixels.as_ptr().cast_mut().add(destination_start);
        std::slice::from_raw_parts_mut(ptr, width * 4)
      };

      for (destination, &pixel) in destination_row.chunks_exact_mut(4).zip(source_row) {
        write_pixel(destination, pixel);
      }
    });

    drop(frame);
    self.update();
  }

  fn ensure_size(frame: &mut Frame, width: u16, height: u16) {
    let aligned_width = ((width as usize + 63) & !63) as u16;

    if frame.width != aligned_width || frame.height != height {
      frame.width = aligned_width;
      frame.height = height;
      let len = (aligned_width as usize) * (height as usize) * 4;
      let mut pixels = Vec::with_capacity(len);

      unsafe {
        pixels.set_len(len);
      }

      frame.pixels = Arc::new(pixels);
    }
  }

  fn update(&self) {
    if let Some(texture) = self.texture.lock().unwrap().as_ref() {
      texture.mark_frame_available();
    }
  }
}

#[inline]
fn write_pixel(destination: &mut [u8], pixel: u32) {
  destination.copy_from_slice(&(pixel | 0xFF00_0000).to_le_bytes());
}
