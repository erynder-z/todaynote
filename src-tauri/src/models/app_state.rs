//! Application-wide state management for the Tauri backend.

use crate::models::config::AppConfig;
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
    /// Persistent application configuration.
    pub config: Mutex<AppConfig>,
}

impl AppState {
    /// Safely locks the configuration, returning a descriptive error if the lock is poisoned.
    pub fn config(&self) -> Result<MutexGuard<'_, AppConfig>, String> {
        self.config
            .lock()
            .map_err(|_| "Configuration is currently unavailable (poisoned lock)".to_string())
    }

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

///
/// Unit Tests
///
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;

    fn make_state() -> AppState {
        AppState {
            note_manager: Mutex::new(NoteManager::new(PathBuf::from("/tmp/notes"), "en".into())),
            tag_manager: Mutex::new(TagManager::new()),
            note_session: Mutex::new(NoteSession::new()),
            config: Mutex::new(AppConfig::default()),
        }
    }

    #[test]
    fn test_accessors_return_guard() {
        let state = make_state();

        assert_eq!(state.config().unwrap().locale, "en");
        assert_eq!(state.note_manager().unwrap().locale, "en");
        assert!(state.tag_manager().unwrap().cached_tags.is_none());
        assert!(state.note_session().unwrap().path.is_none());
    }

    #[test]
    fn test_accessors_poisoned_error() {
        let state = Arc::new(make_state());

        // Poison each mutex by panicking while holding the lock in a spawned thread.
        // The guard must still be live when the panic occurs.
        let poisons: &[fn(&Arc<AppState>)] = &[
            |s| {
                let _g = s.config.lock().unwrap();
                panic!("poison");
            },
            |s| {
                let _g = s.note_manager.lock().unwrap();
                panic!("poison");
            },
            |s| {
                let _g = s.tag_manager.lock().unwrap();
                panic!("poison");
            },
            |s| {
                let _g = s.note_session.lock().unwrap();
                panic!("poison");
            },
        ];
        for poison in poisons {
            let state = state.clone();
            let _ = std::thread::spawn(move || poison(&state)).join();
        }

        assert_eq!(
            state.config().err().unwrap(),
            "Configuration is currently unavailable (poisoned lock)"
        );
        assert_eq!(
            state.note_manager().err().unwrap(),
            "Note manager is currently unavailable (poisoned lock)"
        );
        assert_eq!(
            state.tag_manager().err().unwrap(),
            "Tag manager is currently unavailable (poisoned lock)"
        );
        assert_eq!(
            state.note_session().err().unwrap(),
            "Note session is currently unavailable (poisoned lock)"
        );
    }
}
