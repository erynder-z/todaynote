use crate::utils;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub notes_folder: PathBuf,
    pub locale: String,
    pub theme: String,
    pub remember_window_size: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        let home_dir = utils::app_data::get_home_dir();
        let notes_folder = home_dir.join("notes");
        Self {
            notes_folder,
            locale: "en".to_string(),
            theme: "light".to_string(),
            remember_window_size: true,
        }
    }
}

impl AppConfig {
    pub fn get_config_path() -> PathBuf {
        utils::app_data::get_app_data_dir().join("config.json")
    }

    pub fn load() -> Self {
        let config_path = Self::get_config_path();

        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(config_content) => {
                    match serde_json::from_str(&config_content) {
                        Ok(config) => config,
                        Err(_) => {
                            // If parsing fails, create default config
                            let config = AppConfig::default();
                            config.save();
                            config
                        }
                    }
                }
                Err(_) => {
                    // If reading fails, create default config
                    let config = AppConfig::default();
                    config.save();
                    config
                }
            }
        } else {
            // Config doesn't exist, create default
            let config = AppConfig::default();
            config.save();
            config
        }
    }

    pub fn save(&self) {
        let config_path = Self::get_config_path();

        // Create parent directory if needed
        if let Some(parent) = config_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        // Serialize and save
        if let Ok(config_content) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&config_path, config_content);
        }
    }
}
