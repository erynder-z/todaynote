use crate::models::config::AppConfig;
use crate::AppState;
use include_dir::{include_dir, Dir};
use std::collections::HashMap;
use tauri::State;

static THEMES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/themes");

#[tauri::command]
pub fn get_theme_colors(theme: String) -> HashMap<String, String> {
    let filename = format!("{}.json", theme);

    THEMES_DIR
        .get_file(&filename)
        .and_then(|file| file.contents_utf8())
        .and_then(|contents| serde_json::from_str(contents).ok())
        .unwrap_or_else(|| {
            THEMES_DIR
                .get_file("light.json")
                .and_then(|file| file.contents_utf8())
                .and_then(|contents| serde_json::from_str(contents).ok())
                .unwrap_or_default()
        })
}

#[tauri::command]
pub async fn set_theme(theme: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut config = AppConfig::load();
    config.theme = theme.clone();
    config.save();

    let mut note_manager = state.note_manager.lock().unwrap();
    note_manager.update_config(config.notes_folder, config.locale);

    Ok(())
}

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
