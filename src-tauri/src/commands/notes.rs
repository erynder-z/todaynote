use crate::models::response_types::{FormattedNote, SearchResult};
use crate::AppState;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub async fn search_notes(_query: String) -> Result<Vec<SearchResult>, String> {
    // TODO: Implement note search
    Ok(vec![])
}

#[tauri::command]
pub async fn get_today_note_path(state: State<'_, AppState>) -> Result<String, String> {
    let note_manager = state.note_manager.lock().unwrap();
    note_manager.ensure_notes_folder_exists()?;
    let file_path = note_manager.get_today_note_path();
    Ok(file_path.to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn check_todays_note_exists(state: State<'_, AppState>) -> Result<bool, String> {
    let note_manager = state.note_manager.lock().unwrap();
    let file_path = note_manager.get_today_note_path();
    Ok(file_path.exists())
}

#[tauri::command]
pub async fn create_todays_note(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let file_path = PathBuf::from(path);

    if file_path.exists() {
        return Ok(());
    }

    let note_manager = state.note_manager.lock().unwrap();
    let translations = crate::commands::i18n::get_translations(note_manager.locale.clone());
    let note_header = translations
        .get("note.header")
        .map(|s| s.as_str())
        .unwrap_or("Note");

    note_manager.create_todays_note(note_header)?;

    Ok(())
}

#[tauri::command]
pub async fn read_note_content(path: String, state: State<'_, AppState>) -> Result<String, String> {
    let note_manager = state.note_manager.lock().unwrap();
    note_manager.read_note_content(&PathBuf::from(path))
}

#[tauri::command]
pub async fn list_notes(state: State<'_, AppState>) -> Result<Vec<FormattedNote>, String> {
    let note_manager = state.note_manager.lock().unwrap();
    note_manager.list_notes()
}
