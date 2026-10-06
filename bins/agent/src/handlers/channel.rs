use ironrdp::{
  core::impl_as_any,
  pdu::{PduResult, gcc::ChannelName},
  server::StaticChannelFactory,
  svc::{SvcMessage, SvcProcessor, SvcServerProcessor},
};
use pdu::{
  static_channel::{StaticChannelInput, StaticChannelOutput},
  terminal::{TerminalInput, TerminalOutput},
};
use tokio::{
  sync::mpsc::{Receiver, Sender, channel},
  task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use crate::handlers::terminal::Terminal;

#[derive(Debug)]
struct Desk0StaticChannel {
  input_sender: Sender<Vec<u8>>,
  input_receiver: Option<Receiver<Vec<u8>>>,
  output_sender: Sender<TerminalOutput>,
  output_receiver: Receiver<TerminalOutput>,

  terminal: Option<Terminal>,
  thread: Option<JoinHandle<()>>,
  cancellation_token: CancellationToken,
}

#[derive(Debug)]
pub struct Desk0StaticChannelFactory;

impl Desk0StaticChannel {
  fn run(&mut self) {
    let Some(terminal) = self.terminal.take() else {
      return;
    };

    let Some(mut receiver) = self.input_receiver.take() else {
      return;
    };

    let output_sender = self.output_sender.clone();
    let cancellation_token = self.cancellation_token.clone();

    let handle = tokio::spawn(async move {
      let mut stdin = terminal.stdin;
      let mut stdout = terminal.stdout;
      let mut stderr = terminal.stderr;

      loop {
        tokio::select! {
            _ = cancellation_token.cancelled() => break,
           result = receiver.recv() => {
               match result {
                  Some(data) => {
                      match Terminal::write(&mut stdin, &data).await {
                        Ok(()) => {},
                        Err(error) => {
                            tracing::error!(?error, "stdin error");
                            break;
                        },
                    }
                  },
                  None => break,
              }
           }
           result = Terminal::read(&mut stdout) => {
               match result {
                  Ok(data) if !data.is_empty() => {
                      if output_sender.send(TerminalOutput::StdOut(data)).await.is_err() {
                          break;
                      }
                  },
                  Ok(_) => {}
                  Err(error) => {
                      tracing::error!(?error, "stdout error");
                      break;
                  },
              }
           },
           result = Terminal::read(&mut stderr) => {
               match result {
                  Ok(data) if !data.is_empty() => {
                      if output_sender.send(TerminalOutput::StdErr(data)).await.is_err() {
                          break;
                      }
                  },
                  Ok(_) => {}
                  Err(error) => {
                      tracing::error!(?error, "stdout error");
                      break;
                  },
              }
           }
        }
      }
    });

    self.thread = Some(handle);
  }

  pub fn drain(&mut self) -> Vec<SvcMessage> {
    let mut output: Vec<SvcMessage> = Vec::new();

    while let Ok(data) = self.output_receiver.try_recv() {
      let data = StaticChannelOutput::Terminal(data);
      output.push(data.into());
    }

    output
  }
}

impl_as_any!(Desk0StaticChannel);
impl SvcServerProcessor for Desk0StaticChannel {}

impl StaticChannelFactory for Desk0StaticChannelFactory {
  fn attach(&self, acceptor: &mut ironrdp::server::Acceptor) {
    let (input_sender, input_receiver) = channel(64);
    let (output_sender, output_receiver) = channel(64);

    let mut channel = Desk0StaticChannel {
      thread: None,
      input_sender,
      output_sender,
      output_receiver,
      input_receiver: Some(input_receiver),
      cancellation_token: CancellationToken::new(),
      terminal: Some(Terminal::spawn().expect("initialize terminal")),
    };

    channel.run();
    acceptor.attach_static_channel(channel);
  }
}

impl SvcProcessor for Desk0StaticChannel {
  fn channel_name(&self) -> ironrdp::pdu::gcc::ChannelName {
    ChannelName::new(*b"desk0123")
  }

  fn process(&mut self, payload: &[u8]) -> PduResult<Vec<SvcMessage>> {
    let payload = StaticChannelInput::try_from(payload)?;

    match payload {
      StaticChannelInput::Terminal(message) => match message {
        TerminalInput::StdIn(data) => {
          let _ = self.input_sender.try_send(data);

          Ok(vec![])
        }
        TerminalInput::Output => Ok(self.drain()),
      },
    }
  }
}

impl Drop for Desk0StaticChannel {
  fn drop(&mut self) {
    self.cancellation_token.cancel();

    if let Some(thread) = self.thread.take() {
      thread.abort();
    }
  }
}
