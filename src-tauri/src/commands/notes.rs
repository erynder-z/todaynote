use crate::models::config::AppConfig;
use crate::models::response_types::{FormattedNote, SearchResult};
use chrono::{Locale, NaiveDate};
use std::fs;
use std::path::PathBuf;

#[tauri::command]
pub async fn search_notes(_query: String) -> Result<Vec<SearchResult>, String> {
    // TODO: Implement note search
    Ok(vec![])
}

fn ensure_notes_folder_exists(config: &AppConfig) -> Result<(), String> {
    if !config.notes_folder.exists() {
        fs::create_dir_all(&config.notes_folder)
            .map_err(|e| format!("Failed to create notes folder: {}", e))?;
    }
    Ok(())
}

fn get_todays_note_path_internal(config: &AppConfig, current_date: &str) -> PathBuf {
    let file_name = format!("{}.md", current_date);
    config.notes_folder.join(&file_name)
}

fn create_note(file_path: &PathBuf, note_content: &str) -> Result<(), String> {
    fs::write(file_path, note_content).map_err(|e| format!("Failed to create note: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn get_today_note_path() -> Result<String, String> {
    let config = AppConfig::load();
    let current_date = crate::utils::date::get_current_date();

    ensure_notes_folder_exists(&config)?;
    let file_path = get_todays_note_path_internal(&config, &current_date);
    Ok(file_path.to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn check_todays_note_exists() -> Result<bool, String> {
    let config = AppConfig::load();
    let current_date = crate::utils::date::get_current_date();

    ensure_notes_folder_exists(&config)?;
    let file_path = get_todays_note_path_internal(&config, &current_date);

    Ok(file_path.exists())
}

#[tauri::command]
pub async fn create_todays_note(path: String) -> Result<(), String> {
    let file_path = PathBuf::from(path);

    if file_path.exists() {
        return Ok(()); // return early if file already exists
    }

    let config = AppConfig::load();
    let translations = crate::commands::i18n::get_translations(config.locale);
    let note_header = translations
        .get("note.header")
        .map(|s| s.as_str())
        .unwrap_or("Note");

    let current_date = crate::utils::date::get_current_date();
    let note_content = format!("# {}: {}", note_header, current_date);
    create_note(&file_path, &note_content)?;

    Ok(())
}

#[tauri::command]
pub async fn read_note_content(path: String) -> Result<String, String> {
    fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read note content from {}: {}", path, e))
}

fn format_note_name(note_name: &str, locale_str: &str) -> String {
    let without_ext = note_name.replace(".md", "");

    if let Ok(date) = NaiveDate::parse_from_str(&without_ext, "%Y-%m-%d") {
        let locale = match locale_str {
            "de" => Locale::de_DE,
            "jp" => Locale::ja_JP,
            _ => Locale::en_US,
        };

        // Format based on locale preference
        match locale_str {
            "de" => format!("{}", date.format_localized("%A, %e. %B %Y", locale)),
            "jp" => format!("{}", date.format_localized("%Y年%m月%d日 (%A)", locale)),
            _ => format!("{}", date.format_localized("%A, %B %e, %Y", locale)),
        }
    } else {
        without_ext
    }
}

#[tauri::command]
pub async fn list_notes() -> Result<Vec<FormattedNote>, String> {
    let config = AppConfig::load();

    let entries = fs::read_dir(&config.notes_folder)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    let mut notes: Vec<FormattedNote> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let file_name = entry.file_name().into_string().ok()?;
            if file_name.ends_with(".md") && !file_name.starts_with(".") {
                Some(FormattedNote {
                    filename: file_name.clone(),
                    formatted_name: format_note_name(&file_name, &config.locale),
                })
            } else {
                None
            }
        })
        .collect();

    notes.sort_by(|a, b| a.filename.cmp(&b.filename));
    notes.reverse();

    Ok(notes)
}
