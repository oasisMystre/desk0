// Compatibility re-exports for the legacy implementation while the
// cliprdr module is being migrated. New code should import these from
// `crate::cliprdr::types` directly.
pub use crate::cliprdr::types::clipboard_file::{ClipboardFile, ClipboardState};
pub use crate::cliprdr::types::download_state::{DownloadState, DownloadTask};
