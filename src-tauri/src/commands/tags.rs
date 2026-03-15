//! Tauri commands for tag-related operations.

use crate::models::app_state::AppState;
use crate::models::response_types::NoteContentResponse;
use std::fs;
use tauri::State;

/// Adds a tag to the current note session and writes it to disk.
#[tauri::command]
pub async fn add_note_tag(
    tag: String,
    state: State<'_, AppState>,
) -> Result<NoteContentResponse, String> {
    let mut session = state.note_session.lock().unwrap();
    let tag_manager = state.tag_manager.lock().unwrap();
    tag_manager.add_tag_to_session(&mut session, tag);

    let path = session
        .path
        .clone()
        .ok_or_else(|| "No active note session".to_string())?;
    let full_content = session.get_full_content();
    fs::write(&path, full_content).map_err(|e| format!("Failed to save note: {}", e))?;

    let note_manager = state.note_manager.lock().unwrap();
    Ok(NoteContentResponse::from_session(
        &session,
        &note_manager,
        &tag_manager,
    ))
}

/// Returns all unique tags from all notes, sorted by usage frequency.
#[tauri::command]
pub async fn get_all_tags(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let folder_path = {
        let note_manager = state.note_manager.lock().unwrap();
        note_manager.notes_folder.clone()
    };
    let tag_manager = state.tag_manager.lock().unwrap();
    tag_manager.get_all_tags(&folder_path)
}
