use std::sync::Arc;

use clipboard_rs::{
  Clipboard, ClipboardContext, ClipboardHandler, ClipboardWatcher, ClipboardWatcherContext,
  RustImageData, WatcherShutdown, common::RustImage,
};
use ipc::IpcSharedBufferField;
use ipc_pdu::{
  inbound::IpcUnpackedClipboardData,
  outbound::{IpcOutboundEvent, IpcPackedClipboardEvent, IpcPackedClipboardSnapshot},
};
use ironrdp_cliprdr_format::{bitmap::png_to_cf_dibv5, html::plain_html_to_cf_html};
use tokio::{sync::mpsc::UnboundedReceiver, task::JoinHandle};
use tokio_util::sync::CancellationToken;

pub struct IpcClipboardHandle<T, H> {
  thread: Option<H>,
  shutdown: T,
}

pub struct IpcClipboardWatcher {
  clipboard: Arc<ClipboardContext>,

  ipc: Arc<ipc_pdu::Connector>,
  inbound: Option<UnboundedReceiver<IpcUnpackedClipboardData>>,
  inbound_handle: IpcClipboardHandle<CancellationToken, JoinHandle<()>>,
  outbound_handle: IpcClipboardHandle<Option<WatcherShutdown>, std::thread::JoinHandle<()>>,
}

#[derive(Clone)]
pub struct IpcClipboardWatcherHandler {
  clipboard: Arc<ClipboardContext>,
  ipc: Arc<ipc_pdu::Connector>,
}

impl IpcClipboardWatcher {
  pub fn new(
    ipc: Arc<ipc_pdu::Connector>,
    inbound: UnboundedReceiver<IpcUnpackedClipboardData>,
  ) -> Self {
    let clipboard = ClipboardContext::new().expect("creating clipboard instance failed");

    Self {
      ipc,
      inbound: Some(inbound),
      clipboard: Arc::new(clipboard),
      outbound_handle: IpcClipboardHandle {
        thread: None,
        shutdown: None,
      },
      inbound_handle: IpcClipboardHandle {
        thread: None,
        shutdown: CancellationToken::new(),
      },
    }
  }

  pub fn start(&mut self) {
    if self.inbound_handle.thread.is_some() {
      tracing::warn!("Clipboard watcher already started");
      return;
    }

    let Ok(mut watcher) = ClipboardWatcherContext::new() else {
      tracing::error!("Failed to create clipboard watcher context");
      return;
    };

    let shutdown = watcher.get_shutdown_channel();

    let thread = std::thread::spawn({
      let ipc = self.ipc.clone();
      let clipboard = self.clipboard.clone();

      move || {
        tracing::info!("Clipboard watcher started");
        watcher.add_handler(IpcClipboardWatcherHandler { ipc, clipboard });
        watcher.start_watch();
        tracing::info!("Clipboard watcher stopped");
      }
    });

    self.outbound_handle.thread = Some(thread);
    self.outbound_handle.shutdown = Some(shutdown);

    let Some(mut receiver) = self.inbound.take() else {
      tracing::warn!("Ipc clipboard receiver already started");
      return;
    };

    let thread = tokio::spawn({
      let ipc = self.ipc.clone();
      let clipboard = self.clipboard.clone();
      let cancellation_token = self.inbound_handle.shutdown.clone();

      async move {
        loop {
          let event = tokio::select! {
              _ = cancellation_token.cancelled() => break,
              event = receiver.recv() => event
          };

          if cancellation_token.is_cancelled() {
            break;
          }

          let Some(event) = event else {
            break;
          };

          match event {
            IpcUnpackedClipboardData::Text(data) => {
              if let Err(error) = clipboard.set_text(data) {
                tracing::error!(?error, "Failed paste text");
              };
            }
            IpcUnpackedClipboardData::RichText(data) => {
              if let Err(error) = clipboard.set_rich_text(data) {
                tracing::error!(?error, "Failed to paste richtext");
              };
            }
            IpcUnpackedClipboardData::Html(data) => {
              if let Err(error) = clipboard.set_html(data) {
                tracing::error!(?error, "Failed to paste html");
              };
            }
            IpcUnpackedClipboardData::Image(data) => {
              match ipc.blocking_copy_from_shared_buffer(data) {
                Ok(data) => {
                  match image::load_from_memory_with_format(&data, image::ImageFormat::Png) {
                    Ok(image) => {
                      let image = RustImageData::from_dynamic_image(image);

                      if let Err(error) = clipboard.set_image(image) {
                        tracing::error!(?error, "Failed to paste image");
                      }
                    }
                    Err(error) => {
                      tracing::warn!(?error, "Failed to decode clipboard PNG");
                    }
                  }
                }
                Err(error) => {
                  tracing::error!(
                    ?error,
                    "getting clipboard paste image from shared buffer failed"
                  );
                }
              }
            }
            IpcUnpackedClipboardData::Files(files) => {
              if let Err(error) = clipboard.set_files(files) {
                tracing::error!(?error, "Failed to paste files");
              };
            }
          }
        }
      }
    });

    self.inbound_handle.thread = Some(thread);
  }

