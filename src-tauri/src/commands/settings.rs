use crate::commands::setup;
use crate::models::config::AppConfig;
use crate::models::response_types::{ConfigResponse, InitialAppState};
use std::collections::HashMap;
use std::path::PathBuf;

#[tauri::command]
pub async fn get_config() -> Result<ConfigResponse, String> {
    let config = AppConfig::load();
    Ok(ConfigResponse {
        notes_folder: config.notes_folder.to_string_lossy().into_owned(),
        locale: config.locale,
    })
}

#[tauri::command]
pub async fn set_notes_folder(path: String) -> Result<(), String> {
    let mut config = AppConfig::load();
    config.notes_folder = PathBuf::from(path);
    config.save();
    Ok(())
}

#[tauri::command]
pub async fn set_locale(locale: String) -> Result<(), String> {
    let mut config = AppConfig::load();
    config.locale = locale;
    config.save();
    Ok(())
}

#[tauri::command]
pub async fn switch_notes_folder(path: String) -> Result<InitialAppState, String> {
    let mut config = AppConfig::load();
    config.notes_folder = PathBuf::from(path);
    config.save();

    Ok(setup::get_initial_state(config))
}

#[tauri::command]
pub fn get_translations(locale: String) -> HashMap<String, String> {
    match locale.as_str() {
        "de" => {
            serde_json::from_str(include_str!("../../translations/de.json")).unwrap_or_default()
        }
        "jp" => {
            serde_json::from_str(include_str!("../../translations/jp.json")).unwrap_or_default()
        }
        _ => serde_json::from_str(include_str!("../../translations/en.json")).unwrap_or_default(),
    }
}
