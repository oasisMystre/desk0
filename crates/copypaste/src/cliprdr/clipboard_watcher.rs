use std::{
  sync::{Arc, Mutex},
  thread::JoinHandle,
};

use clipboard_rs::{
  Clipboard, ClipboardContext, ClipboardHandler, ClipboardWatcher, ClipboardWatcherContext,
  WatcherShutdown, common::RustImage,
};
use ironrdp_cliprdr_format::{bitmap::png_to_cf_dibv5, html::plain_html_to_cf_html};

use crate::cliprdr::types::clipboard_file::{ClipboardFile, ClipboardState};

pub struct ClipboardHandle {
  thread: Option<JoinHandle<()>>,
  shutdown: Option<WatcherShutdown>,
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

  pub(crate) clipboard: Arc<ClipboardContext>,
  pub(crate) state: Arc<Mutex<ClipboardState>>,
  pub(crate) status: Arc<Mutex<ClipboardStatus>>,
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
  pub fn new<F>(on_format_list: F) -> Self
  where
    F: FnMut(&ClipboardState) + Send + 'static,
  {
    let clipboard = ClipboardContext::new().expect("creating clipboard instance failed");

    Self {
      handle: Arc::new(Mutex::new(ClipboardHandle {
        thread: None,
        shutdown: None,
      })),
      on_change: Arc::new(Mutex::new(Box::new(on_format_list))),
      clipboard: Arc::new(clipboard),
      state: Arc::new(Mutex::new(ClipboardState::default())),
      status: Arc::new(Mutex::new(ClipboardStatus::Initialize)),
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

  pub fn start(&self) {
    let mut handle = self.handle.lock().unwrap();

    if handle.thread.is_some() {
      tracing::warn!("Clipboard watcher already started");
      return;
    }

    let Ok(mut watcher) = ClipboardWatcherContext::new() else {
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

impl ClipboardHandler for ClipboardWatcherHandler {
  fn on_clipboard_change(&mut self) {
    tracing::debug!("on_clipboard_change");
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

impl Drop for ClipboardWatcherHandler {
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
