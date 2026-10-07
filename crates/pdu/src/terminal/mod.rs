mod input;
mod output;

pub use input::*;
pub use output::*;

use ironrdp::pdu::gcc::ChannelName;

pub const TERMINAL_STATIC_CHANNEL_NAME: ChannelName = ChannelName::new(*b"termpty\0");
