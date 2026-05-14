use crate::models::app_state::AppState;
use crate::models::response_types::{SearchResult, ThreadAggregationResult, ThreadSearchResult};
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

/// Searches for unique thread names across all notes.
#[tauri::command]
pub async fn search_threads(
    query: String,
    is_fuzzy: bool,
    state: State<'_, AppState>,
) -> Result<Vec<ThreadSearchResult>, String> {
    let note_manager = state.note_manager()?;
    let service = SearchService::new(&note_manager);
    service.search_threads(&query, is_fuzzy)
}

/// Aggregates content from all sections with the given thread name.
#[tauri::command]
pub async fn aggregate_thread(
    name: String,
    state: State<'_, AppState>,
) -> Result<ThreadAggregationResult, String> {
    let note_manager = state.note_manager()?;
    let service = SearchService::new(&note_manager);
    service.aggregate_thread(&name)
}
