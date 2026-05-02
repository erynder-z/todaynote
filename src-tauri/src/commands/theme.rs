//! Tauri commands for theme management and visual customization.

use crate::models::app_state::AppState;
use crate::models::config::AppConfig;
use include_dir::{include_dir, Dir};
use std::collections::HashMap;
use tauri::State;

/// Directory containing JSON theme definition files.
static THEMES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/themes");

/// Returns the color mapping for a specific theme.
///
/// If the requested theme is not found, it defaults to the 'light' theme.
#[tauri::command]
pub fn get_theme_colors(theme: String) -> HashMap<String, String> {
    let filename = format!("{}.json", theme);

    THEMES_DIR
        .get_file(&filename)
        .and_then(|file| file.contents_utf8())
        .and_then(|contents| serde_json::from_str(contents).ok())
        .unwrap_or_else(|| {
            THEMES_DIR
                .get_file("blind-spot.json")
                .and_then(|file| file.contents_utf8())
                .and_then(|contents| serde_json::from_str(contents).ok())
                .unwrap_or_default()
        })
}

/// Updates the current application theme.
#[tauri::command]
pub async fn set_theme(theme: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut config = AppConfig::load();
    config.theme = theme.clone();
    config.save();

    let mut note_manager = state.note_manager()?;
    note_manager.update_config(config.notes_folder, config.locale);

    Ok(())
}

/// Returns a list of all available themes based on the JSON files in the themes directory.
pub fn get_available_themes() -> Vec<(String, String)> {
    THEMES_DIR
        .files()
        .filter_map(|file| {
            let id = file.path().file_stem()?.to_str()?.to_string();
            let name = id
                .split('-')
                .map(|s| {
                    let mut c = s.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");

            Some((id, name))
        })
        .collect()
}
