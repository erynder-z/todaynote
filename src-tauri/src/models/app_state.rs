//! Application-wide state management for the Tauri backend.

use crate::models::note_session::NoteSession;
use crate::services::note_manager::NoteManager;
use crate::services::tag_manager::TagManager;
use std::sync::{Mutex, MutexGuard};

/// Global state accessible to all Tauri command handlers.
///
/// This struct is wrapped in a `tauri::State` and manages shared access
/// to services and transient session data via `Mutex` for thread safety.
pub struct AppState {
    /// Service for managing note files and directories.
    pub note_manager: Mutex<NoteManager>,
    /// Service for managing tag aggregation.
    pub tag_manager: Mutex<TagManager>,
    /// Transient state for the currently active editing session.
    pub note_session: Mutex<NoteSession>,
}

impl AppState {
    /// Safely locks the note manager, returning a descriptive error if the lock is poisoned.
    pub fn note_manager(&self) -> Result<MutexGuard<'_, NoteManager>, String> {
        self.note_manager
            .lock()
            .map_err(|_| "Note manager is currently unavailable (poisoned lock)".to_string())
    }

    /// Safely locks the tag manager, returning a descriptive error if the lock is poisoned.
    pub fn tag_manager(&self) -> Result<MutexGuard<'_, TagManager>, String> {
        self.tag_manager
            .lock()
            .map_err(|_| "Tag manager is currently unavailable (poisoned lock)".to_string())
    }

    /// Safely locks the note session, returning a descriptive error if the lock is poisoned.
    pub fn note_session(&self) -> Result<MutexGuard<'_, NoteSession>, String> {
        self.note_session
            .lock()
            .map_err(|_| "Note session is currently unavailable (poisoned lock)".to_string())
    }
}
