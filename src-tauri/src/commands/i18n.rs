use include_dir::{include_dir, Dir};
use std::collections::HashMap;

static TRANSLATIONS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/translations");

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
