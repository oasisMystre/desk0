use std::collections::VecDeque;

use ipc::{IpcSharedBuffer, IpcSharedBufferField};
use ironrdp::rdpsnd::pdu::{AudioFormat, WaveFormat};
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct IpcAudioFormat {
  pub format: u16,
  pub n_channels: u16,
  pub n_block_align: u16,
  pub bits_per_sample: u16,
  pub n_samples_per_sec: u32,
  pub n_avg_bytes_per_sec: u32,
  pub data: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct IpcAudioSource {
  pub is_default: bool,
  pub formats: Vec<IpcAudioFormat>,
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct IpcPackedAudioWave {
  pub timestamp_ms: u32,
  pub pcm16: IpcSharedBufferField,
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct IpcUnpackedAudioWave {
  pub timestamp_ms: u32,
  pub pcm16: IpcSharedBuffer,
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub enum IpcPackedAudioEvent {
  Wave(IpcPackedAudioWave),
  AudioSource(Vec<IpcAudioSource>),
}

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub enum IpcUnpackedAudioEvent {
  Wave(IpcUnpackedAudioWave),
  AudioSource(Vec<IpcAudioSource>),
}

impl From<AudioFormat> for IpcAudioFormat {
  fn from(value: AudioFormat) -> IpcAudioFormat {
    IpcAudioFormat {
      data: value.data,
      format: 1,
      n_channels: value.n_channels,
      n_block_align: value.n_block_align,
      bits_per_sample: value.bits_per_sample,
      n_samples_per_sec: value.n_samples_per_sec,
      n_avg_bytes_per_sec: value.n_avg_bytes_per_sec,
    }
  }
}

impl From<IpcAudioFormat> for AudioFormat {
  fn from(value: IpcAudioFormat) -> Self {
    Self {
      data: value.data,
      format: WaveFormat::PCM,
      n_channels: value.n_channels,
      n_block_align: value.n_block_align,
      bits_per_sample: value.bits_per_sample,
      n_samples_per_sec: value.n_samples_per_sec,
      n_avg_bytes_per_sec: value.n_avg_bytes_per_sec,
    }
  }
}

impl IpcPackedAudioWave {
  pub fn unpack(
    self,
    buffers: &mut VecDeque<IpcSharedBuffer>,
  ) -> anyhow::Result<IpcUnpackedAudioWave> {
    let pcm16 = buffers
      .pop_front()
      .ok_or_else(|| anyhow::anyhow!("missing audio wave at position {}", self.pcm16.0))?;

    Ok(IpcUnpackedAudioWave {
      pcm16,
      timestamp_ms: self.timestamp_ms,
    })
  }
}

impl IpcPackedAudioEvent {
  pub fn unpack(
    self,
    buffers: &mut VecDeque<IpcSharedBuffer>,
  ) -> anyhow::Result<IpcUnpackedAudioEvent> {
    Ok(match self {
      IpcPackedAudioEvent::Wave(wave) => IpcUnpackedAudioEvent::Wave(wave.unpack(buffers)?),
      IpcPackedAudioEvent::AudioSource(sources) => IpcUnpackedAudioEvent::AudioSource(sources),
    })
  }
}
