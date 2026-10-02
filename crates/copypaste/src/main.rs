use copypaste::{CliprdrHandler, cliprdr::clipboard_watcher::ClipboardWatcherHandler};
use ironrdp::server::ServerEventSender;
use tokio::sync::mpsc::unbounded_channel;

#[tokio::main]
pub async fn main() {
  tracing_subscriber::fmt()
    .with_env_filter(
      tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        tracing_subscriber::EnvFilter::new("info,desk0_agent=debug,copypaste=debug")
      }),
    )
    .with_target(true)
    .with_thread_names(true)
    .with_file(true)
    .with_line_number(true)
    .init();

  let (rx, mut tx) = unbounded_channel();
  let watcher = ClipboardWatcherHandler::new(|state| {});
  let mut handler = CliprdrHandler::new();
  handler.set_sender(rx);

  loop {
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            break;
        }
        result = tx.recv() => {
            if let Some(data) = result {
                tracing::debug!(?data, "data");
            }
        }
    }
  }
}
