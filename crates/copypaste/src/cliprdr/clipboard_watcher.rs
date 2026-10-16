use std::{
  sync::{Arc, Mutex},
  thread::JoinHandle,
};

use crate::cliprdr::types::clipboard_file::{ClipboardFile, ClipboardState};

pub struct ClipboardHandle {
  thread: Option<JoinHandle<()>>,
  #[cfg(not(feature = "ipc"))]
  shutdown: Option<clipboard_rs::WatcherShutdown>,
  #[cfg(feature = "ipc")]
  cancellation_token: tokio_util::sync::CancellationToken,
}

#[derive(Clone, Copy)]
pub enum ClipboardStatus {
  Locked,
  Copy,
  Ready,
  Paste,
  Initialize,
}

#[derive(Clone)]
pub struct ClipboardWatcherHandler {
  handle: Arc<Mutex<ClipboardHandle>>,
  on_change: Arc<Mutex<Box<dyn FnMut(&ClipboardState) + Send>>>,

  #[cfg(not(feature = "ipc"))]
  pub(crate) clipboard: Arc<clipboard_rs::ClipboardContext>,
  pub(crate) state: Arc<Mutex<ClipboardState>>,
  pub(crate) status: Arc<Mutex<ClipboardStatus>>,

  #[cfg(feature = "ipc")]
  ipc: Arc<ipc_pdu::Listener>,
  #[cfg(feature = "ipc")]
  inbound: Arc<
    Mutex<
      Option<tokio::sync::mpsc::UnboundedReceiver<ipc_pdu::outbound::IpcUnpackedClipboardEvent>>,
    >,
  >,
}

impl ClipboardStatus {
  pub fn allow_next_event(&self) -> bool {
    match self {
      Self::Ready | Self::Initialize => true,
      Self::Paste | Self::Copy | Self::Locked => false,
    }
  }
}

impl ClipboardWatcherHandler {
  #[cfg(feature = "ipc")]
  pub fn new<F>(
    ipc: Arc<ipc_pdu::Listener>,
    receiver: tokio::sync::mpsc::UnboundedReceiver<ipc_pdu::outbound::IpcUnpackedClipboardEvent>,
    on_format_list: F,
  ) -> Self
  where
    F: FnMut(&ClipboardState) + Send + 'static,
  {
    Self {
      ipc,
      inbound: Arc::new(Mutex::new(Some(receiver))),
      state: Arc::new(Mutex::new(ClipboardState::default())),
      on_change: Arc::new(Mutex::new(Box::new(on_format_list))),
      status: Arc::new(Mutex::new(ClipboardStatus::Initialize)),
      handle: Arc::new(Mutex::new(ClipboardHandle {
        thread: None,
        cancellation_token: tokio_util::sync::CancellationToken::new(),
      })),
    }
  }

  #[cfg(not(feature = "ipc"))]
  pub fn new<F>(on_format_list: F) -> Self
  where
    F: FnMut(&ClipboardState) + Send + 'static,
  {
    let clipboard =
      clipboard_rs::ClipboardContext::new().expect("creating clipboard instance failed");

    Self {
      clipboard: Arc::new(clipboard),
      state: Arc::new(Mutex::new(ClipboardState::default())),
      on_change: Arc::new(Mutex::new(Box::new(on_format_list))),
      status: Arc::new(Mutex::new(ClipboardStatus::Initialize)),
      handle: Arc::new(Mutex::new(ClipboardHandle {
        thread: None,
        shutdown: None,
      })),
    }
  }

  pub fn clear(&self) {
    let mut state = self.state.lock().unwrap();
    state.clear();
  }

  pub fn set_status(&self, value: ClipboardStatus) {
    let mut status = self.status.lock().unwrap();
    *status = value;
  }

  pub fn get_status(&self) -> ClipboardStatus {
    *self.status.lock().unwrap()
  }

  #[cfg(not(feature = "ipc"))]
  pub fn start(&self) {
    use clipboard_rs::ClipboardWatcher;

    let mut handle = self.handle.lock().unwrap();

    if handle.thread.is_some() {
      tracing::warn!("Clipboard watcher already started");
      return;
    }

    let Ok(mut watcher) = clipboard_rs::ClipboardWatcherContext::new() else {
      tracing::error!("Failed to create clipboard watcher context");
      return;
    };

    let shutdown = watcher.get_shutdown_channel();

    let thread = std::thread::spawn({
      let this = self.clone();

      move || {
        tracing::info!("Clipboard watcher started");
        watcher.add_handler(this);
        watcher.start_watch();
        tracing::info!("Clipboard watcher stopped");
      }
    });

    handle.thread = Some(thread);
    handle.shutdown = Some(shutdown);

    drop(handle);
  }
}

