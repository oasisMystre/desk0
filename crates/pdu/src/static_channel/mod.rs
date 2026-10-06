mod input;
mod output;

pub use input::*;
use ironrdp::pdu::gcc::ChannelName;
pub use output::*;


pub const STATIC_CHANNEL_NAME: ChannelName = ChannelName::new(*b"desk0\0\0\0");