  pub async fn stop(&mut self) {
    self.inbound_handle.shutdown.cancel();
    self
      .outbound_handle
      .shutdown
      .take()
      .map(|shutdown| shutdown.stop());

    if let Some(thread) = self.inbound_handle.thread.take() {
      if let Err(error) = thread.await {
        if !error.is_cancelled() {
          tracing::error!(?error, "stopping outbound task failed");
        }
      }
    }

    if let Some(thread) = self.outbound_handle.thread.take() {
      if let Err(error) = thread.join() {
        tracing::error!(?error, "stopping outbound task failed");
      };
    }
  }
}

impl ClipboardHandler for IpcClipboardWatcherHandler {
  fn on_clipboard_change(&mut self) {
    let text = self.clipboard.get_text().ok();
    let files = self.clipboard.get_files().ok();
    let rich_text = self.clipboard.get_rich_text().ok();
    let html = self
      .clipboard
      .get_html()
      .ok()
      .map(|html| plain_html_to_cf_html(&html));
    let imagebytes = self.clipboard.get_image().ok().and_then(|image| {
      image
        .to_png()
        .ok()
        .and_then(|bytes| png_to_cf_dibv5(bytes.get_bytes()).ok())
    });

    let mut snapshot = IpcPackedClipboardSnapshot {
      files,
      image: None,
      html,
      text,
      rich_text,
    };

    let mut shared_buffers = vec![];

    if let Some(imagebytes) = imagebytes {
      snapshot.image = Some(IpcSharedBufferField(0));
      shared_buffers.push(imagebytes);
    }

    if shared_buffers.is_empty() {
      if let Err(error) = self.ipc.blocking_send(IpcOutboundEvent::Clipboard(
        IpcPackedClipboardEvent::Snapshot(snapshot),
      )) {
        tracing::error!(?error, "sending clipboard snapshot failed")
      };

      return;
    }

    if let Err(error) = self.ipc.blocking_send_with_shared_buffers(
      IpcOutboundEvent::Clipboard(IpcPackedClipboardEvent::Snapshot(snapshot)),
      &shared_buffers,
    ) {
      tracing::error!(?error, "sending clipboard snapshot failed")
    };
  }
}

impl Drop for IpcClipboardWatcher {
  fn drop(&mut self) {
    self.inbound_handle.shutdown.cancel();
    self
      .outbound_handle
      .shutdown
      .take()
      .map(|shutdown| shutdown.stop());

    if let Some(thread) = self.inbound_handle.thread.take() {
      thread.abort();
    }

    if let Some(thread) = self.outbound_handle.thread.take() {
      let _ = thread.join();
    }
  }
}
