use std::num::{NonZeroU16, NonZeroUsize};
use std::time::Duration;

use ironrdp::server::{BitmapUpdate, DisplayUpdate, PixelFormat, ServerEvent, RdpsndServerMessage};
use pinray::{CaptureSession, FrameData, AudioCapture, VideoCaptureTarget, SourceId, CaptureEvent};
use tokio::sync::mpsc::UnboundedSender;

#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

// Define constants to avoid magic numbers and make the code legible
const BYTES_PER_F32_SAMPLE: usize = 4;
const BYTES_PER_I16_SAMPLE: usize = 2;
const BYTES_PER_I32_SAMPLE: usize = 4;
const BYTES_PER_F64_SAMPLE: usize = 8;
const BYTES_PER_PIXEL: usize = 4; // BgrX32/RgbX32 uses 4 bytes per pixel

const MAX_I16_VALUE_AS_FLOAT: f32 = 32768.0;
const MAX_I32_VALUE_AS_FLOAT: f32 = 2147483648.0;

const AUDIO_SAMPLE_MULTIPLIER: f32 = 32767.0;
const AUDIO_SAMPLE_CLAMP_MIN: f32 = -1.0;
const AUDIO_SAMPLE_CLAMP_MAX: f32 = 1.0;

const DEFAULT_RDP_AUDIO_SAMPLE_RATE: u32 = 44100;
const NANOSECONDS_PER_MILLISECOND: i64 = 1_000_000;
const CAPTURE_TIMEOUT_SECONDS: u64 = 5;

const FALLBACK_DESKTOP_WIDTH: u16 = 1280;
const FALLBACK_DESKTOP_HEIGHT: u16 = 720;

pub struct Display;

