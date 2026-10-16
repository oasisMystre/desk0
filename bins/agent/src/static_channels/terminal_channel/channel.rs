use std::{collections::HashMap, fmt::Debug};

use ironrdp::{
  core::{decode, impl_as_any},
  pdu::{PduError, PduErrorExt, PduResult},
  server::StaticChannelFactory,
  svc::{SvcMessage, SvcProcessor, SvcServerProcessor},
};
use pdu::terminal::{
  Mode, TERMINAL_STATIC_CHANNEL_NAME, TerminalError, TerminalInputEvent, TerminalOutputEvent,
};
use tokio::{
  sync::mpsc::{Sender, UnboundedReceiver, UnboundedSender, channel, unbounded_channel},
  task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use super::terminal::Terminal;

pub struct TerminalHandle {
  handles: Option<Vec<JoinHandle<()>>>,
  input_sender: Sender<TerminalInputEvent>,
}

struct TerminalStaticChannel {
  output_sender: UnboundedSender<TerminalOutputEvent>,
  output_receiver: UnboundedReceiver<TerminalOutputEvent>,

  terminals: HashMap<u32, TerminalHandle>,
  cancellation_token: CancellationToken,
}

#[derive(Debug)]
pub struct TerminalStaticChannelFactory;

impl Debug for TerminalStaticChannel {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("TerminalStaticChannel")
      .field("output_sender", &self.output_sender)
      .field("output_receiver", &self.output_receiver)
      .field("cancellation_token", &self.cancellation_token)
      .finish()
  }
}

impl TerminalHandle {
  pub fn abort(&mut self) {
    if let Some(handles) = self.handles.take() {
      for handle in handles {
        handle.abort();
      }
    }
  }
}

impl TerminalStaticChannel {
  const MAX_CHANNEL_SIZE: usize = 512;

  fn create(&mut self, id: u32) -> SvcMessage {
    if let Some(mut terminal) = self.terminals.remove(&id) {
      terminal.abort();
    }

    let (input_sender, mut input_receiver) = channel(Self::MAX_CHANNEL_SIZE);

    let terminal = match Terminal::spawn() {
      Ok(value) => value,
      Err(error) => {
        tracing::error!(?id, ?error, "failed to create terminal");
        return TerminalOutputEvent::Error {
          id,
          code: TerminalError::SpawnFailed,
          description: Some(error.to_string()),
        }
        .into();
      }
    };

    let mut handles = vec![];

    handles.push(tokio::spawn({
      let mut child = terminal.child;
      let mut writer = terminal.writer;
      let mut master = terminal.master;

      let output_sender = self.output_sender.clone();
      let cancellation_token = self.cancellation_token.clone();

      async move {
        let result = tokio::task::spawn_blocking(move || {
          loop {
            let input = match input_receiver.blocking_recv() {
              Some(input) => input,
              None => break,
            };

            match input {
              TerminalInputEvent::Data { data, .. } => {
                if let Err(error) = Terminal::write(&mut writer, &data) {
                  tracing::error!(?id, ?error, "failed to write to terminal");

                  let _ = output_sender.send(TerminalOutputEvent::Error {
                    id,
                    code: TerminalError::WriteFailed,
                    description: Some(error.to_string()),
                  });

                  continue;
                }
              }
              TerminalInputEvent::Resize {
                rows, cols, pw, ph, ..
              } => {
                if let Err(error) = Terminal::resize(&mut master, rows, cols, pw, ph) {
                  tracing::error!(?error, "failed to resize terminal");

                  let _ = output_sender.send(TerminalOutputEvent::Error {
                    id,
                    code: TerminalError::ResizeFailed,
                    description: Some(error.to_string()),
                  });

                  continue;
                }
              }
              _ => {}
            }
          }

          let code = match child.wait() {
            Ok(status) => status.exit_code(),
            Err(error) => {
              tracing::error!(?id, ?error, "failed to wait for terminal process");

              let _ = output_sender.send(TerminalOutputEvent::Error {
                id,
                code: TerminalError::ReadFailed,
                description: Some(error.to_string()),
              });

              1
            }
          };

          let _ = output_sender.send(TerminalOutputEvent::Exit {
            id,
            code: code as i32,
            description: None,
          });
        })
        .await;

        if let Err(error) = result {
          if !cancellation_token.is_cancelled() {
            tracing::error!(?id, ?error, "terminal writer task failed");
          }
        }
      }
    }));

    handles.push(tokio::spawn({
      let mut reader = terminal.reader;
      let output_sender = self.output_sender.clone();
      let cancellation_token = self.cancellation_token.clone();

      async move {
        let result = tokio::task::spawn_blocking(move || {
          loop {
            match Terminal::read(&mut reader) {
              Ok(Some(data)) => {
                if output_sender
                  .send(TerminalOutputEvent::Data { id, data })
                  .is_err()
                {
                  break;
                }
              }
              Ok(None) => {
                break;
              }
              Err(error) => {
                let _ = output_sender.send(TerminalOutputEvent::Error {
                  id,
                  code: TerminalError::ReadFailed,
                  description: Some(error.to_string()),
                });
                break;
              }
            }
          }
        })
        .await;

        if let Err(error) = result {
          if !cancellation_token.is_cancelled() {
            tracing::error!(?id, ?error, "terminal reader task failed");
          }
        }
      }
    }));

    handles.push(tokio::spawn({
      let input_sender = input_sender.clone();
      let cancellation_token = self.cancellation_token.clone();

      async move {
        cancellation_token.cancelled().await;

        drop(input_sender);
      }
    }));

    self.terminals.insert(
      id,
      TerminalHandle {
        input_sender,
        handles: Some(handles),
      },
    );

    TerminalOutputEvent::Connected(id).into()
  }

