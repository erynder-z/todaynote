//! Tauri commands for note-related operations.
//!
//! This module provides functions for reading, writing, and manipulating note files, as well as managing the current note editing session.

use crate::models::app_state::AppState;
use crate::models::note_session::NoteSection;
use crate::models::response_types::{FormattedNote, NoteContentResponse};
use std::fs;
use std::path::PathBuf;
use tauri::State;

/// Extracts frontmatter from file content and returns (frontmatter_lines_count, frontmatter_string).
/// If no frontmatter exists, returns (0, String::new()).
fn extract_frontmatter(file_content: &str) -> (usize, String) {
    let lines: Vec<&str> = file_content.split('\n').collect();

    if lines.first().map(|l| l.trim()) != Some("---") {
        return (0, String::new());
    }

    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            let count = i + 1;
            let frontmatter = lines[..count].join("\n");
            return (count, frontmatter);
        }
    }

    (0, String::new())
}

/// Reconstructs full note content by prepending frontmatter from disk to the given content.
pub fn reconstruct_full_content(path: &PathBuf, content: &str) -> Result<String, String> {
    let file_content = fs::read_to_string(path).unwrap_or_default();
    let (content_start, frontmatter) = extract_frontmatter(&file_content);

    if content_start > 0 {
        Ok(format!("{}\n{}", frontmatter, content))
    } else {
        Ok(content.to_string())
    }
}

/// Saves the complete content of a note to the specified path.
///
/// The `content` parameter is the content portion (after frontmatter).
/// This function preserves frontmatter by reading it from disk and prepending it.
#[tauri::command]
pub async fn save_note_content(
    path: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);
    let full_content = reconstruct_full_content(&path_buf, &content)?;

    // Update the active session so other commands (like tags) have the latest content
    let mut session = state.note_session()?;
    session.load(path_buf.clone(), full_content.clone());

    fs::write(path_buf, full_content).map_err(|e| format!("Failed to save note: {}", e))
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
    let mut session = state.note_session()?;
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
    let mut session = state.note_session()?;
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
    let mut session = state.note_session()?;
    session.delete_content_line(index);

    if let Some(path) = &session.path {
        let full_content = session.get_full_content();
        fs::write(path, full_content).map_err(|e| format!("Failed to save note: {}", e))?;
    }
    Ok(())
}

/// Returns the absolute path to today's daily note.
#[tauri::command]
pub async fn get_today_note_path(state: State<'_, AppState>) -> Result<String, String> {
    let note_manager = state.note_manager()?;
    note_manager.ensure_notes_folder_exists()?;
    let file_path = note_manager.get_today_note_path();
    Ok(file_path.to_string_lossy().into_owned())
}

/// Checks if today's daily note file already exists.
#[tauri::command]
pub async fn check_todays_note_exists(state: State<'_, AppState>) -> Result<bool, String> {
    let note_manager = state.note_manager()?;
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

    let note_manager = state.note_manager()?;
    let translations = crate::commands::i18n::get_translations(note_manager.locale.clone());
    let note_header = translations
        .get("note.header")
        .map(|s| s.as_str())
        .unwrap_or("Note");

    let created_path = note_manager.create_todays_note(note_header)?;

    // Load into session so auto-save works immediately
    if let Ok(content) = note_manager.read_note_content(&created_path) {
        let mut session = state.note_session()?;
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
    let note_manager = state.note_manager()?;
    let content = note_manager.read_note_content(&path_buf)?;

    let mut session = state.note_session()?;
    session.load(path_buf.clone(), content);

    let tag_manager = state.tag_manager()?;

    Ok(NoteContentResponse::from_session(
        &session,
        &*note_manager,
        &*tag_manager,
    ))
}

/// Returns a list of all notes available in the current notes folder.
#[tauri::command]
pub async fn list_notes(state: State<'_, AppState>) -> Result<Vec<FormattedNote>, String> {
    let note_manager = state.note_manager()?;
    note_manager.list_notes()
}

/// Finds or creates a section by name and returns its content-relative line index.
///
/// Jumps to the end of the section (ready to type). If the section does not exist,
/// it is appended to the end of the note.
///
/// The `current_content` parameter should be the content portion from the frontend
/// (excluding frontmatter). The backend reconstructs the full note by reading the
/// frontmatter from disk.
#[tauri::command]
pub async fn ensure_section(
    name: String,
    current_content: String,
    state: State<'_, AppState>,
) -> Result<NoteContentResponse, String> {
    let mut session = state.note_session()?;
    let path = session.path.clone();

    if let Some(path) = &path {
        let full_content = reconstruct_full_content(path, &current_content)?;
        session.load(path.clone(), full_content);
    } else {
        return Err("No note session loaded".to_string());
    }

    if !session.sections.iter().any(|s| s.name == name) {
        let last_idx = session.lines.len();
        session.insert_line(last_idx, format!("# {}", name));
        session.insert_line(last_idx + 1, "".to_string());
        session.insert_line(last_idx + 2, "".to_string());
    }

    if let Some(path) = &session.path {
        let full_content = session.get_full_content();
        fs::write(path, full_content).map_err(|e| format!("Failed to save note: {}", e))?;
    }

    let note_manager = state.note_manager()?;
    let tag_manager = state.tag_manager()?;

    Ok(NoteContentResponse::from_session(
        &session,
        &note_manager,
        &tag_manager,
    ))
}

/// Detects top-level headings (`# `) in markdown content and returns them as sections.
#[tauri::command]
pub async fn detect_sections(content: String) -> Result<Vec<NoteSection>, String> {
    let lines: Vec<&str> = content.split('\n').collect();
    let mut sections: Vec<NoteSection> = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("# ") && !line.starts_with("## ") {
            let name = line[2..].trim().to_string();
            if !name.is_empty() {
                // Update previous section's end_line
                if let Some(prev) = sections.last_mut() {
                    prev.end_line = i;
                }

                sections.push(NoteSection {
                    name,
                    level: 1,
                    start_line: i,
                    end_line: lines.len(),
                });
            }
        }
    }

    Ok(sections)
}
