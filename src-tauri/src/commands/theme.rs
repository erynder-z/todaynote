//! Tauri commands for theme management and visual customization.

use crate::models::app_state::AppState;
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
                .get_file("lobby.json")
                .and_then(|file| file.contents_utf8())
                .and_then(|contents| serde_json::from_str(contents).ok())
                .unwrap_or_default()
        })
}

/// Updates the current application theme.
#[tauri::command]
pub async fn set_theme(theme: String, state: State<'_, AppState>) -> Result<(), String> {
    let (notes_folder, locale) = {
        let mut config = state.config()?;
        config.theme = theme.clone();
        config.save();
        (config.notes_folder.clone(), config.locale.clone())
    };

    let mut note_manager = state.note_manager()?;
    note_manager.update_config(notes_folder, locale);

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

///
/// Unit Tests
///
#[cfg(test)]
mod tests {
    use super::*;

    ///
    /// get_theme_colors Tests
    ///
    #[test]
    fn test_get_theme_colors_known_theme() {
        let colors = get_theme_colors("lobby".to_string());

        assert!(!colors.is_empty(), "Known theme should return colors");
        assert!(
            colors.contains_key("--bg-base"),
            "Should contain --bg-base key"
        );
        assert!(
            colors.contains_key("--accent"),
            "Should contain --accent key"
        );
        assert!(
            colors["--bg-base"].starts_with('#'),
            "Color values should be hex strings"
        );
    }

    #[test]
    fn test_get_theme_colors_different_themes_differ() {
        let lobby = get_theme_colors("lobby".to_string());
        let dracula = get_theme_colors("dracula".to_string());

        assert_ne!(
            lobby["--bg-base"], dracula["--bg-base"],
            "Different themes should have different color values"
        );
    }

    #[test]
    fn test_get_theme_colors_unknown_theme_falls_back_to_lobby() {
        let unknown = get_theme_colors("nonexistent-theme".to_string());
        let lobby = get_theme_colors("lobby".to_string());

        assert_eq!(
            unknown, lobby,
            "Unknown theme should fall back to lobby colors"
        );
        assert!(!unknown.is_empty(), "Fallback should not be empty");
    }

    #[test]
    fn test_get_theme_colors_all_shipped_themes_are_valid() {
        for (id, _) in get_available_themes() {
            let colors = get_theme_colors(id.clone());
            assert!(
                !colors.is_empty(),
                "Theme '{}' should produce a non-empty color map",
                id
            );
            assert!(
                colors.contains_key("--bg-base"),
                "Theme '{}' should contain --bg-base",
                id
            );
            assert!(
                colors.contains_key("--text-main"),
                "Theme '{}' should contain --text-main",
                id
            );
        }
    }

    ///
    /// get_available_themes Tests
    ///
    #[test]
    fn test_get_available_themes_returns_entries() {
        let themes = get_available_themes();

        assert!(!themes.is_empty(), "Should return available themes");
        let ids: Vec<&str> = themes.iter().map(|(id, _)| id.as_str()).collect();
        assert!(
            ids.contains(&"lobby"),
            "Should include the default 'lobby' theme"
        );
        assert!(ids.contains(&"dracula"), "Should include 'dracula'");
        assert!(ids.contains(&"nord"), "Should include 'nord'");
    }

    #[test]
    fn test_get_available_themes_names_are_human_readable() {
        let themes = get_available_themes();
        let by_id: std::collections::HashMap<&str, &str> = themes
            .iter()
            .map(|(id, name)| (id.as_str(), name.as_str()))
            .collect();

        // Hyphenated ids are split into capitalized words
        assert_eq!(
            by_id.get("catppuccin-frappe"),
            Some(&"Catppuccin Frappe"),
            "Hyphenated id should be title-cased with spaces"
        );
        assert_eq!(
            by_id.get("void-heart"),
            Some(&"Void Heart"),
            "Hyphenated id should be title-cased with spaces"
        );
        // Single-word ids are just capitalized
        assert_eq!(
            by_id.get("lobby"),
            Some(&"Lobby"),
            "Single-word id should be title-cased"
        );
    }

    #[test]
    fn test_get_available_themes_names_are_non_empty() {
        let themes = get_available_themes();

        assert!(
            themes.iter().all(|(_, name)| !name.is_empty()),
            "Every theme should have a non-empty display name"
        );
        assert!(
            themes.iter().all(|(id, _)| !id.is_empty()),
            "Every theme should have a non-empty id"
        );
    }

    #[test]
    fn test_get_available_themes_ids_are_unique() {
        let themes = get_available_themes();
        let ids: Vec<&str> = themes.iter().map(|(id, _)| id.as_str()).collect();

        let mut sorted = ids.clone();
        sorted.sort();
        let mut deduped = sorted.clone();
        deduped.dedup();

        assert_eq!(sorted.len(), deduped.len(), "Theme ids should be unique");
    }
}
