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
/// This function preserves frontmatter and ensures thread ID comments are maintained.
#[tauri::command]
pub async fn save_note_content(
    path: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<NoteContentResponse, String> {
    let path_buf = PathBuf::from(&path);
    
    // For save operations, we need to merge the frontend's content with the session's frontmatter
    let mut session = state.note_session()?;
    let full_content = if let Some((_, end)) = session.frontmatter_range {
        let frontmatter_lines = &session.lines[..=end];
        let frontmatter = frontmatter_lines.join("\n");
        format!("{}\n{}", frontmatter, content)
    } else {
        // No frontmatter in session, read from disk
        reconstruct_full_content(&path_buf, &content)?
    };

    // Update the active session with the new content
    session.load(path_buf.clone(), full_content.clone());

    fs::write(path_buf, session.get_full_content()).map_err(|e| format!("Failed to save note: {}", e))?;

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
        fs::write(path, &full_content).map_err(|e| format!("Failed to save note: {}", e))?;
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
        fs::write(path, &full_content).map_err(|e| format!("Failed to save note: {}", e))?;
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
        fs::write(path, &full_content).map_err(|e| format!("Failed to save note: {}", e))?;
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
/// If the note has legacy thread IDs, migrates them to UUID-based IDs and saves the updated content.
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

    // Reload the session with the latest content from disk to ensure we have
    // any changes made by previous operations (like thread ID comments from ensure_thread).
    // This is critical because the session might be stale.
    let full_content = reconstruct_full_content(&path, &current_content)?;
    session.load(path.clone(), full_content);

    operation(&mut session)?;

    let full_content = session.get_full_content();
    fs::write(&path, &full_content).map_err(|e| format!("Failed to save note: {}", e))?;

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
/// it is appended to the end of the note with a unique ID comment.
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
        use uuid::Uuid;
        
        if !session.threads.iter().any(|s| s.name == name) {
            let last_idx = session.lines.len();
            let thread_id = Uuid::new_v4().to_string();
            
            session.insert_line(last_idx, format!("# {}", name));
            session.insert_line(last_idx + 1, "".to_string());
            session.insert_line(last_idx + 2, "".to_string());
            
            // Create a new thread and add it to the list with proper metadata
            session.threads.push(NoteThread {
                id: thread_id,
                name: name.clone(),
                start_line: last_idx,
                end_line: session.lines.len(),
                level: 1,
            });
            
            // Update frontmatter with the new thread
            session.update_threads_in_frontmatter();
        }
        Ok(())
    })
    .await
}

/// Detects top-level headings (`# `) in markdown content and returns them as threads.
/// Uses thread IDs from frontmatter or generates new UUIDs.
/// If path is provided, loads the full content from disk to access frontmatter.
#[tauri::command]
pub async fn detect_threads(
    content: String,
    path: Option<String>,
) -> Result<Vec<NoteThread>, String> {
    // If we have a path, try to load the full content from disk
    // to access frontmatter with thread metadata
    if let Some(path_str) = path {
        let path_buf = PathBuf::from(path_str);
        if path_buf.exists() {
            if let Ok(file_content) = std::fs::read_to_string(&path_buf) {
                let mut session = NoteSession::new();
                session.load(path_buf, file_content);
                return Ok(session.threads);
            }
        }
    }
    
    // Fallback: detect threads from content only (no frontmatter)
    // This will generate new UUIDs, which may not match existing threads
    let mut session = NoteSession::new();
    session.load(PathBuf::new(), content);
    
    Ok(session.threads)
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

/// Removes a thread by ID from the current note session.
///
/// Removes the thread header, its ID comment (if present), and all content until the next thread.
/// The `current_content` parameter should be the content portion from the frontend
/// (excluding frontmatter). The backend reconstructs the full note by reading the
/// frontmatter from disk.
#[tauri::command]
pub async fn remove_thread(
    thread_id: String,
    current_content: String,
    state: State<'_, AppState>,
) -> Result<NoteContentResponse, String> {
    perform_thread_operation(current_content.clone(), state, |session| {
        // Reload session to ensure we have the latest content from disk
        let full_content = reconstruct_full_content(&session.path.clone().ok_or("No path".to_string())?, &current_content)?;
        session.load(session.path.clone().ok_or("No path".to_string())?, full_content);
        
        // Find the thread by ID from the threads list (which is populated from frontmatter)
        if let Some(thread_to_remove) = session.find_thread_by_id(&thread_id) {
            session.delete_line_range(thread_to_remove.start_line, thread_to_remove.end_line);
            Ok(())
        } else {
            // Debug: print all thread IDs
            let found_ids: Vec<String> = session.threads.iter().map(|t| t.id.clone()).collect();
            Err(format!("Thread '{}' not found. Available thread IDs: {:?}", thread_id, found_ids))
        }
    })
    .await
}
