//! Tauri commands for note-related operations.
//!
//! This module provides functions for reading, writing, and manipulating note files, as well as managing the current note editing session.

use crate::models::app_state::AppState;
use crate::models::note_session::{NoteSession, NoteThread};
use crate::models::response_types::{AppStatistics, NoteContentResponse, NoteListResponse};
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
) -> Result<NoteContentResponse, String> {
    let path_buf = PathBuf::from(&path);
    let full_content = reconstruct_full_content(&path_buf, &content)?;

    // Update the active session so other commands (like tags) have the latest content
    let mut session = state.note_session()?;
    session.load(path_buf.clone(), full_content.clone());

    fs::write(path_buf, full_content).map_err(|e| format!("Failed to save note: {}", e))?;

    let note_manager = state.note_manager()?;
    let tag_manager = state.tag_manager()?;

    Ok(NoteContentResponse::from_session(
        &session,
        &*note_manager,
        &*tag_manager,
    ))
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

/// Returns the absolute path to the most recent note that is not today's note.
#[tauri::command]
pub async fn get_last_available_note_path(
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let note_manager = state.note_manager()?;
    let response = note_manager.list_notes(None)?;
    let today_filename = format!("{}.md", crate::utils::date::get_current_date());

    for note in response.notes {
        if note.filename != today_filename {
            let file_path = note_manager.notes_folder.join(note.filename);
            return Ok(Some(file_path.to_string_lossy().into_owned()));
        }
    }

    Ok(None)
}

/// Reads the content of the most recent note that is not today's note.
#[tauri::command]
pub async fn read_last_available_note(
    state: State<'_, AppState>,
) -> Result<Option<NoteContentResponse>, String> {
    if let Some(path) = get_last_available_note_path(state.clone()).await? {
        Ok(Some(read_note_content(path, state).await?))
    } else {
        Ok(None)
    }
}

/// Returns the absolute path to a note file offset from today.
///
/// - offset = 0: today
/// - offset = -1: yesterday
/// - offset = -7: one week ago
#[tauri::command]
pub async fn get_note_path_by_offset(
    offset: i32,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let note_manager = state.note_manager()?;
    note_manager.ensure_notes_folder_exists()?;
    let file_path = crate::utils::date::get_note_path_by_offset(&note_manager.notes_folder, offset);
    Ok(file_path.to_string_lossy().into_owned())
}

/// Reads the content of a note file offset from today.
#[tauri::command]
pub async fn read_note_by_offset(
    offset: i32,
    state: State<'_, AppState>,
) -> Result<NoteContentResponse, String> {
    let path = get_note_path_by_offset(offset, state.clone()).await?;
    read_note_content(path, state).await
}

/// Checks if today's daily note file already exists.
#[tauri::command]
pub async fn check_todays_note_exists(state: State<'_, AppState>) -> Result<bool, String> {
    let note_manager = state.note_manager()?;
    Ok(note_manager.todays_note_exists())
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
pub async fn list_notes(
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<NoteListResponse, String> {
    let note_manager = state.note_manager()?;
    note_manager.list_notes(limit)
}

/// Deletes all notes that have no content (only frontmatter and headings).
#[tauri::command]
pub async fn purge_empty_notes(state: State<'_, AppState>) -> Result<usize, String> {
    let note_manager = state.note_manager()?;
    note_manager.purge_empty_notes()
}

/// Gathers comprehensive statistics across all notes in the configured folder.
#[tauri::command]
pub async fn get_statistics(state: State<'_, AppState>) -> Result<AppStatistics, String> {
    let note_manager = state.note_manager()?;
    note_manager.get_statistics()
}

/// Helper to sync frontend content, perform a thread operation, and save the result.
async fn perform_thread_operation<F>(
    current_content: String,
    state: State<'_, AppState>,
    operation: F,
) -> Result<NoteContentResponse, String>
where
    F: FnOnce(&mut NoteSession) -> Result<(), String>,
{
    let mut session = state.note_session()?;
    let path = session
        .path
        .clone()
        .ok_or_else(|| "No active note session".to_string())?;

    let full_content = reconstruct_full_content(&path, &current_content)?;
    session.load(path.clone(), full_content);

    operation(&mut session)?;

    let full_content = session.get_full_content();
    fs::write(&path, full_content).map_err(|e| format!("Failed to save note: {}", e))?;

    let note_manager = state.note_manager()?;
    let tag_manager = state.tag_manager()?;

    Ok(NoteContentResponse::from_session(
        &session,
        &*note_manager,
        &*tag_manager,
    ))
}

/// Finds or creates a thread by name and returns its content-relative line index.
///
/// Jumps to the end of the thread (ready to type). If the thread does not exist,
/// it is appended to the end of the note.
///
/// The `current_content` parameter should be the content portion from the frontend
/// (excluding frontmatter). The backend reconstructs the full note by reading the
/// frontmatter from disk.
#[tauri::command]
pub async fn ensure_thread(
    name: String,
    current_content: String,
    state: State<'_, AppState>,
) -> Result<NoteContentResponse, String> {
    perform_thread_operation(current_content, state, |session| {
        if !session.threads.iter().any(|s| s.name == name) {
            let last_idx = session.lines.len();
            session.insert_line(last_idx, format!("# {}", name));
            session.insert_line(last_idx + 1, "".to_string());
            session.insert_line(last_idx + 2, "".to_string());
        }
        Ok(())
    })
    .await
}

/// Detects top-level headings (`# `) in markdown content and returns them as threads.
#[tauri::command]
pub async fn detect_threads(content: String) -> Result<Vec<NoteThread>, String> {
    let lines: Vec<&str> = content.split('\n').collect();
    let mut threads: Vec<NoteThread> = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("# ") && !line.starts_with("## ") {
            let name = line[2..].trim().to_string();
            if !name.is_empty() {
                // Update previous thread's end_line
                if let Some(prev) = threads.last_mut() {
                    prev.end_line = i;
                }

                threads.push(NoteThread {
                    name,
                    level: 1,
                    start_line: i,
                    end_line: lines.len(),
                });
            }
        }
    }

    Ok(threads)
}

/// Renames the first top-level heading in a markdown string if it differs from the new name.
fn rename_primary_thread(content: &str, new_name: &str) -> Option<String> {
    let (content_start, _) = extract_frontmatter(content);
    let lines: Vec<&str> = content.split('\n').collect();

    for i in content_start..lines.len() {
        if lines[i].starts_with("# ") {
            let current_name = lines[i][2..].trim();
            if current_name != new_name {
                let mut owned_lines: Vec<String> =
                    lines.into_iter().map(|s| s.to_string()).collect();
                owned_lines[i] = format!("# {}", new_name);
                return Some(owned_lines.join("\n"));
            }
            break; // Only rename the first thread
        }
    }
    None
}

/// Updates the active note session if the modified file is currently open.
fn update_session_if_active(
    state: &State<'_, AppState>,
    path: &PathBuf,
    new_content: String,
) -> Result<(), String> {
    let mut session = state.note_session()?;
    if session.path.as_ref() == Some(path) {
        session.load(path.clone(), new_content);
    }
    Ok(())
}

/// Renames the first top-level heading in all notes to the specified name.
///
/// This is used to batch-apply a new default thread name to existing notes.
#[tauri::command]
pub async fn apply_default_thread_name(
    new_name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let notes_folder = state.note_manager()?.notes_folder.clone();

    if !notes_folder.exists() {
        return Ok(());
    }

    let entries = fs::read_dir(&notes_folder)
        .map_err(|e| format!("Failed to read notes directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Error reading directory entry: {}", e))?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read file {:?}: {}", path, e))?;

            if let Some(new_content) = rename_primary_thread(&content, &new_name) {
                fs::write(&path, &new_content)
                    .map_err(|e| format!("Failed to write file {:?}: {}", path, e))?;

                update_session_if_active(&state, &path, new_content)?;
            }
        }
    }

    Ok(())
}

/// Removes a thread by name from the current note session.
///
/// The `current_content` parameter should be the content portion from the frontend
/// (excluding frontmatter). The backend reconstructs the full note by reading the
/// frontmatter from disk.
#[tauri::command]
pub async fn remove_thread(
    name: String,
    current_content: String,
    state: State<'_, AppState>,
) -> Result<NoteContentResponse, String> {
    perform_thread_operation(current_content, state, |session| {
        if let Some(thread_to_remove) = session.threads.iter().find(|t| t.name == name) {
            let start_line = thread_to_remove.start_line;
            let end_line = thread_to_remove.end_line;
            session.delete_line_range(start_line, end_line);
            Ok(())
        } else {
            Err(format!("Thread '{}' not found", name))
        }
    })
    .await
}
