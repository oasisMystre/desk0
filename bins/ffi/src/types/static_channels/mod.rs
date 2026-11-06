pub mod io;
pub mod terminal;

pub use io::*;
pub use terminal::*;

pub enum SseStaticChannelInputEvent {
  Io(SseIoInputEvent),
  Terminal(SseTerminalInputEvent),
}

impl Into<ironrdp::svc::SvcMessage> for SseStaticChannelInputEvent {
  fn into(self) -> ironrdp::svc::SvcMessage {
    match self {
      SseStaticChannelInputEvent::Io(event) => {
        let input: pdu::io::IoInputEvent = event.into();
        input.into()
      }
      SseStaticChannelInputEvent::Terminal(event) => {
        let input: pdu::terminal::TerminalInputEvent = event.into();
        input.into()
      }
    }
  }
}
