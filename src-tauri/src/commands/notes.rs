//! Tauri commands for note-related operations.
//!
//! This module provides functions for reading, writing, and manipulating note files, as well as managing the current note editing session.

use crate::models::app_state::AppState;
use crate::models::response_types::{FormattedNote, NoteContentResponse, SearchResult};
use std::fs;
use std::path::PathBuf;
use tauri::State;

/// Saves the complete content of a note to the specified path.
#[tauri::command]
pub async fn save_note_content(
    path: String,
    content: String,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    fs::write(PathBuf::from(path), content).map_err(|e| format!("Failed to save note: {}", e))
}

/// Updates the content of a specific line in the current note session.
///
/// This operation also writes the entire note to disk.
#[tauri::command]
pub async fn update_note_line(
    index: usize,
    content: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut session = state.note_session.lock().unwrap();
    session.update_content_line(index, content);
    if let Some(path) = &session.path {
        let full_content = session.get_full_content();
        fs::write(path, full_content).map_err(|e| format!("Failed to save note: {}", e))?;
    }
    Ok(())
}

/// Inserts a new line into the current note session at the specified index.
///
/// This operation also writes the entire note to disk.
#[tauri::command]
pub async fn insert_note_line(
    index: usize,
    content: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut session = state.note_session.lock().unwrap();
    session.insert_content_line(index, content);

    if let Some(path) = &session.path {
        let full_content = session.get_full_content();
        fs::write(path, full_content).map_err(|e| format!("Failed to save note: {}", e))?;
    }
    Ok(())
}

/// Deletes the line at the specified index from the current note session.
///
/// This operation also writes the entire note to disk.
#[tauri::command]
pub async fn delete_note_line(index: usize, state: State<'_, AppState>) -> Result<(), String> {
    let mut session = state.note_session.lock().unwrap();
    session.delete_content_line(index);

    if let Some(path) = &session.path {
        let full_content = session.get_full_content();
        fs::write(path, full_content).map_err(|e| format!("Failed to save note: {}", e))?;
    }
    Ok(())
}

/// Performs a full-text search across all notes. (Currently not implemented)
#[tauri::command]
pub async fn search_notes(_query: String) -> Result<Vec<SearchResult>, String> {
    // TODO: Implement note search
    Ok(vec![])
}

/// Returns the absolute path to today's daily note.
#[tauri::command]
pub async fn get_today_note_path(state: State<'_, AppState>) -> Result<String, String> {
    let note_manager = state.note_manager.lock().unwrap();
    note_manager.ensure_notes_folder_exists()?;
    let file_path = note_manager.get_today_note_path();
    Ok(file_path.to_string_lossy().into_owned())
}

/// Checks if today's daily note file already exists.
#[tauri::command]
pub async fn check_todays_note_exists(state: State<'_, AppState>) -> Result<bool, String> {
    let note_manager = state.note_manager.lock().unwrap();
    let file_path = note_manager.get_today_note_path();
    Ok(file_path.exists())
}

/// Creates a new daily note for today if it doesn't already exist.
///
/// Automatically initializes the note with a localized header.
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

    let created_path = note_manager.create_todays_note(note_header)?;

    // Load into session so auto-save works immediately
    if let Ok(content) = note_manager.read_note_content(&created_path) {
        let mut session = state.note_session.lock().unwrap();
        session.load(created_path, content);
    }

    Ok(())
}

/// Reads the content of a note file from the specified path.
///
/// If the note file does not exist, returns an error.
///
/// Automatically loads the note content into the application session.
///
/// Returns a `NoteContentResponse` containing the note's content and metadata.
#[tauri::command]
pub async fn read_note_content(
    path: String,
    state: State<'_, AppState>,
) -> Result<NoteContentResponse, String> {
    let path_buf = PathBuf::from(&path);
    let note_manager = state.note_manager.lock().unwrap();
    let content = note_manager.read_note_content(&path_buf)?;

    let mut session = state.note_session.lock().unwrap();
    session.load(path_buf.clone(), content);

    let tag_manager = state.tag_manager.lock().unwrap();

    Ok(NoteContentResponse::from_session(
        &session,
        &*note_manager,
        &*tag_manager,
    ))
}

/// Returns a list of all notes available in the current notes folder.
#[tauri::command]
pub async fn list_notes(state: State<'_, AppState>) -> Result<Vec<FormattedNote>, String> {
    let note_manager = state.note_manager.lock().unwrap();
    note_manager.list_notes()
}

/// Finds or creates a section by name and returns its content-relative line index.
///
/// If the section does not exist, it is appended to the end of the note.
#[tauri::command]
pub async fn jump_to_section(
    name: String,
    state: State<'_, AppState>,
) -> Result<NoteContentResponse, String> {
    let mut session = state.note_session.lock().unwrap();

    let section_idx = session.sections.iter().position(|s| s.name == name);
    let target_idx = match section_idx {
        Some(idx) => {
            // Section exists, jump to the end of this section
            let end_line = session.sections[idx].end_line;

            // If the line before the end is not empty, insert a new one unless the section is has no content
            if end_line > 0 {
                let last_content_line_idx = end_line - 1;
                if !session.lines[last_content_line_idx].trim().is_empty() {
                    session.insert_line(end_line, "".to_string());
                    end_line
                } else {
                    last_content_line_idx
                }
            } else {
                end_line
            }
        }
        None => {
            // Section doesn't exist, create it at the end with the [#] marker
            let last_idx = session.lines.len();
            session.insert_line(last_idx, format!("[#] {}", name));
            let new_line_idx = last_idx + 1;
            session.insert_line(new_line_idx, "".to_string());
            new_line_idx
        }
    };

    if let Some(path) = &session.path {
        let full_content = session.get_full_content();
        fs::write(path, full_content).map_err(|e| format!("Failed to save note: {}", e))?;
    }

    let note_manager = state.note_manager.lock().unwrap();
    let tag_manager = state.tag_manager.lock().unwrap();

    Ok(NoteContentResponse::from_session_with_target(
        &session,
        &note_manager,
        &tag_manager,
        Some(target_idx),
    ))
}