fn audio_frame_to_pcm_i16(frame: &pinray::AudioFrame) -> Vec<i16> {
  let channels = frame.channels as usize;
  
  let samples_f32: Vec<f32> = match &frame.data {
    pinray::AudioData::Interleaved(bytes) => {
      match frame.sample_format {
        pinray::SampleFormat::F32 => {
          bytes.chunks_exact(BYTES_PER_F32_SAMPLE)
            .map(|chunk| f32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect()
        }
        pinray::SampleFormat::I16 => {
          bytes.chunks_exact(BYTES_PER_I16_SAMPLE)
            .map(|chunk| {
              let val = i16::from_ne_bytes([chunk[0], chunk[1]]) as f32;
              val / MAX_I16_VALUE_AS_FLOAT
            })
            .collect()
        }
        pinray::SampleFormat::I32 => {
          bytes.chunks_exact(BYTES_PER_I32_SAMPLE)
            .map(|chunk| {
              let val = i32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) as f32;
              val / MAX_I32_VALUE_AS_FLOAT
            })
            .collect()
        }
        pinray::SampleFormat::F64 => {
          bytes.chunks_exact(BYTES_PER_F64_SAMPLE)
            .map(|chunk| {
              f64::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7]]) as f32
            })
            .collect()
        }
      }
    }
    pinray::AudioData::Planar(planes) => {
      let sample_size = match frame.sample_format {
        pinray::SampleFormat::I16 => BYTES_PER_I16_SAMPLE,
        pinray::SampleFormat::F32 | pinray::SampleFormat::I32 => BYTES_PER_F32_SAMPLE,
        pinray::SampleFormat::F64 => BYTES_PER_F64_SAMPLE,
      };
      
      let num_samples = if planes.is_empty() { 0 } else { planes[0].len() / sample_size };
      let mut interleaved = Vec::with_capacity(num_samples * channels);
      
      for sample_index in 0..num_samples {
        for channel_index in 0..channels {
          if let Some(plane) = planes.get(channel_index) {
            let offset = sample_index * sample_size;
            if offset + sample_size <= plane.len() {
              let val = match frame.sample_format {
                pinray::SampleFormat::F32 => {
                  f32::from_ne_bytes([plane[offset], plane[offset+1], plane[offset+2], plane[offset+3]])
                }
                pinray::SampleFormat::I16 => {
                  let i_val = i16::from_ne_bytes([plane[offset], plane[offset+1]]) as f32;
                  i_val / MAX_I16_VALUE_AS_FLOAT
                }
                pinray::SampleFormat::I32 => {
                  let i_val = i32::from_ne_bytes([plane[offset], plane[offset+1], plane[offset+2], plane[offset+3]]) as f32;
                  i_val / MAX_I32_VALUE_AS_FLOAT
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
    .map(|sample| {
      let clamped = sample.clamp(AUDIO_SAMPLE_CLAMP_MIN, AUDIO_SAMPLE_CLAMP_MAX);
      (clamped * AUDIO_SAMPLE_MULTIPLIER) as i16
    })
    .collect()
}

fn resample_stereo_i16(input: &[i16], input_sample_rate: u32, output_sample_rate: u32) -> Vec<i16> {
  if input_sample_rate == output_sample_rate {
    return input.to_vec();
  }
  
  let num_input_frames = input.len() / 2;
  let num_output_frames = (num_input_frames as u64 * output_sample_rate as u64 / input_sample_rate as u64) as usize;
  
  let mut output = Vec::with_capacity(num_output_frames * 2);
  for i in 0..num_output_frames {
    let position = i as f64 * (input_sample_rate as f64) / (output_sample_rate as f64);
    let index = position.floor() as usize;
    let fractional = position - index as f64;
    
    if index + 1 < num_input_frames {
      let left_sample_0 = input[index * 2];
      let left_sample_1 = input[(index + 1) * 2];
      let right_sample_0 = input[index * 2 + 1];
      let right_sample_1 = input[(index + 1) * 2 + 1];
      
      let left = left_sample_0 as f64 * (1.0 - fractional) + left_sample_1 as f64 * fractional;
      let right = right_sample_0 as f64 * (1.0 - fractional) + right_sample_1 as f64 * fractional;
      
      output.push(left as i16);
      output.push(right as i16);
    } else if index < num_input_frames {
      output.push(input[index * 2]);
      output.push(input[index * 2 + 1]);
    }
  }
  output
}

fn crop_rect(
  data: &[u8],
  stride: usize,
  rect_x: i32,
  rect_y: i32,
  rect_width: u32,
  rect_height: u32,
) -> Vec<u8> {
  let mut cropped = Vec::with_capacity((rect_width * rect_height * BYTES_PER_PIXEL as u32) as usize);
  for y in rect_y..(rect_y + rect_height as i32) {
    let start_offset = y as usize * stride + rect_x as usize * BYTES_PER_PIXEL;
    let end_offset = start_offset + rect_width as usize * BYTES_PER_PIXEL;
    if end_offset <= data.len() {
      cropped.extend_from_slice(&data[start_offset..end_offset]);
    }
  }
  cropped
}

impl crate::platform::traits::Display for Display {
  async fn capture(
    _width: u16,
    _height: u16,
    display_update_sender: UnboundedSender<DisplayUpdate>,
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

    if let Ok(mut active_session) = session {
      if active_session.start().is_ok() {
        println!("Successfully started pinray capture session.");
        
        loop {
          match active_session.next_event(Some(Duration::from_secs(CAPTURE_TIMEOUT_SECONDS))) {
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
                        stride: unsafe { NonZeroUsize::new_unchecked(rect.width as usize * BYTES_PER_PIXEL) },
                      });
                      
                      if display_update_sender.send(update).is_err() {
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
                  let resampled = resample_stereo_i16(&pcm_i16, frame.sample_rate, DEFAULT_RDP_AUDIO_SAMPLE_RATE);
                  
                  let pcm_bytes: Vec<u8> = resampled.iter()
                    .flat_map(|sample| sample.to_le_bytes())
                    .collect();
                    
                  let timestamp_ms = (frame.stream_time_ns / NANOSECONDS_PER_MILLISECOND) as u32;
                  let _ = ref_audio_sender.send(ServerEvent::Rdpsnd(RdpsndServerMessage::Wave(pcm_bytes, timestamp_ms)));
                }
              }
              CaptureEvent::Gap(_) => {}
              CaptureEvent::End => break,
            }
            Err(_) => {}
          }
        }
        
        let _ = active_session.stop();
        return;
      }
    }

    eprintln!("Display capture backend (pinray) failed or unavailable.");
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
      (FALLBACK_DESKTOP_WIDTH, FALLBACK_DESKTOP_HEIGHT)
    }
  }
}
