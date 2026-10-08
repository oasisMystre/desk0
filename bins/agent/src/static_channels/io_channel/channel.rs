use ironrdp::{
  core::{decode, impl_as_any},
  pdu::{PduError, PduErrorExt, PduResult, gcc::ChannelName},
  server::{Acceptor, StaticChannelFactory},
  svc::{SvcMessage, SvcProcessor, SvcServerProcessor},
};
use pdu::io::{IO_STATIC_CHANNEL_NAME, IoInputEvent, IoOutputEvent};
use tokio::{
  sync::mpsc::{Receiver, Sender, UnboundedReceiver, UnboundedSender, channel, unbounded_channel},
  task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use crate::static_channels::io_channel::download::Download;

pub struct IoStaticChannelFactory;

#[derive(Debug)]
struct IoStaticChannel {
  input_sender: Sender<IoInputEvent>,
  output_sender: UnboundedSender<IoOutputEvent>,
  output_receiver: UnboundedReceiver<IoOutputEvent>,

  handle: Option<JoinHandle<()>>,
  cancellation_token: CancellationToken,
}

impl_as_any!(IoStaticChannel);

impl IoStaticChannel {
  fn run(&mut self, mut input_receiver: Receiver<IoInputEvent>) {
    let mut download = Download::new(self.output_sender.clone());

    let handle = tokio::spawn({
      let cancellation_token = self.cancellation_token.clone();

      async move {
        loop {
          tokio::select! {
              _ = cancellation_token.cancelled() => {
                  break;
              }
              result = input_receiver.recv() => {
                  let Some(input) = result else { break;};
                  if let IoInputEvent::Download { id, uri, path } = input {
                      download.download(id, &uri, &path).await;
                  }
              }
          }
        }
      }
    });

    self.handle = Some(handle);
  }

  fn drain(&mut self) -> Vec<SvcMessage> {
    let mut outputs: Vec<SvcMessage> = vec![];

    while let Ok(output) = self.output_receiver.try_recv() {
      outputs.push(output.into());
    }

    outputs
  }
}

impl SvcServerProcessor for IoStaticChannel {}

impl StaticChannelFactory for IoStaticChannelFactory {
  fn attach(&self, acceptor: &mut Acceptor) {
    let (input_sender, input_receiver) = channel(64);
    let (output_sender, output_receiver) = unbounded_channel();

    let cancellation_token = CancellationToken::new();

    let mut channel = IoStaticChannel {
      input_sender,
      output_receiver,
      cancellation_token,
      handle: None,
      output_sender: output_sender.clone(),
    };

    channel.run(input_receiver);

    acceptor.attach_static_channel(channel);
  }
}

impl SvcProcessor for IoStaticChannel {
  fn channel_name(&self) -> ChannelName {
    IO_STATIC_CHANNEL_NAME
  }

  fn process(&mut self, payload: &[u8]) -> PduResult<Vec<SvcMessage>> {
    let input = decode::<IoInputEvent>(payload)
      .map_err(|error| PduError::decode("io channel input", error))?;

    match input {
      IoInputEvent::Poll => Ok(self.drain()),
      input @ IoInputEvent::Download { id, .. } => {
        self.send_input(id, input);
        Ok(vec![])
      }
    }
  }
}

impl IoStaticChannel {
  fn send_input(&mut self, id: u32, input: IoInputEvent) {
    if let Err(error) = self.input_sender.try_send(input) {
      tracing::warn!(?id, ?error, "failed to queue io input event");
    }
  }
}

impl Drop for IoStaticChannel {
  fn drop(&mut self) {
    self.cancellation_token.cancel();

    if let Some(handle) = self.handle.take() {
      handle.abort();
    }
  }
}
