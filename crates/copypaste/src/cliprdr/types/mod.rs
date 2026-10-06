#[cfg(feature = "server")]
use ironrdp::server::ServerEvent;
#[cfg(feature = "server")]
use tokio::sync::mpsc::UnboundedSender;

#[cfg(not(feature = "server"))]
use ironrdp::client::rdp::{RdpInputEvent, RdpInputSender};

pub mod clipboard_event;
#[cfg(any(feature = "client", feature = "server"))]
pub mod clipboard_file;
#[cfg(any(feature = "client", feature = "server"))]
pub mod download_state;

#[cfg(feature = "server")]
pub type Event = ServerEvent;
#[cfg(feature = "server")]
pub type Sender = Option<UnboundedSender<Event>>;

#[cfg(not(feature = "server"))]
pub type Event = RdpInputEvent;
#[cfg(not(feature = "server"))]
pub type Sender = Option<RdpInputSender>;
