use std::{
  borrow::Cow,
  fmt::Debug,
  sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
  },
};

use audio::audio::cpal_supported_stream_config_range_to_audio_format;
use cpal::{
  Device,
  traits::{DeviceTrait, HostTrait, StreamTrait},
};
use ironrdp::rdpsnd::{
  client::Rdpsnd,
  client::RdpsndClientHandler,
  pdu::{AudioFormat, PitchPdu, VolumePdu},
};
use ringbuf::{
  HeapProd, HeapRb,
  traits::{Consumer, Producer, Split},
};

#[derive(Debug)]
pub struct CandidateOutputDevice {
  pub device: Device,
  pub is_default: bool,
  pub formats: Vec<AudioFormat>,
}

pub struct SoundRdpsndBackend {
  audio_formats: Vec<AudioFormat>,
  candidates: Vec<CandidateOutputDevice>,

  active_stream: Option<cpal::Stream>,
  audio_sender: Option<HeapProd<u8>>,
  current_format: Option<AudioFormat>,

  volume: Arc<AtomicU32>,
  pitch: Arc<AtomicU32>,
}

impl Debug for SoundRdpsndBackend {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SoundRdpsndBackend")
      .field("audio_formats", &self.audio_formats)
      .field("candidates", &self.candidates)
      .finish()
  }
}
impl SoundRdpsndBackend {
  fn new() -> Self {
    let host = cpal::default_host();
    let mut candidates = vec![];
    let mut audio_formats = vec![];

    let default_output_device_name = host
      .default_output_device()
      .map(|device| device.to_string());

    if let Ok(devices) = host.output_devices() {
      for device in devices {
        if let Ok(configs) = device.supported_output_configs() {
          let mut formats = vec![];

          for config in configs {
            if let Some(format) = cpal_supported_stream_config_range_to_audio_format(config) {
              formats.push(format);
            }
          }

          let is_default = default_output_device_name
            .as_ref()
            .map(|name| name == &device.to_string())
            .unwrap_or(false);

          audio_formats.extend_from_slice(&formats);
          candidates.push(CandidateOutputDevice {
            device,
            formats,
            is_default,
          });
        }
      }
    }

    Self {
      candidates,
      audio_formats,
      active_stream: None,
      audio_sender: None,
      current_format: None,
      pitch: Arc::new(AtomicU32::new(0xFFFF_FFFF)),
      volume: Arc::new(AtomicU32::new(0x0001_0000)),
    }
  }

  fn start_stream(&mut self, format: &AudioFormat) {
    let candidate = self
      .candidates
      .iter()
      .find(|candidate| candidate.is_default && candidate.formats.contains(format))
      .or_else(|| {
        self
          .candidates
          .iter()
          .find(|candidate| candidate.formats.contains(format))
      })
      .or_else(|| self.candidates.first())
      .expect("No output device available");

    let buffer_size = format.n_avg_bytes_per_sec as usize;
    let ring = HeapRb::<u8>::new(buffer_size);
    let (sender, mut receiver) = ring.split();
    let config = cpal::StreamConfig {
      channels: format.n_channels,
      sample_rate: format.n_samples_per_sec,
      buffer_size: cpal::BufferSize::Default,
    };

    let stream = candidate
      .device
      .build_output_stream(
        config,
        {
          let volume = self.volume.clone();
          let channels = format.n_channels as usize;

          move |out: &mut [i16], _| {
            let volume_packed = volume.load(Ordering::Relaxed);
            let volume_left = (volume_packed >> 16) as u16;
            let volume_right = (volume_packed & 0xFFFF) as u16;

            let left_multipier = volume_left as f32 / 65535.0;
            let right_multipier = volume_right as f32 / 65535.0;

            for frame in out.chunks_mut(channels) {
              for (index, sample) in frame.iter_mut().enumerate() {
                let mut bytes = [0u8; 2];
                if receiver.pop_slice(&mut bytes) == 2 {
                  let raw_sample = i16::from_le_bytes(bytes) as f32;
                  let multiplier = if index == 0 {
                    left_multipier
                  } else {
                    right_multipier
                  };

                  *sample = (raw_sample * multiplier) as i16;
                } else {
                  *sample = 0
                }
              }
            }
          }
        },
        |err| tracing::error!("Audio stream error: {}", err),
        None,
      )
      .expect("Failed to build output stream");

    stream.play();

    self.audio_sender = Some(sender);
    self.active_stream = Some(stream);
    self.current_format = Some(format.clone());
  }
}

impl RdpsndClientHandler for SoundRdpsndBackend {
  fn get_formats(&self) -> &[ironrdp::rdpsnd::pdu::AudioFormat] {
    &self.audio_formats
  }

  fn wave(&mut self, format: &AudioFormat, _ts: u32, data: Cow<'_, [u8]>) {
    if self.current_format.as_ref() != Some(format) {
      self.start_stream(format);
    }

    if let Some(sender) = &mut self.audio_sender {
      let _ = sender.push_slice(&data);
    }
  }

  fn set_volume(&mut self, volume: VolumePdu) {
    let packed = ((volume.volume_left as u32) << 16) | (volume.volume_right as u32);
    self.volume.store(packed, Ordering::Relaxed);
  }

  fn set_pitch(&mut self, pitch: PitchPdu) {
    self.pitch.store(pitch.pitch, Ordering::Relaxed);
  }

  fn close(&mut self) {
    self.active_stream = None;
    self.audio_sender = None;
    self.current_format = None;
  }
}

impl SoundRdpsndBackend {
  pub fn build_backend() -> Rdpsnd {
    Rdpsnd::new(Box::new(SoundRdpsndBackend::new()))
  }
}
