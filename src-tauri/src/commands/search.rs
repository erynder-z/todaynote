//! Tauri commands for note search operations.

use crate::models::app_state::AppState;
use crate::models::response_types::SearchResult;
use crate::services::search::SearchService;
use tauri::State;

/// Performs a full-text search across all notes.
#[tauri::command]
pub async fn search_notes(
    query: String,
    is_fuzzy: bool,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
    let note_manager = state.note_manager()?;
    let service = SearchService::new(&note_manager);
    service.search(&query, is_fuzzy)
}
