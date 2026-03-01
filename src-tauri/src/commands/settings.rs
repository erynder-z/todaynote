use crate::commands::setup;
use crate::models::config::AppConfig;
use crate::models::response_types::{ConfigResponse, InitialAppState};
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
