//! Tauri commands for tag-related operations.

use crate::commands::notes::reconstruct_full_content;
use crate::models::app_state::AppState;
use crate::models::note_session::NoteSession;
use crate::models::response_types::NoteContentResponse;
use crate::services::tag_manager::TagManager;
use std::fs;
use tauri::State;

/// Helper to sync frontend content, perform a tag operation, and save the result.
async fn perform_tag_operation<F>(
    current_content: String,
    state: State<'_, AppState>,
    operation: F,
) -> Result<NoteContentResponse, String>
where
    F: FnOnce(&TagManager, &mut NoteSession),
{
    let mut session = state.note_session()?;

    let path = session
        .path
        .clone()
        .ok_or_else(|| "No active note session".to_string())?;

    // Sync session with frontend content first
    // Use the session's current frontmatter combined with the frontend's content
    // to ensure we don't lose any backend-made changes (like thread ID comments)
    let full_content = if let Some((_, end)) = session.frontmatter_range {
        let frontmatter_lines = &session.lines[..=end];
        let frontmatter = frontmatter_lines.join("\n");
        format!("{}\n{}", frontmatter, current_content)
    } else {
        // No frontmatter in session, use reconstruct_full_content as fallback
        reconstruct_full_content(&path, &current_content)?
    };
    session.load(path.clone(), full_content);

    let mut tag_manager = state.tag_manager()?;
    operation(&tag_manager, &mut session);
    tag_manager.invalidate_cache();

    let full_content = session.get_full_content();
    fs::write(&path, &full_content).map_err(|e| format!("Failed to save note: {}", e))?;

    let note_manager = state.note_manager()?;
    Ok(NoteContentResponse::from_session(
        &session,
        &note_manager,
        &tag_manager,
    ))
}

/// Adds a tag to the current note session and writes it to disk.
#[tauri::command]
pub async fn add_note_tag(
    tag: String,
    current_content: String,
    state: State<'_, AppState>,
) -> Result<NoteContentResponse, String> {
    perform_tag_operation(current_content, state, |tm, session| {
        tm.add_tag_to_session(session, tag);
    })
    .await
}

/// Removes a tag from the current note session and writes it to disk.
#[tauri::command]
pub async fn remove_note_tag(
    tag: String,
    current_content: String,
    state: State<'_, AppState>,
) -> Result<NoteContentResponse, String> {
    perform_tag_operation(current_content, state, |tm, session| {
        tm.remove_tag_from_session(session, tag);
    })
    .await
}

/// Returns all unique tags from all notes, sorted by usage frequency.
#[tauri::command]
pub async fn get_all_tags(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let folder_path = {
        let note_manager = state.note_manager()?;
        note_manager.notes_folder.clone()
    };
    let mut tag_manager = state.tag_manager()?;
    tag_manager.get_all_tags(&folder_path)
}

/// Returns a filtered list of tag suggestions.
#[tauri::command]
pub async fn get_tag_suggestions(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let folder_path = {
        let note_manager = state.note_manager()?;
        note_manager.notes_folder.clone()
    };

    let session = state.note_session()?;
    let mut tag_manager = state.tag_manager()?;

    let active_tags = tag_manager.get_tags_from_session(&session);
    Ok(tag_manager.suggest_tags(&folder_path, &query, &active_tags, 20))
}
