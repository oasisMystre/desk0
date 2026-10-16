#[cfg(any(feature = "client", feature = "server"))]
pub mod clipboard_watcher;
#[cfg(any(feature = "client", feature = "server"))]
pub mod cliprdr_backend;
#[cfg(any(feature = "client", feature = "server"))]
pub mod cliprdr_handler;
pub mod types;

#[cfg(feature = "ipc")]
pub mod ipc;
