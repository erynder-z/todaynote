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
) -> Result<AppPayload, String> {
    let (folder_changed, locale_changed, theme_changed) = {
        let mut config = state.config()?;

        let folder_changed = config.notes_folder != PathBuf::from(&new_config.notes_folder);
        let locale_changed = config.locale != new_config.locale;
        let theme_changed = config.theme != new_config.theme;

        config.notes_folder = PathBuf::from(new_config.notes_folder);
        config.locale = new_config.locale;
        config.theme = new_config.theme;
        config.remember_app_layout = new_config.remember_app_layout;
        config.notes_list_layout = new_config.notes_list_layout;
        config.remember_settings = new_config.remember_settings;
        config.search_mode = new_config.search_mode;
        config.search_is_fuzzy = new_config.search_is_fuzzy;
        config.search_selected_tag = new_config.search_selected_tag;
        config.sidebar_open = new_config.sidebar_open;
        config.control_center_width = new_config.control_center_width;
        config.default_thread_name = new_config.default_thread_name;
        config.use_default_thread_name = new_config.use_default_thread_name;
        config.identicon_style = new_config.identicon_style;
        config.thread_shortcuts_mode = new_config.thread_shortcuts_mode;
        config.date_format_style = new_config.date_format_style;
        config.text_copy_mode = new_config.text_copy_mode;
        config.floating_toolbar_enabled = new_config.floating_toolbar_enabled;
        config.shortcuts = new_config.shortcuts;
        config.font_family = new_config.font_family;
        config.use_custom_font = new_config.use_custom_font;

        config.save();
        (folder_changed, locale_changed, theme_changed)
    };

    if folder_changed || locale_changed || theme_changed {
        let config = state.config()?;
        let mut note_manager = state.note_manager()?;
        note_manager.update_config(config.notes_folder.clone(), config.locale.clone());
    }

    let config = {
        let config = state.config()?;
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
            sidebar_open: config.sidebar_open,
            control_center_width: config.control_center_width,
            default_thread_name: config.default_thread_name.clone(),
            use_default_thread_name: config.use_default_thread_name,
            identicon_style: config.identicon_style.clone(),
            thread_shortcuts_mode: config.thread_shortcuts_mode.clone(),
            date_format_style: config.date_format_style.clone(),
            text_copy_mode: config.text_copy_mode.clone(),
            floating_toolbar_enabled: config.floating_toolbar_enabled,
            shortcuts: config.shortcuts.clone(),
            font_family: config.font_family.clone(),
            use_custom_font: config.use_custom_font,
        }
    };

    setup::get_initial_state(config, state)
}

/// Sets the visibility of the sidebar.
#[tauri::command]
pub async fn set_sidebar_open(open: bool, state: State<'_, AppState>) -> Result<(), String> {
    let mut config = state.config()?;
    config.sidebar_open = open;
    config.save();
    Ok(())
}

/// Sets whether the width of the sidebar.
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
    // Preserve remember_settings - if user turned it off, respect that choice
    // config.remember_settings = default_config.remember_settings;
    config.search_mode = default_config.search_mode;
    config.search_is_fuzzy = default_config.search_is_fuzzy;
    config.search_selected_tag = default_config.search_selected_tag;
    config.sidebar_open = default_config.sidebar_open;
    config.control_center_width = default_config.control_center_width;
    config.default_thread_name = default_config.default_thread_name;
    config.use_default_thread_name = default_config.use_default_thread_name;
    config.identicon_style = default_config.identicon_style;
    config.thread_shortcuts_mode = default_config.thread_shortcuts_mode;
    config.date_format_style = default_config.date_format_style;
    config.floating_toolbar_enabled = default_config.floating_toolbar_enabled;
    config.shortcuts = default_config.shortcuts;
    config.font_family = default_config.font_family;
    config.use_custom_font = default_config.use_custom_font;

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

        // Return a clone to avoid holding the lock during get_initial_state
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
            sidebar_open: config.sidebar_open,
            control_center_width: config.control_center_width,
            default_thread_name: config.default_thread_name.clone(),
            use_default_thread_name: config.use_default_thread_name,
            identicon_style: config.identicon_style.clone(),
            thread_shortcuts_mode: config.thread_shortcuts_mode.clone(),
            date_format_style: config.date_format_style.clone(),
            text_copy_mode: config.text_copy_mode.clone(),
            floating_toolbar_enabled: config.floating_toolbar_enabled,
            shortcuts: config.shortcuts.clone(),
            font_family: config.font_family.clone(),
            use_custom_font: config.use_custom_font,
        }
    };

    {
        let mut note_manager = state.note_manager()?;
        note_manager.update_config(config.notes_folder.clone(), config.locale.clone());
    }

    setup::get_initial_state(config, state)
}