  fn drain(&mut self) -> Vec<SvcMessage> {
    let mut outputs: Vec<SvcMessage> = vec![];

    while let Ok(output) = self.output_receiver.try_recv() {
      outputs.push(output.into());
    }

    outputs
  }
}

impl_as_any!(TerminalStaticChannel);
impl SvcServerProcessor for TerminalStaticChannel {}

impl StaticChannelFactory for TerminalStaticChannelFactory {
  fn attach(&self, acceptor: &mut ironrdp::server::Acceptor) {
    let (output_sender, output_receiver) = unbounded_channel();

    let channel = TerminalStaticChannel {
      output_sender,
      output_receiver,
      terminals: HashMap::new(),
      cancellation_token: CancellationToken::new(),
    };

    acceptor.attach_static_channel(channel);
  }
}

impl SvcProcessor for TerminalStaticChannel {
  fn channel_name(&self) -> ironrdp::pdu::gcc::ChannelName {
    TERMINAL_STATIC_CHANNEL_NAME
  }

  fn process(&mut self, payload: &[u8]) -> PduResult<Vec<SvcMessage>> {
    let input = decode::<TerminalInputEvent>(payload)
      .map_err(|error| PduError::decode("terminal channel input", error))?;

    match input {
      TerminalInputEvent::Poll => Ok(self.drain()),
      TerminalInputEvent::Create(id) => Ok(vec![self.create(id)]),
      TerminalInputEvent::Exit(id) => {
        if let Some(mut terminal) = self.terminals.remove(&id) {
          terminal.abort();
        }

        Ok(vec![])
      }

      TerminalInputEvent::Data { id, mode, data } => {
        if matches!(mode, Mode::Sync) {
          return Ok(self.run_sync(id, &data));
        }

        self.send_input(id, TerminalInputEvent::Data { id, mode, data });

        Ok(vec![])
      }
      input @ TerminalInputEvent::Resize { id, .. } => {
        self.send_input(id, input);
        Ok(vec![])
      }
    }
  }
}

impl TerminalStaticChannel {
  fn send_input(&mut self, id: u32, input: TerminalInputEvent) {
    let Some(terminal) = self.terminals.get(&id) else {
      return;
    };

    if let Err(error) = terminal.input_sender.try_send(input) {
      tracing::warn!(?id, ?error, "failed to queue terminal input");
    }
  }

  fn run_sync(&self, id: u32, data: &[u8]) -> Vec<SvcMessage> {
    match Terminal::run(data) {
      Ok(output) => {
        let mut outputs: Vec<SvcMessage> = vec![];

        let mut data = output.stdout;
        data.extend_from_slice(&output.stderr);

        if !data.is_empty() {
          outputs.push(TerminalOutputEvent::Data { id, data }.into());
        }

        let code = output.status.code().unwrap_or(1);

        outputs.push(
          TerminalOutputEvent::Exit {
            id,
            code,
            description: None,
          }
          .into(),
        );

        outputs
      }
      Err(error) => vec![
        TerminalOutputEvent::Error {
          id,
          code: TerminalError::SpawnFailed,
          description: Some(error.to_string()),
        }
        .into(),
      ],
    }
  }
}

impl Drop for TerminalStaticChannel {
  fn drop(&mut self) {
    self.cancellation_token.cancel();

    for terminal in self.terminals.values_mut() {
      terminal.abort();
    }
  }
}
