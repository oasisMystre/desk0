use std::num::{NonZeroU16, NonZeroUsize};
use std::time::Duration;

use ironrdp::server::{BitmapUpdate, DisplayUpdate, PixelFormat, ServerEvent, RdpsndServerMessage};
use pinray::{CaptureSession, FrameData, AudioCapture, VideoCaptureTarget, SourceId, CaptureEvent};
use tokio::sync::mpsc::{Sender, UnboundedSender};

#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

pub struct Display;

fn audio_frame_to_pcm_i16(frame: &pinray::AudioFrame) -> Vec<i16> {
  let channels = frame.channels as usize;
  
  let samples_f32: Vec<f32> = match &frame.data {
    pinray::AudioData::Interleaved(bytes) => {
      match frame.sample_format {
        pinray::SampleFormat::F32 => {
          bytes.chunks_exact(4)
            .map(|chunk| f32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect()
        }
        pinray::SampleFormat::I16 => {
          bytes.chunks_exact(2)
            .map(|chunk| {
              let val = i16::from_ne_bytes([chunk[0], chunk[1]]) as f32;
              val / 32768.0
            })
            .collect()
        }
        pinray::SampleFormat::I32 => {
          bytes.chunks_exact(4)
            .map(|chunk| {
              let val = i32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) as f32;
              val / 2147483648.0
            })
            .collect()
        }
        pinray::SampleFormat::F64 => {
          bytes.chunks_exact(8)
            .map(|chunk| {
              let val = f64::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7]]) as f32;
              val
            })
            .collect()
        }
      }
    }
    pinray::AudioData::Planar(planes) => {
      let sample_size = match frame.sample_format {
        pinray::SampleFormat::I16 => 2,
        pinray::SampleFormat::F32 | pinray::SampleFormat::I32 => 4,
        pinray::SampleFormat::F64 => 8,
      };
      
      let num_samples = if planes.is_empty() { 0 } else { planes[0].len() / sample_size };
      let mut interleaved = Vec::with_capacity(num_samples * channels);
      
      for s in 0..num_samples {
        for c in 0..channels {
          if let Some(plane) = planes.get(c) {
            let offset = s * sample_size;
            if offset + sample_size <= plane.len() {
              let val = match frame.sample_format {
                pinray::SampleFormat::F32 => {
                  f32::from_ne_bytes([plane[offset], plane[offset+1], plane[offset+2], plane[offset+3]])
                }
                pinray::SampleFormat::I16 => {
                  let i_val = i16::from_ne_bytes([plane[offset], plane[offset+1]]) as f32;
                  i_val / 32768.0
                }
                pinray::SampleFormat::I32 => {
                  let i_val = i32::from_ne_bytes([plane[offset], plane[offset+1], plane[offset+2], plane[offset+3]]) as f32;
                  i_val / 2147483648.0
                }
                pinray::SampleFormat::F64 => {
                  f64::from_ne_bytes([plane[offset], plane[offset+1], plane[offset+2], plane[offset+3], plane[offset+4], plane[offset+5], plane[offset+6], plane[offset+7]]) as f32
                }
              };
              interleaved.push(val);
            }
          }
        }
      }
      interleaved
    }
  };
  
  let stereo_f32: Vec<f32> = if channels == 1 {
    let mut out = Vec::with_capacity(samples_f32.len() * 2);
    for sample in samples_f32 {
      out.push(sample);
      out.push(sample);
    }
    out
  } else if channels > 2 {
    let num_frames = samples_f32.len() / channels;
    let mut out = Vec::with_capacity(num_frames * 2);
    for f in 0..num_frames {
      out.push(samples_f32[f * channels]);
      out.push(samples_f32[f * channels + 1]);
    }
    out
  } else {
    samples_f32
  };
  
  stereo_f32.into_iter()
    .map(|s| {
      let clamped = s.clamp(-1.0, 1.0);
      (clamped * 32767.0) as i16
    })
    .collect()
}

