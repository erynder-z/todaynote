//! Tauri commands for application settings management.

use crate::commands::setup;
use crate::models::app_state::AppState;
use crate::models::config::AppConfig;
use crate::models::response_types::{AppPayload, ConfigResponse};
use std::path::PathBuf;
use tauri::State;

/// Loads the current application configuration.
#[tauri::command]
pub async fn get_config() -> Result<ConfigResponse, String> {
    let config = AppConfig::load();
    Ok(ConfigResponse {
        notes_folder: config.notes_folder.to_string_lossy().into_owned(),
        locale: config.locale,
        theme: config.theme,
        remember_window_size: config.remember_window_size,
        notes_list_layout: config.notes_list_layout,
        remember_settings: config.remember_settings,
        search_mode: config.search_mode,
        search_is_fuzzy: config.search_is_fuzzy,
        search_selected_tag: config.search_selected_tag,
        control_center_width: config.control_center_width,
    })
}

/// Sets the width of the NoteControlCenter sidebar.
#[tauri::command]
pub async fn set_control_center_width(width: f64) -> Result<(), String> {
    let mut config = AppConfig::load();
    config.control_center_width = width;
    config.save();
    Ok(())
}

/// Sets whether the application should remember component settings.
#[tauri::command]
pub async fn set_remember_settings(remember: bool) -> Result<(), String> {
    let mut config = AppConfig::load();
    config.remember_settings = remember;
    config.save();
    Ok(())
}

/// Sets the last used search mode.
#[tauri::command]
pub async fn set_search_mode(mode: String) -> Result<(), String> {
    let mut config = AppConfig::load();
    config.search_mode = mode;
    config.save();
    Ok(())
}

/// Sets the last used fuzzy search setting.
#[tauri::command]
pub async fn set_search_is_fuzzy(is_fuzzy: bool) -> Result<(), String> {
    let mut config = AppConfig::load();
    config.search_is_fuzzy = is_fuzzy;
    config.save();
    Ok(())
}

/// Sets the last selected tag in search.
#[tauri::command]
pub async fn set_search_selected_tag(tag: Option<String>) -> Result<(), String> {
    let mut config = AppConfig::load();
    config.search_selected_tag = tag;
    config.save();
    Ok(())
}

/// Sets the layout style for the notes list.
#[tauri::command]
pub async fn set_notes_list_layout(layout: String) -> Result<(), String> {
    let mut config = AppConfig::load();
    config.notes_list_layout = layout;
    config.save();
    Ok(())
}

/// Sets whether the application should remember the window size.
#[tauri::command]
pub async fn set_remember_window_size(remember: bool) -> Result<(), String> {
    let mut config = AppConfig::load();
    config.remember_window_size = remember;
    config.save();
    Ok(())
}

/// Updates the notes folder and re-initializes the note manager.
#[tauri::command]
pub async fn set_notes_folder(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut config = AppConfig::load();
    let new_path = PathBuf::from(path);
    config.notes_folder = new_path.clone();
    config.save();

    let mut note_manager = state.note_manager()?;
    note_manager.update_config(new_path, config.locale);

    Ok(())
}

/// Updates the application locale and re-initializes the note manager.
#[tauri::command]
pub async fn set_locale(locale: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut config = AppConfig::load();
    config.locale = locale.clone();
    config.save();

    let mut note_manager = state.note_manager()?;
    note_manager.update_config(config.notes_folder, locale);

    Ok(())
}

/// Updates the notes folder and returns the newly initialized application state.
///
/// This is used when the frontend needs to completely refresh its state
/// after a major settings change.
#[tauri::command]
pub async fn switch_notes_folder(
    path: String,
    state: State<'_, AppState>,
) -> Result<AppPayload, String> {
    let mut config = AppConfig::load();
    let new_path = PathBuf::from(path);
    config.notes_folder = new_path.clone();
    config.save();

    {
        let mut note_manager = state.note_manager()?;
        note_manager.update_config(new_path, config.locale.clone());
    }

    setup::get_initial_state(config, state)
}
