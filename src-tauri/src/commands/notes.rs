//! Tauri commands for note-related operations.
//!
//! This module provides functions for reading, writing, and manipulating note files, as well as managing the current note editing session.

use crate::models::app_state::AppState;
use crate::models::note_session::{NoteSession, NoteThread};
use crate::models::response_types::{
    AppStatistics, DayBoundaryStatus, NoteContentResponse, NoteListResponse,
};
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
    session.update_last_modified();

    fs::write(path_buf, session.get_full_content())
        .map_err(|e| format!("Failed to save note: {}", e))?;

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
    session.update_last_modified();
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
    session.update_last_modified();

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
    session.update_last_modified();

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

/// Checks whether the newest note corresponds to an older date than today.
/// This detects when a new day has started and there are no notes for today yet.
#[tauri::command]
pub async fn check_day_boundary(state: State<'_, AppState>) -> Result<DayBoundaryStatus, String> {
    let current_date = crate::utils::date::get_current_date();
    let note_manager = state.note_manager()?;

    let sorted_files = note_manager.get_sorted_note_files()?;

    let newest_note_date = sorted_files
        .first()
        .and_then(|filename| Some(filename.trim_end_matches(".md").to_string()));

    let is_new_day = match &newest_note_date {
        Some(date) => date < &current_date,
        None => false, // No notes exist yet
    };

    Ok(DayBoundaryStatus {
        is_new_day,
        current_date,
    })
}

/// Creates today's daily note if needed and returns its loaded content.
#[tauri::command]
pub async fn open_todays_note(state: State<'_, AppState>) -> Result<NoteContentResponse, String> {
    // First, create the note
    let note_manager = state.note_manager()?;
    let config = state.config()?;
    let translations = crate::commands::i18n::get_translations(config.locale.clone());
    let note_header = crate::commands::setup::create_note_header(&config, &translations);
    let created_path = note_manager.create_todays_note(&note_header)?;

    // Drop the guards we no longer need
    drop(note_manager);
    drop(config);

    // Now read the content and load it into the session
    let path_buf = PathBuf::from(&created_path);
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

    session.update_last_modified();
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

            session.insert_line(last_idx, format!("!!! {}", name));
            session.insert_line(last_idx + 1, "".to_string());
            session.insert_line(last_idx + 2, "".to_string());

            // Create a new thread and add it to the list with proper metadata
            session.threads.push(NoteThread {
                id: thread_id,
                name: name.clone(),
                start_line: last_idx,
                end_line: session.lines.len(),
                pinned: false,
            });

            // Update frontmatter with the new thread
            session.update_threads_in_frontmatter();
        }
        Ok(())
    })
    .await
}

/// Detects thread identifiers (`!!! `) in markdown content and returns them as threads.
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

