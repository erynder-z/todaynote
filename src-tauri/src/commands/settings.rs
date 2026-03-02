use crate::commands::setup;
use crate::models::config::AppConfig;
use crate::models::response_types::{ConfigResponse, InitialAppState};
use crate::AppState;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub async fn get_config() -> Result<ConfigResponse, String> {
    let config = AppConfig::load();
    Ok(ConfigResponse {
        notes_folder: config.notes_folder.to_string_lossy().into_owned(),
        locale: config.locale,
        theme: config.theme,
        remember_window_size: config.remember_window_size,
    })
}

#[tauri::command]
pub async fn set_remember_window_size(remember: bool) -> Result<(), String> {
    let mut config = AppConfig::load();
    config.remember_window_size = remember;
    config.save();
    Ok(())
}

#[tauri::command]
pub async fn set_notes_folder(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut config = AppConfig::load();
    let new_path = PathBuf::from(path);
    config.notes_folder = new_path.clone();
    config.save();

    let mut note_manager = state.note_manager.lock().unwrap();
    note_manager.update_config(new_path, config.locale);

    Ok(())
}

#[tauri::command]
pub async fn set_locale(locale: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut config = AppConfig::load();
    config.locale = locale.clone();
    config.save();

    let mut note_manager = state.note_manager.lock().unwrap();
    note_manager.update_config(config.notes_folder, locale);

    Ok(())
}

#[tauri::command]
pub async fn switch_notes_folder(
    path: String,
    state: State<'_, AppState>,
) -> Result<InitialAppState, String> {
    let mut config = AppConfig::load();
    let new_path = PathBuf::from(path);
    config.notes_folder = new_path.clone();
    config.save();

    {
        let mut note_manager = state.note_manager.lock().unwrap();
        note_manager.update_config(new_path, config.locale.clone());
    }

    Ok(setup::get_initial_state(config, state))
}