fn resample_stereo_i16(input: &[i16], in_rate: u32, out_rate: u32) -> Vec<i16> {
  if in_rate == out_rate {
    return input.to_vec();
  }
  
  let num_input_frames = input.len() / 2;
  let num_output_frames = (num_input_frames as u64 * out_rate as u64 / in_rate as u64) as usize;
  
  let mut output = Vec::with_capacity(num_output_frames * 2);
  for i in 0..num_output_frames {
    let pos = i as f64 * (in_rate as f64) / (out_rate as f64);
    let idx = pos.floor() as usize;
    let frac = pos - idx as f64;
    
    if idx + 1 < num_input_frames {
      let l0 = input[idx * 2];
      let l1 = input[(idx + 1) * 2];
      let r0 = input[idx * 2 + 1];
      let r1 = input[(idx + 1) * 2 + 1];
      
      let l = l0 as f64 * (1.0 - frac) + l1 as f64 * frac;
      let r = r0 as f64 * (1.0 - frac) + r1 as f64 * frac;
      
      output.push(l as i16);
      output.push(r as i16);
    } else if idx < num_input_frames {
      output.push(input[idx * 2]);
      output.push(input[idx * 2 + 1]);
    }
  }
  output
}

fn crop_rect(
  data: &[u8],
  stride: usize,
  rx: i32,
  ry: i32,
  rw: u32,
  rh: u32,
) -> Vec<u8> {
  let mut cropped = Vec::with_capacity((rw * rh * 4) as usize);
  for y in ry..(ry + rh as i32) {
    let start_offset = y as usize * stride + rx as usize * 4;
    let end_offset = start_offset + rw as usize * 4;
    if end_offset <= data.len() {
      cropped.extend_from_slice(&data[start_offset..end_offset]);
    }
  }
  cropped
}

#[cfg(target_os = "windows")]
fn capture_screen_gdi(width: u16, height: u16) -> anyhow::Result<Vec<u8>> {
  use windows::Win32::Graphics::Gdi::*;
  use windows::Win32::UI::WindowsAndMessaging::*;

  unsafe {
    let hdc_screen = GetDC(None);
    if hdc_screen.0.is_null() {
      anyhow::bail!("Failed to get screen device context");
    }

    let hdc_screen_mem = CreateCompatibleDC(Some(hdc_screen));
    if hdc_screen_mem.0.is_null() {
      ReleaseDC(None, hdc_screen);
      anyhow::bail!("Failed to create compatible device context");
    }

    let hbitmap = CreateCompatibleBitmap(hdc_screen, width as i32, height as i32);
    if hbitmap.0.is_null() {
      DeleteDC(hdc_screen_mem);
      ReleaseDC(None, hdc_screen);
      anyhow::bail!("Failed to create compatible bitmap");
    }

    let old_object = SelectObject(hdc_screen_mem, hbitmap.into());
    if let Err(error) = BitBlt(
      hdc_screen_mem,
      0,
      0,
      width as i32,
      height as i32,
      Some(hdc_screen),
      0,
      0,
      SRCCOPY,
    ) {
      SelectObject(hdc_screen_mem, old_object);
      DeleteObject(hbitmap.into());
      DeleteDC(hdc_screen_mem);
      ReleaseDC(None, hdc_screen);
      return Err(error.into());
    }

    SelectObject(hdc_screen_mem, old_object);

    let mut bmi = BITMAPINFO {
      bmiHeader: BITMAPINFOHEADER {
        biPlanes: 1,
        biBitCount: 32,
        biWidth: width as i32,
        biHeight: -(height as i32),
        biCompression: BI_RGB.0 as u32,
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        ..Default::default()
      },
      ..Default::default()
    };

    let buffer_size = width as usize * height as usize * 4;
    let mut buffer = vec![0u8; buffer_size];

    let result = GetDIBits(
      hdc_screen_mem,
      hbitmap,
      0,
      height as u32,
      Some(buffer.as_mut_ptr() as *mut _),
      &mut bmi,
      DIB_RGB_COLORS,
    );

    DeleteObject(hbitmap.into());
    DeleteDC(hdc_screen_mem);
    ReleaseDC(None, hdc_screen);

    if result == 0 {
      anyhow::bail!("GetDIBits failed to copy display buffer");
    }

    Ok(buffer)
  }
}

