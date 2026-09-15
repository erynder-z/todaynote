use crate::models::app_state::AppState;
use crate::models::response_types::{
    PinnedThreadItem, SearchResult, TagSearchResult, ThreadAggregationResult, ThreadSearchResult,
};
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

/// Searches for unique tags across all notes.
#[tauri::command]
pub async fn search_tags(
    query: String,
    is_fuzzy: bool,
    state: State<'_, AppState>,
) -> Result<Vec<TagSearchResult>, String> {
    let note_manager = state.note_manager()?;
    let service = SearchService::new(&note_manager);
    service.search_tags(&query, is_fuzzy)
}

/// Finds all notes that contain a specific tag.
#[tauri::command]
pub async fn search_notes_by_tag(
    tag: String,
    query: String,
    is_fuzzy: bool,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
    let note_manager = state.note_manager()?;
    let service = SearchService::new(&note_manager);
    service.search_notes_by_tag(&tag, &query, is_fuzzy)
}

/// Aggregates content from all threads with the given thread name.
#[tauri::command]
pub async fn aggregate_thread(
    name: String,
    state: State<'_, AppState>,
) -> Result<ThreadAggregationResult, String> {
    let note_manager = state.note_manager()?;
    let service = SearchService::new(&note_manager);
    service.aggregate_thread(&name)
}

/// Processes search results with filtering and sorting options.
#[tauri::command]
pub async fn process_search_results(
    results: Vec<SearchResult>,
    min_score: Option<i64>,
    max_results: Option<usize>,
    filename_filter: Option<String>,
    sort_by: String,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
    let note_manager = state.note_manager()?;
    let service = SearchService::new(&note_manager);
    let filename_filter_ref = filename_filter.as_deref();
    Ok(service.process_search_results(
        results,
        min_score,
        max_results,
        filename_filter_ref,
        &sort_by,
    ))
}

/// Returns all pinned threads across all notes with excerpts.
#[tauri::command]
pub async fn get_pinned_threads(
    state: State<'_, AppState>,
) -> Result<Vec<PinnedThreadItem>, String> {
    let note_manager = state.note_manager()?;
    let service = SearchService::new(&note_manager);
    service.get_pinned_threads()
}

/// Returns the content of a specific thread from a specific note.
#[tauri::command]
pub async fn get_thread_content(
    filename: String,
    thread_id: String,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let note_manager = state.note_manager()?;
    let service = SearchService::new(&note_manager);
    service.get_thread_content(&filename, &thread_id)
}

