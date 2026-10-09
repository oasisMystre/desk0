use pinray::{AudioFrame, SampleFormat};

const I16_MAX: f32 = 32768.0;
const I32_MAX: f32 = 2147483648.0;

#[inline(always)]
fn append_pcm16(output: &mut Vec<u8>, sample: f32) {
  let sample = if sample.is_finite() {
    sample.clamp(-1.0, 1.0)
  } else {
    0.0
  };

  output.extend_from_slice(&((sample * 32767.0) as i16).to_le_bytes());
}

fn sample_width(format: SampleFormat) -> usize {
  match format {
    SampleFormat::I16 => 2,
    SampleFormat::F64 => 8,
    SampleFormat::I32 | SampleFormat::F32 => 4,
  }
}

fn sample_to_f32(data: &[u8], format: SampleFormat, index: usize) -> f32 {
  let width = sample_width(format);
  let start = index * width;
  let bytes = &data[start..start + width];
  match format {
    SampleFormat::F32 => f32::from_le_bytes(bytes.try_into().unwrap()),
    SampleFormat::F64 => f64::from_le_bytes(bytes.try_into().unwrap()) as f32,
    SampleFormat::I16 => i16::from_le_bytes(bytes.try_into().unwrap()) as f32 / I16_MAX,
    SampleFormat::I32 => i32::from_le_bytes(bytes.try_into().unwrap()) as f32 / I32_MAX,
  }
}

pub fn from_audio_frame_to_wav(frame: &AudioFrame) -> (Vec<u8>, u32) {
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
          append_pcm16(
            &mut output,
            sample_to_f32(&data, frame.sample_format, index),
          );
        }
      }
    }
    pinray::AudioData::Planar(channels) => {
      if channels.len() != frame.channels as usize && !channels.is_empty() {
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
