use crate::commands::setup;
use crate::models::config::AppConfig;
use crate::models::response_types::{ConfigResponse, FolderValidation, InitialAppState};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[tauri::command]
pub async fn get_config() -> Result<ConfigResponse, String> {
    let config = AppConfig::load();
    Ok(ConfigResponse {
        notes_folder: config.notes_folder.to_string_lossy().into_owned(),
        locale: config.locale,
    })
}

#[tauri::command]
pub async fn set_notes_folder(path: String) -> Result<(), String> {
    let mut config = AppConfig::load();
    config.notes_folder = PathBuf::from(path);
    config.save();
    Ok(())
}

#[tauri::command]
pub async fn set_locale(locale: String) -> Result<(), String> {
    let mut config = AppConfig::load();
    config.locale = locale;
    config.save();
    Ok(())
}

#[tauri::command]
pub async fn switch_notes_folder(path: String) -> Result<InitialAppState, String> {
    let mut config = AppConfig::load();
    config.notes_folder = PathBuf::from(path);
    config.save();

    Ok(setup::get_initial_state(config))
}

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

#[tauri::command]
pub async fn validate_folder(path: String) -> Result<FolderValidation, String> {
    let path_buf = PathBuf::from(&path);
    let mut validation = FolderValidation {
        is_valid: true,
        is_writable: false,
        exists: path_buf.exists(),
        note_count: 0,
        error: None,
    };

    if validation.exists {
        if !path_buf.is_dir() {
            validation.is_valid = false;
            validation.error = Some("Path is not a directory".to_string());
            return Ok(validation);
        }

        let temp_file = path_buf.join(".todaynote_write_test");
        match fs::write(&temp_file, "test") {
            Ok(_) => {
                validation.is_writable = true;
                let _ = fs::remove_file(temp_file);
            }
            Err(e) => {
                validation.is_writable = false;
                validation.error = Some(format!("Directory is not writable: {}", e));
            }
        }

        if let Ok(entries) = fs::read_dir(&path_buf) {
            validation.note_count = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
                .count();
        }
    } else {
        match fs::create_dir_all(&path_buf) {
            Ok(_) => {
                validation.is_writable = true;

                let _ = fs::remove_dir(&path_buf);
            }
            Err(e) => {
                validation.is_writable = false;
                validation.is_valid = false;
                validation.error = Some(format!("Cannot create directory at this path: {}", e));
            }
        }
    }

    Ok(validation)
}
