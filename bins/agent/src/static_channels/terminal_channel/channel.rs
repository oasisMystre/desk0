use std::{collections::HashMap, fmt::Debug};

use ironrdp::{
  core::{decode, impl_as_any},
  pdu::{PduError, PduErrorExt, PduResult},
  server::StaticChannelFactory,
  svc::{SvcMessage, SvcProcessor, SvcServerProcessor},
};
use pdu::terminal::{TERMINAL_STATIC_CHANNEL_NAME, TerminalInputEvent, TerminalOutputEvent};
use tokio::{
  sync::mpsc::{Receiver, Sender, channel},
  task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use super::terminal::Terminal;

pub struct TerminalHandle {
  handles: Option<Vec<JoinHandle<()>>>,
  input_sender: Sender<TerminalInputEvent>,
}

struct TerminalStaticChannel {
  output_sender: Sender<TerminalOutputEvent>,
  output_receiver: Receiver<TerminalOutputEvent>,

  terminals: HashMap<u8, TerminalHandle>,
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
  fn create(&mut self, id: u8) -> SvcMessage {
    if let Some(mut terminal) = self.terminals.remove(&id) {
      terminal.abort();
    }

    let (sender, mut receiver) = channel(64);
    let output_sender = self.output_sender.clone();

    let terminal = match Terminal::spawn() {
      Ok(value) => value,
      Err(error) => {
        tracing::error!(?error, "failed to create terminal");
        return TerminalOutputEvent::Exit { id, code: 1 }.into();
      }
    };

    let mut handles = vec![];

    handles.push(tokio::spawn({
      let mut writer = terminal.writer;
      let mut master = terminal.master;

      let cancellation_token = self.cancellation_token.clone();

      async move {
        loop {
          tokio::select! {
              _ = cancellation_token.cancelled() => break,
              result = receiver.recv() => {
                  let Some(input) = result else {
                     break;
                  };

                  match input {
                    TerminalInputEvent::Data { data, .. } => {
                        if let Err(error) = Terminal::write(&mut writer, &data).await  {
                            tracing::error!(?error, "failed to execute terminal command");
                            continue;
                        }
                    },
                    TerminalInputEvent::Resize { rows, cols, pw, ph, .. } => {
                        if let Err(error) = Terminal::resize(&mut master, rows, cols, pw, ph)  {
                            tracing::error!(?error, "failed to resize terminal");
                            continue;
                        }
                    },
                    _ => {}
                }
             }
          }
        }
      }
    }));

    handles.push(tokio::spawn({
      let mut reader = terminal.reader;
      let cancellation_token = self.cancellation_token.clone();

      async move {
        loop {
          tokio::select! {
              _ = cancellation_token.cancelled() => break,
              result = Terminal::read(&mut reader) => {
                  match result {
                    Ok(Some(data)) => {
                        if output_sender.send(TerminalOutputEvent::Data{id, data}).await.is_err(){
                            break;
                        }
                    },
                    Ok(None)| Err(_) => {
                        let _ = output_sender.send(TerminalOutputEvent::Exit { id, code: 0 }).await;
                        break;
                    },
                  }

             }
          }
        }
      }
    }));

    self.terminals.insert(
      id,
      TerminalHandle {
        handles: Some(handles),
        input_sender: sender,
      },
    );

    TerminalOutputEvent::Connected(id).into()
  }

  pub fn drain(&mut self) -> Vec<SvcMessage> {
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
    let (output_sender, output_receiver) = channel(64);

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

    if let TerminalInputEvent::Output = input {
      return Ok(self.drain());
    }

    if let TerminalInputEvent::Create(id) = input {
      return Ok(vec![self.create(id)]);
    }

    let id = match input {
      TerminalInputEvent::Data { id, .. } => id,
      TerminalInputEvent::Resize { id, .. } => id,
      _ => return Ok(vec![]),
    };
    let Some(terminal) = self.terminals.get(&id) else {
      return Ok(vec![]);
    };

    if let Err(error) = terminal.input_sender.try_send(input) {
      tracing::warn!(?error, "failed to queue terminal input");
    }

    Ok(vec![])
  }
}

impl Drop for TerminalStaticChannel {
  fn drop(&mut self) {
    self.cancellation_token.cancel();

    for terminal in self.terminals.values_mut() {
      if let Some(handles) = terminal.handles.take() {
        for handle in handles {
          handle.abort();
        }
      }
    }
  }
}
