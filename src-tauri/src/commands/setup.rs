//! Tauri commands for initial application setup and bootstrapping.

use crate::commands::i18n::{get_available_locales, get_translations};
use crate::commands::theme::{get_available_themes, get_theme_colors};
use crate::models::app_state::AppState;
use crate::models::config::AppConfig;
use crate::models::response_types::{AppPayload, LocaleInfo, NoteContentResponse, ThemeInfo};
use std::collections::HashMap;
use tauri::State;

/// Initializes the application and returns the complete initial state for the frontend.
#[tauri::command]
pub async fn initialize_app(state: State<'_, AppState>) -> Result<AppPayload, String> {
    let config = AppConfig::load();
    get_initial_state(config, state)
}

/// Helper function to construct the full initial application state.
pub fn get_initial_state(
    config: AppConfig,
    state: State<'_, AppState>,
) -> Result<AppPayload, String> {
    let notes_folder = resolve_notes_folder(&config);
    let (available_themes, available_locales, translations, theme_colors) =
        get_ui_metadata(&config);

    let mut response = AppPayload {
        notes_folder,
        locale: config.locale.clone(),
        theme: config.theme.clone(),
        remember_window_size: config.remember_window_size,
        available_locales,
        available_themes,
        translations,
        theme_colors,
        today_note_path: None,
        today_note_content: None,
        is_mac: cfg!(target_os = "macos"),
    };

    if response.notes_folder.is_some() {
        let (path, content) = load_today_note(&state, &response.translations)?;
        response.today_note_path = path;
        response.today_note_content = content;
    }

    Ok(response)
}

/// Ensures the notes folder exists and returns its path.
fn resolve_notes_folder(config: &AppConfig) -> Option<String> {
    if config.notes_folder.exists() || std::fs::create_dir_all(&config.notes_folder).is_ok() {
        Some(config.notes_folder.to_string_lossy().into_owned())
    } else {
        None
    }
}

/// Collects all UI-related configuration and metadata.
fn get_ui_metadata(
    config: &AppConfig,
) -> (
    Vec<ThemeInfo>,
    Vec<LocaleInfo>,
    HashMap<String, String>,
    HashMap<String, String>,
) {
    let translations = get_translations(config.locale.clone());
    let theme_colors = get_theme_colors(config.theme.clone());

    let available_themes = get_available_themes()
        .into_iter()
        .map(|(id, name)| ThemeInfo { id, name })
        .collect();

    let available_locales = get_available_locales()
        .into_iter()
        .map(|(id, name)| LocaleInfo { id, name })
        .collect();

    (
        available_themes,
        available_locales,
        translations,
        theme_colors,
    )
}

/// Attempts to load today's daily note and updates the application session.
fn load_today_note(
    state: &State<'_, AppState>,
    translations: &HashMap<String, String>,
) -> Result<(Option<String>, Option<NoteContentResponse>), String> {
    let note_manager = state.note_manager()?;
    let file_path = note_manager.get_today_note_path();
    let path_str = file_path.to_string_lossy().into_owned();

    let note_header = translations
        .get("note.header")
        .map(|s| s.as_str())
        .unwrap_or("Note");

    if let Ok(created_path) = note_manager.create_todays_note(note_header) {
        if let Ok(content) = note_manager.read_note_content(&created_path) {
            let mut session = state.note_session()?;
            session.load(created_path.clone(), content);

            let tag_manager = state.tag_manager()?;

            return Ok((
                Some(path_str),
                Some(NoteContentResponse::from_session(
                    &session,
                    &note_manager,
                    &tag_manager,
                )),
            ));
        }
    }

    Ok((Some(path_str), None))
}