///
/// Unit Tests
///
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::response_types::ThreadAggregationItem;
    use crate::services::note_manager::NoteManager;
    use crate::services::search::SearchService;
    use std::fs;
    use tempfile::tempdir;

    /// Creates a NoteManager backed by a temp dir populated with the given (filename, content) pairs.
    fn setup_notes(notes: &[(&str, &str)]) -> (tempfile::TempDir, NoteManager) {
        let dir = tempdir().expect("Failed to create temp dir");
        let note_manager = NoteManager::new(dir.path().to_path_buf(), "en".to_string());

        for (filename, content) in notes {
            fs::write(dir.path().join(filename), content).expect("Failed to write test note");
        }

        (dir, note_manager)
    }

    fn make_result(filename: &str, score: i64) -> SearchResult {
        SearchResult {
            filename: filename.to_string(),
            formatted_name: filename.to_string(),
            excerpt: "excerpt".to_string(),
            line_number: 0,
            score,
            indices: vec![],
        }
    }

    ///
    /// Serialization tests
    ///
    #[test]
    fn test_search_result_serialization() {
        let result = SearchResult {
            filename: "2024-01-15.md".to_string(),
            formatted_name: "January 15, 2024".to_string(),
            excerpt: "This is a match".to_string(),
            line_number: 5,
            score: 42,
            indices: vec![0, 1, 2],
        };

        let json = serde_json::to_string(&result).expect("Failed to serialize");
        let deserialized: SearchResult =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.filename, result.filename);
        assert_eq!(deserialized.line_number, result.line_number);
        assert_eq!(deserialized.score, result.score);
        assert_eq!(deserialized.indices, result.indices);
        assert!(json.contains("formattedName"), "Should use camelCase");
        assert!(
            !json.contains("formatted_name"),
            "Should not use snake_case"
        );
    }

    #[test]
    fn test_thread_aggregation_result_serialization() {
        let result = ThreadAggregationResult {
            thread_name: "Journal".to_string(),
            items: vec![ThreadAggregationItem {
                filename: "2024-01-01.md".to_string(),
                formatted_date: "January 1, 2024".to_string(),
                content: "Entry content".to_string(),
                thread_id: "abc-123".to_string(),
            }],
        };

        let json = serde_json::to_string(&result).expect("Failed to serialize");
        let deserialized: ThreadAggregationResult =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.thread_name, "Journal");
        assert_eq!(deserialized.items.len(), 1);
        assert_eq!(deserialized.items[0].thread_id, "abc-123");
        assert!(json.contains("threadName"), "Should use camelCase");
        assert!(json.contains("threadId"), "Should use camelCase");
    }

    #[test]
    fn test_pinned_thread_item_serialization() {
        let item = PinnedThreadItem {
            thread_id: "uuid-1".to_string(),
            thread_name: "Tasks".to_string(),
            filename: "2024-03-10.md".to_string(),
            formatted_date: "March 10, 2024".to_string(),
            excerpt: "Do something".to_string(),
            line_number: 7,
        };

        let json = serde_json::to_string(&item).expect("Failed to serialize");
        let deserialized: PinnedThreadItem =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.thread_id, "uuid-1");
        assert_eq!(deserialized.thread_name, "Tasks");
        assert_eq!(deserialized.line_number, 7);
        assert!(json.contains("threadId"), "Should use camelCase");
    }

    ///
    /// filter_search_results & sort_search_results tests
    ///
    #[test]
    fn test_filter_and_sort_combined() {
        let results: Vec<SearchResult> = (0..20)
            .map(|i| {
                let year = if i % 2 == 0 { "2024" } else { "2023" };
                make_result(&format!("{}-{:02}.md", year, i), i as i64)
            })
            .collect();

        let filtered = SearchService::filter_search_results(results, Some(5), None, Some("2024"));

        assert_eq!(
            filtered.len(),
            7,
            "Should keep 2024 results with score >= 5"
        );
        assert!(filtered.iter().all(|r| r.score >= 5));
        assert!(filtered.iter().all(|r| r.filename.contains("2024")));

        let sorted = SearchService::sort_search_results(filtered, "score");
        assert_eq!(sorted[0].score, 18, "Highest score should be first");
    }

    #[test]
    fn test_sort_by_filename_and_date() {
        let mk = || {
            vec![
                make_result("2024-01-01_extra.md", 1),
                make_result("2024-03-15_extra.md", 2),
                make_result("2024-02-10_extra.md", 3),
            ]
        };

        let by_filename = SearchService::sort_search_results(mk(), "filename");
        assert_eq!(by_filename[0].filename, "2024-01-01_extra.md");

        let by_date = SearchService::sort_search_results(mk(), "date");
        assert_eq!(
            by_date[0].filename, "2024-03-15_extra.md",
            "Newest date first"
        );
    }

    ///
    /// search tests
    ///
    #[test]
    fn test_search_exact_match() {
        let (_dir, nm) = setup_notes(&[(
            "2024-01-01.md",
            "---\ntags: []\n---\nThis is a test note about rust programming.",
        )]);

        let service = SearchService::new(&nm);
        let results = service.search("rust", false).expect("Search failed");

        assert_eq!(results.len(), 1, "Should find one match");
        assert!(results[0].excerpt.to_lowercase().contains("rust"));
    }

    #[test]
    fn test_search_fuzzy_match() {
        let (_dir, nm) = setup_notes(&[(
            "2024-01-01.md",
            "---\ntags: []\n---\nThe quick brown fox jumps.",
        )]);

        let service = SearchService::new(&nm);
        let results = service.search("brwn", true).expect("Fuzzy search failed");

        assert_eq!(results.len(), 1, "Fuzzy match should find 'brown'");
        assert!(
            results[0].score > 0,
            "Fuzzy match should have positive score"
        );
    }

    #[test]
    fn test_search_skips_frontmatter_and_empty_query() {
        let (_dir, nm) = setup_notes(&[(
            "2024-01-01.md",
            "---\ntags: [secret]\n---\nThis is visible content.",
        )]);

        let service = SearchService::new(&nm);

        assert!(
            service
                .search("secret", false)
                .expect("Search failed")
                .is_empty(),
            "Should not match content inside frontmatter"
        );
        assert!(
            service.search("", false).expect("Search failed").is_empty(),
            "Empty query should return no results"
        );
    }

    ///
    /// search_threads & search_tags tests
    ///
    #[test]
    fn test_search_threads_finds_and_counts() {
        let (_dir, nm) = setup_notes(&[
            ("2024-01-01.md", "!!! Ideas\nFirst idea"),
            (
                "2024-01-02.md",
                "!!! Ideas\nSecond idea\n!!! Tasks\nTask one",
            ),
        ]);

        let service = SearchService::new(&nm);
        let results = service
            .search_threads("idea", false)
            .expect("Thread search failed");

        assert_eq!(results.len(), 1, "Should only match 'Ideas'");
        assert_eq!(results[0].note_count, 2, "Ideas should appear in 2 notes");
    }

    #[test]
    fn test_search_tags_finds_and_counts() {
        let (_dir, nm) = setup_notes(&[
            (
                "2024-01-01.md",
                "---\ntags: [rust, programming]\n---\nContent",
            ),
            ("2024-01-02.md", "---\ntags: [rust, testing]\n---\nContent"),
        ]);

        let service = SearchService::new(&nm);
        let results = service
            .search_tags("rust", false)
            .expect("Tag search failed");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].note_count, 2, "rust should be in 2 notes");
    }

    ///
    // search_notes_by_tag tests
    ///
    #[test]
    fn test_search_notes_by_tag_with_query() {
        let (_dir, nm) = setup_notes(&[
            (
                "2024-01-01.md",
                "---\ntags: [Important]\n---\nNote about rust.",
            ),
            (
                "2024-01-02.md",
                "---\ntags: [important]\n---\nNote about python.",
            ),
        ]);

        let service = SearchService::new(&nm);
        let results = service
            .search_notes_by_tag("important", "rust", false)
            .expect("Tag note search failed");

        assert_eq!(
            results.len(),
            1,
            "Should find one matching note (case-insensitive tag)"
        );
        assert_eq!(results[0].filename, "2024-01-01.md");
    }

    ///
    /// aggregate_thread tests
    ///
    #[test]
    fn test_aggregate_thread_collects_blocks() {
        let (_dir, nm) = setup_notes(&[
            ("2024-01-01.md", "!!! Journal\nFirst entry"),
            ("2024-01-02.md", "!!! Journal\nSecond entry"),
        ]);

        let service = SearchService::new(&nm);
        let result = service
            .aggregate_thread("Journal")
            .expect("Aggregation failed");

        assert_eq!(result.thread_name, "Journal");
        assert_eq!(result.items.len(), 2, "Should aggregate from both notes");
    }

    ///
    /// get_pinned_threads & get_thread_content tests
    ///
    #[test]
    fn test_get_pinned_threads_finds_pinned() {
        let content = "---\nthreads: t1:0:pinned\ntags: []\n---\n!!! Tasks\nDo something\n\n";
        let (_dir, nm) = setup_notes(&[("2024-01-01.md", content)]);

        let service = SearchService::new(&nm);
        let results = service.get_pinned_threads().expect("Pinned threads failed");

        assert_eq!(results.len(), 1, "Should find one pinned thread");
        assert_eq!(results[0].thread_id, "t1");
        assert_eq!(results[0].thread_name, "Tasks");
    }

    #[test]
    fn test_get_thread_content_returns_content() {
        let content = "---\nthreads: t1:0\ntags: []\n---\n!!! Tasks\nDo something\n\n";
        let (_dir, nm) = setup_notes(&[("2024-01-01.md", content)]);

        let service = SearchService::new(&nm);
        let result = service
            .get_thread_content("2024-01-01.md", "t1")
            .expect("Get thread content failed");

        assert!(result.is_some(), "Should find the thread");
        assert!(result.unwrap().contains("Do something"));

        assert!(
            service
                .get_thread_content("nonexistent.md", "t1")
                .expect("Get thread content failed")
                .is_none(),
            "Nonexistent file should return None"
        );
    }

    ///
    /// process_search_results tests
    ///
    #[test]
    fn test_process_search_results_filters_and_sorts() {
        let (_dir, nm) = setup_notes(&[]);
        let service = SearchService::new(&nm);

        let results: Vec<SearchResult> = vec![
            make_result("2024-01-01.md", 1),
            make_result("2024-02-01.md", 10),
            make_result("2024-03-01.md", 5),
        ];

        let processed = service.process_search_results(results, Some(5), None, None, "score");

        assert_eq!(processed.len(), 2, "Should filter by min_score");
        assert_eq!(
            processed[0].score, 10,
            "Should be sorted by score descending"
        );
    }
}