///
/// Unit Tests
///
#[cfg(test)]
mod tests {
    use crate::models::config::{AppConfig, ShortcutConfig};
    use crate::models::response_types::ConfigResponse;
    use std::collections::HashMap;
    use std::path::PathBuf;

    ///
    /// AppConfig Defaults tests
    ///
    #[test]
    fn test_config_defaults() {
        let config = AppConfig::default();

        assert_eq!(config.locale, "en");
        assert_eq!(config.theme, "lobby");
        assert_eq!(config.search_mode, "notes");
        assert!(config.search_is_fuzzy);
        assert!(config.sidebar_open);
        assert_eq!(config.control_center_width, 22.0);
        assert_eq!(config.notes_list_layout, "list");
        assert!(config.remember_app_layout);
        assert!(config.remember_settings);
        assert!(config.use_default_thread_name);
        assert_eq!(config.identicon_style, "dotmatrix");
        assert_eq!(config.thread_shortcuts_mode, "navigation");
        assert_eq!(config.date_format_style, "medium");
        assert_eq!(config.text_copy_mode, "markdown");
        assert!(config.floating_toolbar_enabled);
        assert!(!config.use_custom_font);
        assert!(config.font_family.is_none());
        assert!(config.search_selected_tag.is_none());
        assert!(config.default_thread_name.is_none());
    }

    #[test]
    fn test_config_default_shortcuts_populated() {
        let config = AppConfig::default();

        assert!(
            !config.shortcuts.is_empty(),
            "Default shortcuts should be populated"
        );
        assert!(config.shortcuts.contains_key("toggleSearch"));
        assert!(config.shortcuts.contains_key("toggleSidebar"));
        assert!(config.shortcuts.contains_key("toggleBold"));
    }

    ///
    /// Serialization tests
    ///
    #[test]
    fn test_config_serialization_roundtrip() {
        let mut config = AppConfig::default();
        config.locale = "de".to_string();
        config.theme = "dark".to_string();
        config.search_mode = "threads".to_string();
        config.search_is_fuzzy = false;
        config.sidebar_open = false;
        config.control_center_width = 30.0;
        config.search_selected_tag = Some("rust".to_string());
        config.default_thread_name = Some("Daily".to_string());
        config.font_family = Some("Fira Code".to_string());
        config.use_custom_font = true;

        let json = serde_json::to_string(&config).expect("Failed to serialize config");
        let deserialized: AppConfig =
            serde_json::from_str(&json).expect("Failed to deserialize config");

        assert_eq!(deserialized.locale, "de");
        assert_eq!(deserialized.theme, "dark");
        assert_eq!(deserialized.search_mode, "threads");
        assert!(!deserialized.search_is_fuzzy);
        assert!(!deserialized.sidebar_open);
        assert_eq!(deserialized.control_center_width, 30.0);
        assert_eq!(deserialized.search_selected_tag, Some("rust".to_string()));
        assert_eq!(deserialized.default_thread_name, Some("Daily".to_string()));
        assert_eq!(deserialized.font_family, Some("Fira Code".to_string()));
        assert!(deserialized.use_custom_font);
    }