#[cfg(target_os = "windows")]
async fn run_gdi_capture_loop(width: u16, height: u16, sender: Sender<DisplayUpdate>) {
  let mut ticker = tokio::time::interval(Duration::from_millis(100));
  loop {
    ticker.tick().await;
    let frame_data = match capture_screen_gdi(width, height) {
      Ok(data) => data,
      Err(error) => {
        eprintln!("GDI capture error: {}", error);
        continue;
      }
    };

    let bitmap = BitmapUpdate {
      x: 0,
      y: 0,
      format: PixelFormat::BgrX32,
      data: bytes::Bytes::from(frame_data),
      width: std::num::NonZeroU16::new(width).unwrap(),
      height: std::num::NonZeroU16::new(height).unwrap(),
      stride: std::num::NonZeroUsize::new(width as usize * 4).unwrap(),
    };

    if sender.send(DisplayUpdate::Bitmap(bitmap)).await.is_err() {
      break;
    }
  }
}

impl crate::platform::traits::Display for Display {
  async fn capture(
    _width: u16,
    _height: u16,
    sender: Sender<DisplayUpdate>,
    audio_sender: Option<UnboundedSender<ServerEvent>>,
  ) {
    let mut builder = CaptureSession::builder()
      .video_target(VideoCaptureTarget::Display(SourceId::new("auto")));
    if audio_sender.is_some() {
      builder = builder.audio(AudioCapture::SystemMix);
    }
    
    let session = builder.clone().build().or_else(|_err| {
      let dxgi_builder = builder.clone().backend_preference(pinray::BackendPreference::WindowsDxgi);
      dxgi_builder.build()
    });

    if let Ok(mut s) = session {
      if s.start().is_ok() {
        println!("Successfully started pinray capture session.");
        
        loop {
          match s.next_event(Some(Duration::from_secs(5))) {
            Ok(event) => match event {
              CaptureEvent::Video(frame) => {
                let damage_rects = frame.damage.clone().unwrap_or_else(|| {
                  vec![pinray::Rect {
                    x: 0,
                    y: 0,
                    width: frame.width,
                    height: frame.height,
                  }]
                });
                
                match frame.data {
                  FrameData::Host(ref data) => {
                    for rect in damage_rects {
                      let cropped_data = crop_rect(
                        data,
                        frame.stride as usize,
                        rect.x,
                        rect.y,
                        rect.width,
                        rect.height,
                      );
                      
                      let update = DisplayUpdate::Bitmap(BitmapUpdate {
                        x: rect.x as u16,
                        y: rect.y as u16,
                        format: PixelFormat::BgrX32,
                        data: bytes::Bytes::from(cropped_data),
                        width: unsafe { NonZeroU16::new_unchecked(rect.width as u16) },
                        height: unsafe { NonZeroU16::new_unchecked(rect.height as u16) },
                        stride: unsafe { NonZeroUsize::new_unchecked(rect.width as usize * 4) },
                      });
                      
                      if sender.send(update).await.is_err() {
                        break;
                      }
                    }
                  }
                  _ => {}
                }
              }
              CaptureEvent::Audio(ref frame) => {
                if let Some(ref ref_audio_sender) = audio_sender {
                  let pcm_i16 = audio_frame_to_pcm_i16(frame);
                  let target_rate = 44100;
                  let resampled = resample_stereo_i16(&pcm_i16, frame.sample_rate, target_rate);
                  
                  let pcm_bytes: Vec<u8> = resampled.iter()
                    .flat_map(|sample| sample.to_le_bytes())
                    .collect();
                    
                  let ts = (frame.stream_time_ns / 1_000_000) as u32;
                  let _ = ref_audio_sender.send(ServerEvent::Rdpsnd(RdpsndServerMessage::Wave(pcm_bytes, ts)));
                }
              }
              CaptureEvent::Gap(_) => {}
              CaptureEvent::End => break,
            }
            Err(_) => {}
          }
        }
        
        let _ = s.stop();
        return;
      }
    }

    #[cfg(target_os = "windows")]
    {
      eprintln!("Display capture backend (pinray) failed or unavailable. Falling back to native GDI screen capture...");
      run_gdi_capture_loop(_width, _height, sender).await;
    }

    #[cfg(not(target_os = "windows"))]
    {
      eprintln!("Display capture backend (pinray) failed and native GDI fallback is only supported on Windows.");
    }
  }
}

impl Display {
  pub fn get_desktop_size() -> (u16, u16) {
    #[cfg(target_os = "windows")]
    unsafe {
      let width = GetSystemMetrics(SM_CXSCREEN) as u16;
      let height = GetSystemMetrics(SM_CYSCREEN) as u16;
      (width, height)
    }
    #[cfg(not(target_os = "windows"))]
    {
      (1280, 720)
    }
  }
}