#[cfg(not(feature = "ipc"))]
impl clipboard_rs::ClipboardHandler for ClipboardWatcherHandler {
  fn on_clipboard_change(&mut self) {
    use clipboard_rs::{Clipboard, common::RustImage};
    use ironrdp_cliprdr_format::{bitmap::png_to_cf_dibv5, html::plain_html_to_cf_html};

    let mut status = self.status.lock().unwrap();

    if !status.allow_next_event() {
      *status = ClipboardStatus::Ready;
      return;
    }

    #[cfg(feature = "server")]
    if !matches!(*status, ClipboardStatus::Ready) {
      return;
    }

    drop(status);

    let mut state = self.state.lock().unwrap();

    state.clear();

    if let Ok(files) = self.clipboard.get_files() {
      state.files.extend(files.into_iter().map(|file| {
        let path = std::path::Path::new(&file);
        let name = path
          .file_name()
          .map(|name| name.to_string_lossy().into_owned());
        let metadata = path.symlink_metadata().ok();
        let size = metadata.as_ref().map(|metadata| metadata.len());
        let modified = metadata
          .as_ref()
          .and_then(|metadata| metadata.modified().ok());

        ClipboardFile {
          name,
          size,
          modified,
          path: path.to_path_buf(),
        }
      }));
    }

    if !state.files.is_empty() {
      let snapshot = state.clone();
      drop(state);
      let mut on_change = self.on_change.lock().unwrap();
      on_change(&snapshot);
      return;
    }

    if let Ok(image) = self.clipboard.get_image() {
      match image.to_png() {
        Ok(bytes) => {
          let bytes = bytes.get_bytes();
          match png_to_cf_dibv5(bytes) {
            Ok(bytes) => state.image = Some(bytes),
            Err(error) => tracing::error!(?error, "failed to convert png to cf dibv5"),
          }
        }
        Err(error) => {
          tracing::error!(?error, "Failed to encode image");
        }
      };
    }

    if let Ok(html) = self.clipboard.get_html() {
      state.html = Some(plain_html_to_cf_html(&html));
    }

    if let Ok(rtf) = self.clipboard.get_rich_text() {
      state.rich_text = Some(rtf);
    }

    if let Ok(text) = self.clipboard.get_text() {
      state.text = Some(text);
    }

    let snapshot = state.clone();
    drop(state);
    let mut on_change = self.on_change.lock().unwrap();
    on_change(&snapshot);
  }
}

#[cfg(feature = "ipc")]
impl ClipboardWatcherHandler {
  pub fn start(&self) {
    let mut handle = self.handle.lock().unwrap();

    if handle.thread.is_some() {
      tracing::warn!("Clipboard watcher already started");
      return;
    }

    let Some(mut receiver) = self.inbound.lock().unwrap().take() else {
      return;
    };

    let ipc = self.ipc.clone();
    let state = self.state.clone();
    let status = self.status.clone();
    let on_change = self.on_change.clone();
    let cancellation_token = handle.cancellation_token.clone();

    let thread = std::thread::spawn(move || {
      tracing::info!("IPC clipboard receiver started");

      loop {
        if cancellation_token.is_cancelled() {
          break;
        }

        let event = match receiver.blocking_recv() {
          Some(event) => event,
          None => {
            break;
          }
        };

        if cancellation_token.is_cancelled() {
          break;
        }

        match event {
          ipc_pdu::outbound::IpcUnpackedClipboardEvent::Snapshot(snapshot) => {
            let mut status = status.lock().unwrap();

            if !status.allow_next_event() {
              *status = ClipboardStatus::Ready;
              return;
            }

            #[cfg(feature = "server")]
            if !matches!(*status, ClipboardStatus::Ready) {
              return;
            }

            drop(status);

            let snapshot = {
              let mut state = state.lock().unwrap();
              Self::apply_snapshot(ipc.clone(), &mut state, snapshot);
              state.clone()
            };

            let mut callback = on_change.lock().unwrap();
            callback(&snapshot);
          }
        }
      }
      tracing::info!("IPC Clipboard receiver stopped");
    });

    handle.thread = Some(thread);
    drop(handle);
  }

  fn apply_snapshot(
    ipc: Arc<ipc_pdu::Listener>,
    state: &mut ClipboardState,
    snapshot: ipc_pdu::outbound::IpcUnpackedClipboardSnapshot,
  ) {
    state.clear();

    if let Some(files) = snapshot.files {
      state.files.extend(files.into_iter().map(|file| {
        let path = std::path::PathBuf::from(file);
        let name = path
          .file_name()
          .map(|name| name.to_string_lossy().into_owned());
        let metadata = path.symlink_metadata().ok();

        ClipboardFile {
          name,
          size: metadata.as_ref().map(|metadata| metadata.len()),
          modified: metadata
            .as_ref()
            .and_then(|metadata| metadata.modified().ok()),
          path,
        }
      }));
    }

    state.html = snapshot.html;
    state.text = snapshot.text;
    state.rich_text = snapshot.rich_text;

    if let Some(shared_buffer) = snapshot.image {
      match ipc.blocking_copy_from_shared_buffer(shared_buffer) {
        Ok(image) => {
          state.image = Some(image);
        }
        Err(error) => {
          tracing::error!(?error, "failed to get clipboard image from shared buffer");
        }
      }
    }
  }
}

impl Drop for ClipboardWatcherHandler {
  #[cfg(feature = "ipc")]
  fn drop(&mut self) {
    let thread = {
      let mut handle = self.handle.lock().unwrap();
      handle.cancellation_token.cancel();
      handle.thread.take()
    };

    if let Some(thread) = thread {
      if thread.thread().id() != std::thread::current().id() {
        let _ = thread.join();
      }
    }
  }

  #[cfg(not(feature = "ipc"))]
  fn drop(&mut self) {
    let (shutdown, thread) = {
      let mut handle = self.handle.lock().unwrap();
      (handle.shutdown.take(), handle.thread.take())
    };

    if let Some(shutdown) = shutdown {
      shutdown.stop();
    }

    if let Some(thread) = thread {
      if thread.thread().id() != std::thread::current().id() {
        let _ = thread.join();
      }
    }
  }
}
