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
    let config = {
        let config = state.config()?;
        // Clone config to release lock early
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
        remember_app_layout: config.remember_app_layout,
        notes_list_layout: config.notes_list_layout.clone(),
        remember_settings: config.remember_settings,
        search_mode: config.search_mode.clone(),
        search_is_fuzzy: config.search_is_fuzzy,
        search_selected_tag: config.search_selected_tag,
        sidebar_open: config.sidebar_open,
        control_center_width: config.control_center_width,
        default_thread_name: config.default_thread_name,
        use_default_thread_name: config.use_default_thread_name,
        identicon_style: config.identicon_style,
        thread_shortcuts_mode: config.thread_shortcuts_mode,
        date_format_style: config.date_format_style,
        text_copy_mode: config.text_copy_mode.clone(),
        floating_toolbar_enabled: config.floating_toolbar_enabled,
        font_family: config.font_family,
        use_custom_font: config.use_custom_font,
        shortcuts: config.shortcuts,
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
/// Creates a note header based on the current configuration.
pub fn create_note_header(config: &AppConfig, translations: &HashMap<String, String>) -> String {
    // When use_default_thread_name is false, return an empty header
    if !config.use_default_thread_name {
        return String::new();
    }
    // When use_default_thread_name is true, return the default thread name
    config.default_thread_name.clone().unwrap_or_else(|| {
        translations
            .get("note.header")
            .cloned()
            .unwrap_or_else(|| "Note".to_string())
    })
}

/// Attempts to load today's daily note and updates the application session.
fn load_today_note(
    state: &State<'_, AppState>,
    translations: &HashMap<String, String>,
) -> Result<(Option<String>, Option<NoteContentResponse>), String> {
    let note_manager = state.note_manager()?;
    let file_path = note_manager.get_today_note_path();
    let path_str = file_path.to_string_lossy().into_owned();

    let config = state.config()?;
    let note_header = create_note_header(&config, translations);

    if let Ok(created_path) = note_manager.create_todays_note(&note_header) {
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

///
/// Unit Tests
///
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::config::AppConfig;
    use tempfile::tempdir;

    ///
    /// create_note_header Tests
    ///
    #[test]
    fn test_create_note_header_branches() {
        let en = get_translations("en".to_string());

        // Disabled returns empty even when a name is set
        let mut config = AppConfig::default();
        config.use_default_thread_name = false;
        config.default_thread_name = Some("Journal".to_string());
        assert_eq!(
            create_note_header(&config, &en),
            "",
            "Should return empty when use_default_thread_name is false"
        );

        // Custom name takes precedence over translation
        let mut config = AppConfig::default();
        config.use_default_thread_name = true;
        config.default_thread_name = Some("Custom".to_string());
        assert_eq!(
            create_note_header(&config, &en),
            "Custom",
            "default_thread_name should take precedence over translations"
        );

        // Falls back to English 'note.header' translation
        let mut config = AppConfig::default();
        config.use_default_thread_name = true;
        config.default_thread_name = None;
        assert_eq!(
            create_note_header(&config, &en),
            "Notes",
            "Should fall back to the 'note.header' translation for English"
        );

        // German translation differs
        let de = get_translations("de".to_string());
        let mut config = AppConfig::default();
        config.use_default_thread_name = true;
        config.default_thread_name = None;
        config.locale = "de".to_string();
        let header_de = create_note_header(&config, &de);
        assert_ne!(header_de, "Notes", "German translation should differ");
        assert!(!header_de.is_empty(), "German fallback should be non-empty");

        // Falls back to literal "Note" when translation key is missing
        let empty: HashMap<String, String> = HashMap::new();
        assert_eq!(
            create_note_header(&config, &empty),
            "Note",
            "Should fall back to 'Note' when 'note.header' key is missing"
        );

        // Default config uses English translation
        let config = AppConfig::default();
        assert_eq!(
            create_note_header(&config, &en),
            "Notes",
            "Default config should fall back to English 'note.header'"
        );
    }

    ///
    /// resolve_notes_folder Tests
    ///
    #[test]
    fn test_resolve_notes_folder_variants() {
        // Existing folder resolves to its path
        let dir = tempdir().expect("Failed to create temp dir");
        let mut config = AppConfig::default();
        config.notes_folder = dir.path().to_path_buf();
        assert_eq!(
            resolve_notes_folder(&config),
            Some(dir.path().to_string_lossy().into_owned()),
            "Existing folder should resolve to Some"
        );

        // Missing but creatable folder is created and resolved
        let new_folder = dir.path().join("new_notes");
        assert!(!new_folder.exists());
        config.notes_folder = new_folder.clone();
        assert_eq!(
            resolve_notes_folder(&config),
            Some(new_folder.to_string_lossy().into_owned()),
            "Creatable folder should resolve to Some"
        );
        assert!(new_folder.exists(), "Folder should have been created");

        // Not creatable (parent is a file) returns None
        let blocker = dir.path().join("blocker_file");
        std::fs::write(&blocker, "x").expect("Failed to create blocker file");
        config.notes_folder = blocker.join("notes");
        assert!(
            resolve_notes_folder(&config).is_none(),
            "Should return None when the folder cannot be created"
        );
    }

    ///
    /// get_ui_metadata Tests
    ///
    #[test]
    fn test_get_ui_metadata_components() {
        let config = AppConfig::default();
        let (themes, locales, translations, theme_colors) = get_ui_metadata(&config);

        assert!(!themes.is_empty(), "Should return available themes");
        assert!(!locales.is_empty(), "Should return available locales");
        assert!(!translations.is_empty(), "Should return translations");
        assert!(!theme_colors.is_empty(), "Should return theme colors");

        // theme_colors matches a direct call to get_theme_colors
        assert_eq!(
            theme_colors,
            get_theme_colors(config.theme.clone()),
            "theme_colors should match get_theme_colors for the configured theme"
        );

        // Translations match the configured locale
        let mut de_config = AppConfig::default();
        de_config.locale = "de".to_string();
        let (_, _, de_translations, _) = get_ui_metadata(&de_config);
        assert_eq!(
            de_translations.get("locale.name"),
            Some(&"Deutsch".to_string()),
            "Translations should match the configured German locale"
        );
    }

    #[test]
    fn test_get_ui_metadata_locales_and_themes_have_ids_and_names() {
        let config = AppConfig::default();
        let (themes, locales, _, _) = get_ui_metadata(&config);

        let locale_ids: Vec<&str> = locales.iter().map(|l| l.id.as_str()).collect();
        assert!(locale_ids.contains(&"en"), "Locales should include English");
        assert!(locale_ids.contains(&"de"), "Locales should include German");
        assert!(
            locales.iter().all(|l| !l.name.is_empty()),
            "Every locale should have a non-empty name"
        );

        let theme_ids: Vec<&str> = themes.iter().map(|t| t.id.as_str()).collect();
        assert!(
            theme_ids.contains(&"lobby"),
            "Themes should include the default 'lobby' theme"
        );
        assert!(
            themes.iter().all(|t| !t.name.is_empty()),
            "Every theme should have a non-empty display name"
        );
    }

    ///
    /// Serialization Tests
    ///
    #[test]
    fn test_app_payload_serialization() {
        let payload = AppPayload {
            notes_folder: Some("/tmp/notes".to_string()),
            locale: "de".to_string(),
            theme: "dracula".to_string(),
            remember_app_layout: false,
            notes_list_layout: "masonry".to_string(),
            remember_settings: false,
            search_mode: "threads".to_string(),
            search_is_fuzzy: false,
            search_selected_tag: Some("rust".to_string()),
            sidebar_open: false,
            control_center_width: 30.0,
            default_thread_name: Some("Daily".to_string()),
            use_default_thread_name: false,
            identicon_style: "none".to_string(),
            thread_shortcuts_mode: "actions".to_string(),
            date_format_style: "narrow".to_string(),
            text_copy_mode: "plain".to_string(),
            floating_toolbar_enabled: false,
            shortcuts: HashMap::new(),
            font_family: Some("Fira Code".to_string()),
            use_custom_font: true,
            available_locales: vec![LocaleInfo {
                id: "en".to_string(),
                name: "English".to_string(),
            }],
            available_themes: vec![ThemeInfo {
                id: "lobby".to_string(),
                name: "Lobby".to_string(),
            }],
            translations: HashMap::new(),
            theme_colors: HashMap::new(),
            today_note_path: Some("/tmp/notes/today.md".to_string()),
            today_note_content: None,
            is_mac: false,
        };

        let json = serde_json::to_string(&payload).expect("Failed to serialize AppPayload");

        // camelCase field names
        assert!(json.contains("notesFolder"), "Should use camelCase");
        assert!(json.contains("rememberAppLayout"), "Should use camelCase");
        assert!(json.contains("searchIsFuzzy"), "Should use camelCase");
        assert!(json.contains("controlCenterWidth"), "Should use camelCase");
        assert!(json.contains("useDefaultThreadName"), "Should use camelCase");
        assert!(json.contains("dateFormatStyle"), "Should use camelCase");
        assert!(json.contains("floatingToolbarEnabled"), "Should use camelCase");
        assert!(json.contains("todayNotePath"), "Should use camelCase");
        assert!(json.contains("todayNoteContent"), "Should use camelCase");
        assert!(!json.contains("remember_app_layout"), "Should not use snake_case");

        // Roundtrip preserves all fields
        let deserialized: AppPayload =
            serde_json::from_str(&json).expect("Failed to deserialize AppPayload");
        assert_eq!(deserialized.notes_folder, payload.notes_folder);
        assert_eq!(deserialized.locale, "de");
        assert_eq!(deserialized.theme, "dracula");
        assert!(!deserialized.remember_app_layout);
        assert_eq!(deserialized.search_selected_tag, Some("rust".to_string()));
        assert_eq!(deserialized.control_center_width, 30.0);
        assert_eq!(deserialized.default_thread_name, Some("Daily".to_string()));
        assert!(!deserialized.use_default_thread_name);
        assert_eq!(deserialized.font_family, Some("Fira Code".to_string()));
        assert!(deserialized.use_custom_font);
        assert_eq!(deserialized.available_locales.len(), 1);
        assert_eq!(deserialized.available_themes[0].id, "lobby");
        assert_eq!(
            deserialized.today_note_path,
            Some("/tmp/notes/today.md".to_string())
        );
        assert!(!deserialized.is_mac);
    }

    #[test]
    fn test_locale_and_theme_info_serialization() {
        let locale = LocaleInfo {
            id: "en".to_string(),
            name: "English".to_string(),
        };
        let theme = ThemeInfo {
            id: "lobby".to_string(),
            name: "Lobby".to_string(),
        };

        let locale_json = serde_json::to_string(&locale).expect("Failed to serialize LocaleInfo");
        let theme_json = serde_json::to_string(&theme).expect("Failed to serialize ThemeInfo");

        let de_locale: LocaleInfo =
            serde_json::from_str(&locale_json).expect("Failed to deserialize LocaleInfo");
        let de_theme: ThemeInfo =
            serde_json::from_str(&theme_json).expect("Failed to deserialize ThemeInfo");

        assert_eq!(de_locale.id, "en");
        assert_eq!(de_locale.name, "English");
        assert_eq!(de_theme.id, "lobby");
        assert_eq!(de_theme.name, "Lobby");
    }

    ///
    /// Default Config Tests
    ///
    #[test]
    fn test_default_config_resolves_notes_folder() {
        // The default notes_folder points at home/notes which is creatable, so resolve
        // should succeed (creating it if needed). Clean up any directory we create.
        let config = AppConfig::default();
        let original = config.notes_folder.clone();
        let existed = original.exists();

        assert!(
            resolve_notes_folder(&config).is_some(),
            "Default folder should be resolvable"
        );

        if !existed && original.exists() {
            let _ = std::fs::remove_dir(&original);
        }
    }
}
