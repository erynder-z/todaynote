//! Tauri command handlers for frontend interaction.
//!
//! This module organizes all backend logic that is exposed
//! to the Svelte frontend via Tauri's `invoke` system.

pub mod backup;
pub mod folder;
pub mod font;
pub mod i18n;
pub mod images;
pub mod notes;
pub mod search;
pub mod settings;
pub mod setup;
pub mod tags;
pub mod theme;

/// Runs a blocking (file I/O- or CPU-heavy) operation on the async runtime's
/// blocking thread pool, so it never stalls Tauri's IPC worker threads.
///
/// The closure must own everything it needs (e.g. a `NoteManager` snapshot):
/// no mutex guards may be held inside it.
pub(crate) async fn run_blocking<F, T>(f: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String> + Send + 'static,
    T: Send + 'static,
{
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| format!("Background task failed: {}", e))?
}
