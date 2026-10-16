use cpal::SupportedStreamConfigRange;
use ironrdp::rdpsnd::pdu::{AudioFormat, WaveFormat};

#[cfg(feature = "convert")]
const I16_MAX: f32 = 32768.0;
#[cfg(feature = "convert")]
const I32_MAX: f32 = 2147483648.0;

#[cfg(feature = "convert")]
#[inline(always)]
fn append_pcm16(output: &mut Vec<u8>, sample: f32) {
  let sample = if sample.is_finite() {
    sample.clamp(-1.0, 1.0)
  } else {
    0.0
  };

  let pcm16 = (sample * 32767.0) as i16;
  output.extend_from_slice(&pcm16.to_le_bytes());
}

#[cfg(feature = "convert")]
fn sample_width(format: pinray::SampleFormat) -> usize {
  match format {
    pinray::SampleFormat::I16 => 2,
    pinray::SampleFormat::F64 => 8,
    pinray::SampleFormat::I32 | pinray::SampleFormat::F32 => 4,
  }
}

#[cfg(feature = "convert")]
fn sample_to_f32(data: &[u8], format: pinray::SampleFormat, index: usize) -> f32 {
  let width = sample_width(format);
  let start = index * width;
  let bytes = &data[start..start + width];
  match format {
    pinray::SampleFormat::F32 => f32::from_le_bytes(bytes.try_into().unwrap()),
    pinray::SampleFormat::F64 => f64::from_le_bytes(bytes.try_into().unwrap()) as f32,
    pinray::SampleFormat::I16 => i16::from_le_bytes(bytes.try_into().unwrap()) as f32 / I16_MAX,
    pinray::SampleFormat::I32 => i32::from_le_bytes(bytes.try_into().unwrap()) as f32 / I32_MAX,
  }
}

#[cfg(feature = "convert")]
pub fn from_audio_frame_to_pcm16(frame: &pinray::AudioFrame) -> (Vec<u8>, u32) {
  let mut output = vec![];
  let width = sample_width(frame.sample_format);

  match &frame.data {
    pinray::AudioData::Interleaved(data) => {
      let channels = frame.channels as usize;

      if channels != 0 {
        let count = data.len() / width;
        let count = count - (count % channels);

        output.reserve(count * 2);

        for index in 0..count {
          append_pcm16(&mut output, sample_to_f32(data, frame.sample_format, index));
        }
      }
    }
    pinray::AudioData::Planar(channels) => {
      if !channels.is_empty() && channels.len() == frame.channels as usize {
        let count = channels
          .iter()
          .map(|channel| channel.len() / width)
          .min()
          .unwrap_or(0);

        output.reserve(count * channels.len() * 2);

        for index in 0..count {
          for channel in channels {
            append_pcm16(
              &mut output,
              sample_to_f32(channel, frame.sample_format, index),
            );
          }
        }
      }
    }
  }

  let timestamp_ms = frame
    .stream_time_ns
    .max(0)
    .saturating_div(1_000_000)
    .min(u32::MAX as i64) as u32;

  (output, timestamp_ms)
}

pub fn cpal_supported_stream_config_range_to_audio_format(
  config: SupportedStreamConfigRange,
) -> Option<AudioFormat> {
  let n_channels = config.channels();
  let n_samples_per_sec = config.max_sample_rate();
  let sample_format = config.sample_format();

  let bits_per_sample = match sample_format {
    cpal::SampleFormat::I16 => 16,
    cpal::SampleFormat::F64 => 64,
    cpal::SampleFormat::I32 | cpal::SampleFormat::F32 => 32,
    _ => return None,
  };

  let n_block_align = n_channels * (bits_per_sample / 8);
  let n_avg_bytes_per_sec = n_samples_per_sec * n_block_align as u32;

  Some(AudioFormat {
    n_channels,
    n_block_align,
    bits_per_sample,
    n_samples_per_sec,
    n_avg_bytes_per_sec,
    data: None,
    format: WaveFormat::PCM,
  })
}
