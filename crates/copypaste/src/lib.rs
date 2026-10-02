pub mod cliprdr;

#[cfg(any(feature = "client", feature = "server"))]
pub use cliprdr::clipboard_watcher::ClipboardStatus;
#[cfg(any(feature = "client", feature = "server"))]
pub use cliprdr::cliprdr_handler::CliprdrHandler;
