mod input;
mod output;

pub use input::*;
pub use output::*;

use ironrdp::pdu::gcc::ChannelName;

pub const IO_STATIC_CHANNEL_NAME: ChannelName = ChannelName::new(*b"desk0io\0");
