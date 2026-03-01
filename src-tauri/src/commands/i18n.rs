use std::collections::HashMap;

#[tauri::command]
pub fn get_translations(locale: String) -> HashMap<String, String> {
    match locale.as_str() {
        "de" => {
            serde_json::from_str(include_str!("../../translations/de.json")).unwrap_or_default()
        }
        "jp" => {
            serde_json::from_str(include_str!("../../translations/jp.json")).unwrap_or_default()
        }
        _ => serde_json::from_str(include_str!("../../translations/en.json")).unwrap_or_default(),
    }
}
