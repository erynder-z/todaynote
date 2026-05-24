//! Tauri commands for application settings management.

use crate::commands::setup;
use crate::models::app_state::AppState;
use crate::models::config::AppConfig;
use crate::models::response_types::{AppPayload, ConfigResponse};
use std::path::PathBuf;
use tauri::State;

/// Updates the entire application configuration and handles side effects.
#[tauri::command]
pub async fn update_config(
    new_config: ConfigResponse,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let (folder_changed, locale_changed) = {
        let mut config = state.config()?;

        let folder_changed = config.notes_folder != PathBuf::from(&new_config.notes_folder);
        let locale_changed = config.locale != new_config.locale;

        config.notes_folder = PathBuf::from(new_config.notes_folder);
        config.locale = new_config.locale;
        config.theme = new_config.theme;
        config.remember_app_layout = new_config.remember_app_layout;
        config.notes_list_layout = new_config.notes_list_layout;
        config.remember_settings = new_config.remember_settings;
        config.search_mode = new_config.search_mode;
        config.search_is_fuzzy = new_config.search_is_fuzzy;
        config.search_selected_tag = new_config.search_selected_tag;
        config.control_center_width = new_config.control_center_width;
        config.default_thread_name = new_config.default_thread_name;
        config.shortcuts = new_config.shortcuts;

        config.save();
        (folder_changed, locale_changed)
    };

    if folder_changed || locale_changed {
        let config = state.config()?;
        let mut note_manager = state.note_manager()?;
        note_manager.update_config(config.notes_folder.clone(), config.locale.clone());
    }

    Ok(())
}

/// Sets whether the width of the NoteControlCenter sidebar.
#[tauri::command]
pub async fn set_control_center_width(
    width: f64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.config()?;
    config.control_center_width = width;
    config.save();
    Ok(())
}

/// Sets whether the application should remember component settings.
#[tauri::command]
pub async fn set_remember_settings(
    remember: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.config()?;
    config.remember_settings = remember;
    config.save();
    Ok(())
}

/// Sets the last used search mode.
#[tauri::command]
pub async fn set_search_mode(mode: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut config = state.config()?;
    config.search_mode = mode;
    config.save();
    Ok(())
}

/// Sets the last used fuzzy search setting.
#[tauri::command]
pub async fn set_search_is_fuzzy(is_fuzzy: bool, state: State<'_, AppState>) -> Result<(), String> {
    let mut config = state.config()?;
    config.search_is_fuzzy = is_fuzzy;
    config.save();
    Ok(())
}

/// Sets the last selected tag in search.
#[tauri::command]
pub async fn set_search_selected_tag(
    tag: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.config()?;
    config.search_selected_tag = tag;
    config.save();
    Ok(())
}

/// Sets the layout style for the notes list.
#[tauri::command]
pub async fn set_notes_list_layout(
    layout: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.config()?;
    config.notes_list_layout = layout;
    config.save();
    Ok(())
}

/// Sets whether the application should remember the app layout.
#[tauri::command]
pub async fn set_remember_app_layout(
    remember: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.config()?;
    config.remember_app_layout = remember;
    config.save();
    Ok(())
}

/// Resets the application configuration to its default values.
///
/// Critical settings like `notes_folder` and `locale` are preserved to avoid
/// breaking the user's basic setup.
#[tauri::command]
pub async fn reset_config_to_defaults(state: State<'_, AppState>) -> Result<(), String> {
    let mut config = state.config()?;
    let default_config = AppConfig::default();

    config.theme = default_config.theme;
    config.remember_app_layout = default_config.remember_app_layout;
    config.notes_list_layout = default_config.notes_list_layout;
    config.remember_settings = default_config.remember_settings;
    config.search_mode = default_config.search_mode;
    config.search_is_fuzzy = default_config.search_is_fuzzy;
    config.search_selected_tag = default_config.search_selected_tag;
    config.control_center_width = default_config.control_center_width;
    config.default_thread_name = default_config.default_thread_name;
    config.shortcuts = default_config.shortcuts;

    config.save();
    Ok(())
}

/// Updates the notes folder and re-initializes the note manager.
#[tauri::command]
pub async fn set_notes_folder(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let (notes_folder, locale) = {
        let mut config = state.config()?;
        let new_path = PathBuf::from(path);
        config.notes_folder = new_path.clone();
        config.save();
        (config.notes_folder.clone(), config.locale.clone())
    };

    let mut note_manager = state.note_manager()?;
    note_manager.update_config(notes_folder, locale);

    Ok(())
}

/// Updates the application locale and re-initializes the note manager.
#[tauri::command]
pub async fn set_locale(locale: String, state: State<'_, AppState>) -> Result<(), String> {
    let (notes_folder, locale) = {
        let mut config = state.config()?;
        config.locale = locale.clone();
        config.save();
        (config.notes_folder.clone(), config.locale.clone())
    };

    let mut note_manager = state.note_manager()?;
    note_manager.update_config(notes_folder, locale);

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
    let config = {
        let mut config = state.config()?;
        let new_path = PathBuf::from(path);
        config.notes_folder = new_path.clone();
        config.save();

        // We return a clone to avoid holding the lock during get_initial_state
        AppConfig {
            notes_folder: config.notes_folder.clone(),
            locale: config.locale.clone(),
            theme: config.theme.clone(),
            remember_app_layout: config.remember_app_layout,
            notes_list_layout: config.notes_list_layout.clone(),
            remember_settings: config.remember_settings,
            search_mode: config.search_mode.clone(),
            search_is_fuzzy: config.search_is_fuzzy,
            search_selected_tag: config.search_selected_tag.clone(),
            control_center_width: config.control_center_width,
            default_thread_name: config.default_thread_name.clone(),
            shortcuts: config.shortcuts.clone(),
        }
    };

    {
        let mut note_manager = state.note_manager()?;
        note_manager.update_config(config.notes_folder.clone(), config.locale.clone());
    }

    setup::get_initial_state(config, state)
}
