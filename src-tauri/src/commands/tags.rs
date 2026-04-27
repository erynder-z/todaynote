//! Tauri commands for tag-related operations.

use crate::commands::notes::reconstruct_full_content;
use crate::models::app_state::AppState;
use crate::models::response_types::NoteContentResponse;
use std::fs;
use tauri::State;

/// Adds a tag to the current note session and writes it to disk.
#[tauri::command]
pub async fn add_note_tag(
    tag: String,
    current_content: String,
    state: State<'_, AppState>,
) -> Result<NoteContentResponse, String> {
    let mut session = state.note_session.lock().unwrap();

    let path = session
        .path
        .clone()
        .ok_or_else(|| "No active note session".to_string())?;

    // Sync session with frontend content first
    let full_content = reconstruct_full_content(&path, &current_content)?;
    session.load(path.clone(), full_content);

    let mut tag_manager = state.tag_manager.lock().unwrap();

    tag_manager.add_tag_to_session(&mut session, tag);
    tag_manager.invalidate_cache();

    let full_content = session.get_full_content();
    fs::write(&path, full_content).map_err(|e| format!("Failed to save note: {}", e))?;

    let note_manager = state.note_manager.lock().unwrap();
    Ok(NoteContentResponse::from_session(
        &session,
        &note_manager,
        &tag_manager,
    ))
}

/// Removes a tag from the current note session and writes it to disk.
#[tauri::command]
pub async fn remove_note_tag(
    tag: String,
    current_content: String,
    state: State<'_, AppState>,
) -> Result<NoteContentResponse, String> {
    let mut session = state.note_session.lock().unwrap();

    // Sync session with frontend content first
    let path = session.path.clone();
    if let Some(path) = path {
        let full_content = reconstruct_full_content(&path, &current_content)?;
        session.load(path, full_content);
    }

    let mut tag_manager = state.tag_manager.lock().unwrap();

    tag_manager.remove_tag_from_session(&mut session, tag);
    tag_manager.invalidate_cache();

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
    let mut tag_manager = state.tag_manager.lock().unwrap();
    tag_manager.get_all_tags(&folder_path)
}

/// Returns a filtered list of tag suggestions.
#[tauri::command]
pub async fn get_tag_suggestions(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let folder_path = {
        let note_manager = state.note_manager.lock().unwrap();
        note_manager.notes_folder.clone()
    };

    let session = state.note_session.lock().unwrap();
    let mut tag_manager = state.tag_manager.lock().unwrap();

    let active_tags = tag_manager.get_tags_from_session(&session);
    Ok(tag_manager.suggest_tags(&folder_path, &query, &active_tags, 20))
}