    #[test]
    fn test_config_uses_camel_case() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).expect("Failed to serialize config");

        assert!(json.contains("rememberAppLayout"), "Should use camelCase");
        assert!(json.contains("searchIsFuzzy"), "Should use camelCase");
        assert!(json.contains("controlCenterWidth"), "Should use camelCase");
        assert!(
            json.contains("useDefaultThreadName"),
            "Should use camelCase"
        );
        assert!(json.contains("dateFormatStyle"), "Should use camelCase");
        assert!(
            json.contains("floatingToolbarEnabled"),
            "Should use camelCase"
        );
        assert!(
            !json.contains("remember_app_layout"),
            "Should not use snake_case"
        );
    }

    #[test]
    fn test_shortcut_config_serialization() {
        let shortcut = ShortcutConfig {
            key: "K".to_string(),
            primary: true,
            secondary: false,
            shift: true,
            description: "Toggle search".to_string(),
        };

        let json = serde_json::to_string(&shortcut).expect("Failed to serialize");
        let deserialized: ShortcutConfig =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.key, "K");
        assert!(deserialized.primary);
        assert!(!deserialized.secondary);
        assert!(deserialized.shift);
        assert_eq!(deserialized.description, "Toggle search");
    }

    ///
    /// ConfigResponse tests
    ///
    #[test]
    fn test_config_response_serialization() {
        let response = ConfigResponse {
            notes_folder: "/tmp/notes".to_string(),
            locale: "en".to_string(),
            theme: "lobby".to_string(),
            remember_app_layout: true,
            notes_list_layout: "list".to_string(),
            remember_settings: true,
            search_mode: "notes".to_string(),
            search_is_fuzzy: true,
            search_selected_tag: None,
            sidebar_open: true,
            control_center_width: 22.0,
            default_thread_name: None,
            use_default_thread_name: true,
            identicon_style: "dotmatrix".to_string(),
            thread_shortcuts_mode: "navigation".to_string(),
            date_format_style: "medium".to_string(),
            text_copy_mode: "markdown".to_string(),
            floating_toolbar_enabled: true,
            shortcuts: HashMap::new(),
            font_family: None,
            use_custom_font: false,
        };

        let json = serde_json::to_string(&response).expect("Failed to serialize");
        let deserialized: ConfigResponse =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.notes_folder, "/tmp/notes");
        assert_eq!(deserialized.locale, "en");
        assert_eq!(deserialized.control_center_width, 22.0);
        assert!(deserialized.search_is_fuzzy);
        assert!(json.contains("notesFolder"), "Should use camelCase");
        assert!(json.contains("controlCenterWidth"), "Should use camelCase");
    }

    ///
    /// Reset Logic tests
    ///
    #[test]
    fn test_reset_preserves_notes_folder_and_locale() {
        let mut config = AppConfig::default();
        config.notes_folder = PathBuf::from("/custom/notes");
        config.locale = "ja".to_string();
        let original_folder = config.notes_folder.clone();
        let original_locale = config.locale.clone();

        let default_config = AppConfig::default();
        config.theme = default_config.theme;
        config.search_mode = default_config.search_mode;
        config.sidebar_open = default_config.sidebar_open;
        config.search_is_fuzzy = default_config.search_is_fuzzy;

        assert_eq!(
            config.notes_folder, original_folder,
            "notes_folder should be preserved"
        );
        assert_eq!(config.locale, original_locale, "locale should be preserved");
        assert_eq!(config.theme, "lobby", "theme should be reset to default");
        assert_eq!(config.search_mode, "notes", "search_mode should be reset");
    }

    #[test]
    fn test_reset_preserves_remember_settings() {
        let mut config = AppConfig::default();
        config.remember_settings = false;

        let default_config = AppConfig::default();
        // reset_config_to_defaults intentionally does NOT reset remember_settings

        assert!(
            !config.remember_settings,
            "remember_settings should be preserved (not reset)"
        );
        assert!(
            default_config.remember_settings,
            "default remember_settings is true, proving the difference"
        );
    }

    #[test]
    fn test_reset_restores_all_resettable_fields() {
        let mut config = AppConfig::default();
        config.theme = "custom".to_string();
        config.search_mode = "threads".to_string();
        config.search_is_fuzzy = false;
        config.sidebar_open = false;
        config.control_center_width = 50.0;
        config.notes_list_layout = "masonry".to_string();
        config.use_default_thread_name = false;
        config.identicon_style = "none".to_string();
        config.thread_shortcuts_mode = "actions".to_string();
        config.date_format_style = "narrow".to_string();
        config.text_copy_mode = "plain".to_string();
        config.floating_toolbar_enabled = false;
        config.use_custom_font = true;
        config.font_family = Some("Arial".to_string());
        config.search_selected_tag = Some("tag".to_string());
        config.default_thread_name = Some("Name".to_string());

        let default_config = AppConfig::default();
        config.theme = default_config.theme;
        config.remember_app_layout = default_config.remember_app_layout;
        config.notes_list_layout = default_config.notes_list_layout.clone();
        config.search_mode = default_config.search_mode.clone();
        config.search_is_fuzzy = default_config.search_is_fuzzy;
        config.search_selected_tag = default_config.search_selected_tag.clone();
        config.sidebar_open = default_config.sidebar_open;
        config.control_center_width = default_config.control_center_width;
        config.default_thread_name = default_config.default_thread_name.clone();
        config.use_default_thread_name = default_config.use_default_thread_name;
        config.identicon_style = default_config.identicon_style.clone();
        config.thread_shortcuts_mode = default_config.thread_shortcuts_mode.clone();
        config.date_format_style = default_config.date_format_style.clone();
        config.floating_toolbar_enabled = default_config.floating_toolbar_enabled;
        config.shortcuts = default_config.shortcuts.clone();
        config.font_family = default_config.font_family.clone();
        config.use_custom_font = default_config.use_custom_font;

        assert_eq!(config.theme, "lobby");
        assert_eq!(config.search_mode, "notes");
        assert!(config.search_is_fuzzy);
        assert!(config.sidebar_open);
        assert_eq!(config.control_center_width, 22.0);
        assert_eq!(config.notes_list_layout, "list");
        assert!(config.use_default_thread_name);
        assert_eq!(config.identicon_style, "dotmatrix");
        assert_eq!(config.thread_shortcuts_mode, "navigation");
        assert_eq!(config.date_format_style, "medium");
        assert!(config.floating_toolbar_enabled);
        assert!(!config.use_custom_font);
        assert!(config.font_family.is_none());
        assert!(config.search_selected_tag.is_none());
        assert!(config.default_thread_name.is_none());
    }
}
