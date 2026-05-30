//! Application configuration and persistent settings.

use crate::utils;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration for a keyboard shortcut.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutConfig {
    pub key: String,
    pub primary: bool,
    pub secondary: bool,
    pub shift: bool,
    pub description: String,
}

/// Persistent configuration for the todaynote application.
///
/// This struct is serialized to and deserialized from a `config.json` file
/// stored in the application's data directory.
#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    /// Root directory for note storage.
    pub notes_folder: PathBuf,
    /// Selected application language code.
    pub locale: String,
    /// Currently active UI theme.
    pub theme: String,
    /// Whether to restore window dimensions and layout across sessions.
    pub remember_app_layout: bool,
    /// Layout style for the notes list ("list" or "masonry").
    pub notes_list_layout: String,
    /// Whether to remember component settings (search mode, layout, etc.).
    pub remember_settings: bool,
    /// Last used search mode.
    pub search_mode: String,
    /// Last used fuzzy search setting.
    pub search_is_fuzzy: bool,
    /// Last selected tag in search.
    pub search_selected_tag: Option<String>,
    /// Width of the NoteControlCenter sidebar in pixels.
    pub control_center_width: f64,
    /// Custom default name for the initial thread in a new daily note.
    pub default_thread_name: Option<String>,
    /// Global keyboard shortcuts configuration.
    pub shortcuts: HashMap<String, ShortcutConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let home_dir = utils::app_data::get_home_dir();
        let notes_folder = home_dir.join("notes");

        let mut shortcuts = HashMap::new();
        shortcuts.insert(
            "toggleSearch".to_string(),
            ShortcutConfig {
                key: "k".to_string(),
                primary: true,
                secondary: false,
                shift: false,
                description: "Toggle search".to_string(),
            },
        );
        shortcuts.insert(
            "toggleNoteBrowser".to_string(),
            ShortcutConfig {
                key: "l".to_string(),
                primary: true,
                secondary: false,
                shift: false,
                description: "Toggle note browser".to_string(),
            },
        );
        shortcuts.insert(
            "toggleSettings".to_string(),
            ShortcutConfig {
                key: ",".to_string(),
                primary: true,
                secondary: false,
                shift: false,
                description: "Toggle settings".to_string(),
            },
        );
        shortcuts.insert(
            "toggleStatistics".to_string(),
            ShortcutConfig {
                key: "x".to_string(),
                primary: true,
                secondary: false,
                shift: false,
                description: "Toggle statistics".to_string(),
            },
        );
        shortcuts.insert(
            "toggleNoteBrowserLayout".to_string(),
            ShortcutConfig {
                key: "l".to_string(),
                primary: true,
                secondary: true,
                shift: false,
                description: "Toggle note browser layout".to_string(),
            },
        );
        shortcuts.insert(
            "manageTags".to_string(),
            ShortcutConfig {
                key: "t".to_string(),
                primary: true,
                secondary: false,
                shift: false,
                description: "Manage tags".to_string(),
            },
        );
        shortcuts.insert(
            "closePopup".to_string(),
            ShortcutConfig {
                key: "Escape".to_string(),
                primary: false,
                secondary: false,
                shift: false,
                description: "Close popup".to_string(),
            },
        );
        shortcuts.insert(
            "focusLastLine".to_string(),
            ShortcutConfig {
                key: "0".to_string(),
                primary: true,
                secondary: true,
                shift: false,
                description: "Focus last line".to_string(),
            },
        );
        shortcuts.insert(
            "jumpByNumber".to_string(),
            ShortcutConfig {
                key: "1,2,3,4,5,6,7,8,9,b,c,d,g,h,i,j,k,n,p,r".to_string(),
                primary: true,
                secondary: true,
                shift: false,
                description: "Jump to thread".to_string(),
            },
        );
        shortcuts.insert(
            "toggleFuzzy".to_string(),
            ShortcutConfig {
                key: "f".to_string(),
                primary: true,
                secondary: true,
                shift: false,
                description: "Toggle fuzzy search".to_string(),
            },
        );
        shortcuts.insert(
            "toggleSearchMode".to_string(),
            ShortcutConfig {
                key: "m".to_string(),
                primary: true,
                secondary: true,
                shift: false,
                description: "Toggle search mode".to_string(),
            },
        );
        shortcuts.insert(
            "navigateYesterday".to_string(),
            ShortcutConfig {
                key: "e".to_string(),
                primary: true,
                secondary: true,
                shift: false,
                description: "Go to yesterday's note".to_string(),
            },
        );
        shortcuts.insert(
            "navigateLastAvailable".to_string(),
            ShortcutConfig {
                key: "a".to_string(),
                primary: true,
                secondary: true,
                shift: false,
                description: "Go to last available note".to_string(),
            },
        );
        shortcuts.insert(
            "navigateToday".to_string(),
            ShortcutConfig {
                key: "o".to_string(),
                primary: true,
                secondary: true,
                shift: false,
                description: "Go to today's note".to_string(),
            },
        );

        Self {
            notes_folder,
            locale: "en".to_string(),
            theme: "blind-spot".to_string(),
            remember_app_layout: true,
            notes_list_layout: "list".to_string(),
            remember_settings: true,
            search_mode: "notes".to_string(),
            search_is_fuzzy: true,
            search_selected_tag: None,
            control_center_width: 22.0, // Default 22rem
            default_thread_name: None,
            shortcuts,
        }
    }
}

impl AppConfig {
    /// Returns the absolute path to the configuration file.
    pub fn get_config_path() -> PathBuf {
        utils::app_data::get_app_data_dir().join("config.json")
    }

    /// Loads the configuration from disk, creating a default one if it doesn't exist.
    pub fn load() -> Self {
        let config_path = Self::get_config_path();

        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(config_content) => match serde_json::from_str(&config_content) {
                    Ok(config) => config,
                    Err(_) => {
                        let config = AppConfig::default();
                        config.save();
                        config
                    }
                },
                Err(_) => {
                    let config = AppConfig::default();
                    config.save();
                    config
                }
            }
        } else {
            let config = AppConfig::default();
            config.save();
            config
        }
    }

    /// Persists the current configuration to disk as a JSON file.
    pub fn save(&self) {
        let config_path = Self::get_config_path();

        if let Some(parent) = config_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        if let Ok(config_content) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&config_path, config_content);
        }
    }
}
