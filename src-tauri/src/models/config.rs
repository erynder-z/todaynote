//! Application configuration and persistent settings.

use crate::utils;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    /// Whether to restore window dimensions across sessions.
    pub remember_window_size: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        let home_dir = utils::app_data::get_home_dir();
        let notes_folder = home_dir.join("notes");
        Self {
            notes_folder,
            locale: "en".to_string(),
            theme: "blind-spot".to_string(),
            remember_window_size: true,
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
