//! Tauri commands for internationalization (i18n).

use include_dir::{include_dir, Dir};
use std::collections::HashMap;

/// Directory containing JSON translation files.
static TRANSLATIONS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/translations");

/// Returns the translation map for a specific locale.
///
/// If the requested locale is not found, it defaults to English ('en').
#[tauri::command]
pub fn get_translations(locale: String) -> HashMap<String, String> {
    let filename = format!("{}.json", locale);

    TRANSLATIONS_DIR
        .get_file(&filename)
        .and_then(|file| file.contents_utf8())
        .and_then(|contents| serde_json::from_str(contents).ok())
        .unwrap_or_else(|| {
            TRANSLATIONS_DIR
                .get_file("en.json")
                .and_then(|file| file.contents_utf8())
                .and_then(|contents| serde_json::from_str(contents).ok())
                .unwrap_or_default()
        })
}

/// Returns a list of all available locales based on the JSON files in the translations directory.
pub fn get_available_locales() -> Vec<(String, String)> {
    TRANSLATIONS_DIR
        .files()
        .filter_map(|file| {
            let id = file.path().file_stem()?.to_str()?.to_string();
            let contents = file.contents_utf8()?;
            let json: HashMap<String, String> = serde_json::from_str(contents).ok()?;
            let name = json
                .get("locale.name")
                .cloned()
                .unwrap_or_else(|| id.clone());

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
    /// get translations tests
    ///
    #[test]
    fn test_get_translations_english() {
        let translations = get_translations("en".to_string());
        assert!(!translations.is_empty());
        assert_eq!(
            translations.get("locale.name"),
            Some(&"English".to_string())
        );
    }

    #[test]
    fn test_get_translations_german() {
        let translations = get_translations("de".to_string());
        assert!(!translations.is_empty());
        assert_eq!(
            translations.get("locale.name"),
            Some(&"Deutsch".to_string())
        );
    }

    #[test]
    fn test_get_translations_japanese() {
        let translations = get_translations("ja".to_string());
        assert!(!translations.is_empty());
    }

    #[test]
    fn test_get_translations_falls_back_to_english() {
        let translations = get_translations("nonexistent".to_string());
        assert!(!translations.is_empty());
        assert_eq!(
            translations.get("locale.name"),
            Some(&"English".to_string())
        );
    }

    #[test]
    fn test_get_translations_contains_expected_keys() {
        let translations = get_translations("en".to_string());
        assert!(translations.contains_key("app.title"));
        assert!(translations.contains_key("navigation.note_list"));
    }

    ///
    /// get locales tests
    ///
    #[test]
    fn test_get_available_locales() {
        let locales = get_available_locales();
        assert!(!locales.is_empty());
    }

    #[test]
    fn test_get_available_locales_contains_all_translation_files() {
        let locales = get_available_locales();
        let ids: Vec<&str> = locales.iter().map(|(id, _)| id.as_str()).collect();
        assert!(ids.contains(&"en"));
        assert!(ids.contains(&"de"));
        assert!(ids.contains(&"ja"));
    }

    #[test]
    fn test_available_locale_names() {
        let locales = get_available_locales();
        let en = locales.iter().find(|(id, _)| *id == "en").unwrap();
        assert_eq!(en.1, "English");
        let de = locales.iter().find(|(id, _)| *id == "de").unwrap();
        assert_eq!(de.1, "Deutsch");
    }

    ///
    /// other tests
    ///
    #[test]
    fn test_translation_values_differ() {
        let en = get_translations("en".to_string());
        let de = get_translations("de".to_string());
        assert_ne!(en.get("app.title"), de.get("app.title"));
    }

    #[test]
    fn test_empty_locale_falls_back() {
        let translations = get_translations("".to_string());
        assert!(!translations.is_empty());
    }

    #[test]
    fn test_path_traversal_safe() {
        let translations = get_translations("../en".to_string());
        assert_eq!(
            translations.get("locale.name"),
            Some(&"English".to_string())
        );
    }
}
