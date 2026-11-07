//! Application-wide state management for the Tauri backend.

use crate::models::config::AppConfig;
use crate::models::note_session::NoteSession;
use crate::services::note_manager::NoteManager;
use crate::services::tag_manager::TagManager;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::Duration;
use tauri::Manager;

/// Global state accessible to all Tauri command handlers.
///
/// This struct is wrapped in a `tauri::State` and manages shared access
/// to services and transient session data via `Mutex` for thread safety.
///
/// # Global lock order
///
/// When more than one mutex is needed, guards must be acquired in this
/// order and never the reverse (doing so can deadlock the backend):
///
/// `config` → `note_manager` → `tag_manager` → `note_session`
///
/// Additionally, guards must be dropped before any blocking file I/O:
/// clone the needed values (paths, locale, session state), release the
/// guard, then perform the I/O lock-free.
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

    /// Returns an owned snapshot of the note manager (folder + locale).
    ///
    /// The lock is released before this returns, so callers can run
    /// folder-wide file I/O on the snapshot without holding the guard.
    pub fn note_manager_snapshot(&self) -> Result<NoteManager, String> {
        let note_manager = self.note_manager()?;
        Ok(NoteManager::new(
            note_manager.notes_folder.clone(),
            note_manager.locale.clone(),
        ))
    }
}

/// How long to wait after the last settings change before writing `config.json`.
const CONFIG_SAVE_DELAY: Duration = Duration::from_millis(500);

/// App handle used by the debounced config saver; set once during setup.
static CONFIG_APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();
/// Set when the in-memory config differs from the on-disk `config.json`.
static CONFIG_DIRTY: AtomicBool = AtomicBool::new(false);
/// Set while a debounced flush task is pending, so bursts spawn only one task.
static CONFIG_FLUSH_SCHEDULED: AtomicBool = AtomicBool::new(false);

/// Registers the app handle used by the debounced config saver. Call once in setup.
pub fn init_config_saver(app: &tauri::AppHandle) {
    let _ = CONFIG_APP_HANDLE.set(app.clone());
}

/// Marks the config as dirty and schedules a debounced write of `config.json`.
///
/// Bursts of settings changes (sidebar resize, toggles) collapse into a single
/// disk write. No-op when the saver was not initialized (e.g. in tests).
pub fn schedule_config_save() {
    let Some(handle) = CONFIG_APP_HANDLE.get() else {
        return;
    };
    CONFIG_DIRTY.store(true, Ordering::SeqCst);
    if CONFIG_FLUSH_SCHEDULED.swap(true, Ordering::SeqCst) {
        return;
    }
    let handle = handle.clone();
    // Debounce on a blocking thread so neither the sleep nor the write
    // occupies an async worker thread.
    tauri::async_runtime::spawn_blocking(move || {
        std::thread::sleep(CONFIG_SAVE_DELAY);
        flush_config_save(&handle);
    });
}

/// Writes `config.json` if the config is dirty. Also called on app exit.
pub fn flush_config_save(app: &tauri::AppHandle) {
    CONFIG_FLUSH_SCHEDULED.store(false, Ordering::SeqCst);
    if !CONFIG_DIRTY.swap(false, Ordering::SeqCst) {
        return;
    }
    // Clone and drop the guard before the blocking write.
    let snapshot = {
        let state = app.state::<AppState>();
        let snapshot = match state.config() {
            Ok(config) => config.clone(),
            Err(_) => return,
        };
        snapshot
    };
    snapshot.save();
}

/// Flushes a pending dirty config using the registered handle (app exit path).
pub fn flush_pending_config_save() {
    if let Some(handle) = CONFIG_APP_HANDLE.get() {
        flush_config_save(handle);
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

    #[test]
    fn test_config_saver_noop_without_handle() {
        // No handle is registered in tests; scheduling and flushing must not panic.
        schedule_config_save();
        flush_pending_config_save();
        assert!(!CONFIG_DIRTY.load(Ordering::SeqCst));
    }
}