/// Renames the first thread identifier in a markdown string if it differs from the new name.
fn rename_primary_thread(content: &str, new_name: &str) -> Option<String> {
    let (content_start, _) = extract_frontmatter(content);
    let lines: Vec<&str> = content.split('\n').collect();

    for i in content_start..lines.len() {
        if lines[i].starts_with("!!! ") {
            let current_name = lines[i][4..].trim();
            if current_name != new_name {
                let mut owned_lines: Vec<String> =
                    lines.into_iter().map(|s| s.to_string()).collect();
                owned_lines[i] = format!("!!! {}", new_name);
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

/// Renames the first thread identifier in all notes to the specified name.
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
        let full_content = reconstruct_full_content(
            &session.path.clone().ok_or("No path".to_string())?,
            &current_content,
        )?;
        session.load(
            session.path.clone().ok_or("No path".to_string())?,
            full_content,
        );

        // Find the thread by ID from the threads list (which is populated from frontmatter)
        if let Some(thread_to_remove) = session.find_thread_by_id(&thread_id) {
            session.delete_line_range(thread_to_remove.start_line, thread_to_remove.end_line);
            Ok(())
        } else {
            // Debug: print all thread IDs
            let found_ids: Vec<String> = session.threads.iter().map(|t| t.id.clone()).collect();
            Err(format!(
                "Thread '{}' not found. Available thread IDs: {:?}",
                thread_id, found_ids
            ))
        }
    })
    .await
}

/// Toggles the pinned status of a thread by ID in the current note session.
#[tauri::command]
pub async fn toggle_thread_pin(
    thread_id: String,
    current_content: String,
    state: State<'_, AppState>,
) -> Result<NoteContentResponse, String> {
    perform_thread_operation(current_content, state, |session| {
        if let Some(thread) = session.threads.iter_mut().find(|t| t.id == thread_id) {
            thread.pinned = !thread.pinned;
            session.update_threads_in_frontmatter();
            Ok(())
        } else {
            let found_ids: Vec<String> = session.threads.iter().map(|t| t.id.clone()).collect();
            Err(format!(
                "Thread '{}' not found. Available thread IDs: {:?}",
                thread_id, found_ids
            ))
        }
    })
    .await
}

///
/// Unit Tests
///
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    ///
    /// extract_frontmatter Tests
    ///
    #[test]
    fn test_extract_frontmatter_with_valid_frontmatter() {
        let content = "---\ntitle: test\n---\n# Heading\nContent";
        let (count, frontmatter) = extract_frontmatter(content);
        assert_eq!(count, 3);
        assert_eq!(frontmatter, "---\ntitle: test\n---");
    }

    #[test]
    fn test_extract_frontmatter_without_frontmatter() {
        let content = "# Heading\nContent";
        let (count, frontmatter) = extract_frontmatter(content);
        assert_eq!(count, 0);
        assert_eq!(frontmatter, "");
    }

    #[test]
    fn test_extract_frontmatter_empty_content() {
        let content = "";
        let (count, frontmatter) = extract_frontmatter(content);
        assert_eq!(count, 0);
        assert_eq!(frontmatter, "");
    }

    #[test]
    fn test_extract_frontmatter_single_delimiter() {
        let content = "---\nContent";
        let (count, frontmatter) = extract_frontmatter(content);
        assert_eq!(count, 0);
        assert_eq!(frontmatter, "");
    }

    #[test]
    fn test_extract_frontmatter_with_whitespace() {
        let content = "  ---\n  title: test\n  ---\n# Heading";
        let (count, frontmatter) = extract_frontmatter(content);
        assert_eq!(count, 3);
        assert!(frontmatter.contains("title: test"));
    }
    ///
    /// reconstruct_full_content Tests
    ///
    #[test]
    fn test_reconstruct_full_content_with_frontmatter() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "---\ntitle: test\n---\n# Original").unwrap();
        let path = file.path().to_path_buf();

        let result = reconstruct_full_content(&path, "# New Content").unwrap();
        assert!(result.contains("---"));
        assert!(result.contains("title: test"));
        assert!(result.contains("# New Content"));
    }

    #[test]
    fn test_reconstruct_full_content_without_frontmatter() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "# Original Content").unwrap();
        let path = file.path().to_path_buf();

        let result = reconstruct_full_content(&path, "# New Content").unwrap();
        assert_eq!(result, "# New Content");
    }

    ///
    /// rename_primary_thread Tests
    ///
    #[test]
    fn test_rename_primary_thread() {
        let content = "# Thread 1\n!!! OldName\nContent";
        let result = rename_primary_thread(content, "NewName");
        assert!(result.is_some());
        assert!(result.unwrap().contains("!!! NewName"));
    }

    #[test]
    fn test_rename_primary_thread_no_thread() {
        let content = "Just content\nNo threads here";
        let result = rename_primary_thread(content, "NewName");
        assert!(result.is_none());
    }

    #[test]
    fn test_rename_primary_thread_same_name() {
        let content = "!!! AlreadyNamed\nContent";
        let result = rename_primary_thread(content, "AlreadyNamed");
        assert!(result.is_none());
    }

    #[test]
    fn test_rename_primary_thread_with_frontmatter() {
        let content = "---\ntitle: test\n---\n!!! OldName\nContent";
        let result = rename_primary_thread(content, "NewName");
        assert!(result.is_some());
        let result_str = result.unwrap();
        assert!(result_str.contains("!!! NewName"));
        assert!(result_str.contains("---"));
    }

    #[test]
    fn test_rename_primary_thread_only_first() {
        let content = "!!! First\nContent\n!!! Second\nMore";
        let result = rename_primary_thread(content, "Renamed");
        assert!(result.is_some());
        let result_content = result.unwrap();
        assert!(result_content.contains("!!! Renamed"));
        assert!(result_content.contains("!!! Second"));
    }

    ///
    /// get_last_available_note_path Tests (helper function behavior via integration)
    ///
    #[test]
    fn test_get_note_path_by_offset_today() {
        let notes_folder = PathBuf::from("/tmp/notes");
        let path = crate::utils::date::get_note_path_by_offset(&notes_folder, 0);
        let filename = path.file_name().unwrap().to_string_lossy();
        let date = crate::utils::date::get_current_date();
        assert!(filename.contains(&date));
        assert!(filename.ends_with(".md"));
    }

    #[test]
    fn test_get_note_path_by_offset_yesterday() {
        let notes_folder = PathBuf::from("/tmp/notes");
        let path = crate::utils::date::get_note_path_by_offset(&notes_folder, -1);
        let filename = path.file_name().unwrap().to_string_lossy();
        assert!(filename.ends_with(".md"));
    }
    ///
    /// Thread Detection Tests
    ///
    #[tokio::test]
    async fn test_detect_threads_from_content() {
        let content = "!!! Thread1\nContent\n!!! Thread2\nMore";
        let threads = detect_threads(content.to_string(), None).await.unwrap();
        assert_eq!(threads.len(), 2);
        assert_eq!(threads[0].name, "Thread1");
        assert_eq!(threads[1].name, "Thread2");
    }

    #[tokio::test]
    async fn test_detect_threads_empty_content() {
        let content = "";
        let threads = detect_threads(content.to_string(), None).await.unwrap();
        assert_eq!(threads.len(), 0);
    }

    #[tokio::test]
    async fn test_detect_threads_no_threads() {
        let content = "Just regular content\nNo threads";
        let threads = detect_threads(content.to_string(), None).await.unwrap();
        assert_eq!(threads.len(), 0);
    }
    ///
    /// Content Structure Tests
    ///
    #[test]
    fn test_frontmatter_multi_line() {
        let content = "---\nkey1: value1\nkey2: value2\nkey3: value3\n---\n# Heading";
        let (count, frontmatter) = extract_frontmatter(content);
        assert_eq!(count, 5);
        assert!(frontmatter.contains("key1: value1"));
        assert!(frontmatter.contains("key3: value3"));
    }

    #[test]
    fn test_frontmatter_with_empty_lines() {
        let content = "---\n\nkey: value\n\n---\n# Heading";
        let (count, frontmatter) = extract_frontmatter(content);
        assert!(count > 0);
        assert!(frontmatter.contains("key: value"));
    }

    ///
    /// Edge Cases
    ///
    #[test]
    fn test_extract_frontmatter_only_delimiters() {
        let content = "---\n---";
        let (count, frontmatter) = extract_frontmatter(content);
        assert_eq!(count, 2);
        assert_eq!(frontmatter, "---\n---");
    }

    #[test]
    fn test_rename_thread_preserves_rest() {
        let content = "Some\nlines\n!!! Old\nmore\n!!! Other\ncontent";
        let result = rename_primary_thread(content, "New");
        assert!(result.is_some());
        let result_content = result.unwrap();
        assert!(result_content.contains("Some"));
        assert!(result_content.contains("lines"));
        assert!(result_content.contains("more"));
        assert!(result_content.contains("!!! Other"));
        assert!(result_content.contains("content"));
    }
}
