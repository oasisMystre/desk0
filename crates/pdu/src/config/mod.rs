use ironrdp::{
  core::{Decode, Encode},
  svc::SvcEncode,
};

pub struct ConfigPdu {
  pub enable_audio: bool,
  pub enable_mouse: bool,
  pub enable_keyboard: bool,
  pub enable_file_copy: bool,
}

impl<'de> Decode<'de> for ConfigPdu {
  fn decode(src: &mut ironrdp::core::ReadCursor<'de>) -> ironrdp::core::DecodeResult<Self> {
    let enable_audio = src.read_u8() != 0;
    let enable_mouse = src.read_u8() != 0;
    let enable_keyboard = src.read_u8() != 0;
    let enable_file_copy = src.read_u8() != 0;

    Ok(Self {
      enable_audio,
      enable_mouse,
      enable_keyboard,
      enable_file_copy,
    })
  }
}

impl Encode for ConfigPdu {
  fn encode(&self, dst: &mut ironrdp::core::WriteCursor<'_>) -> ironrdp::core::EncodeResult<()> {
    dst.write_u8(self.enable_audio as u8);
    dst.write_u8(self.enable_mouse as u8);
    dst.write_u8(self.enable_keyboard as u8);
    dst.write_u8(self.enable_file_copy as u8);

    Ok(())
  }

  fn name(&self) -> &'static str {
    "ConfigPdu"
  }

  fn size(&self) -> usize {
    4
  }
}

impl SvcEncode for ConfigPdu {}
