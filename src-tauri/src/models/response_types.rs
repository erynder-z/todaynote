//! Serializable data structures for frontend communication.

use crate::models::config::ShortcutConfig;
use crate::models::note_session::{NoteSession, NoteThread};
use crate::services::note_manager::NoteManager;
use crate::services::tag_manager::TagManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Status of day boundary check comparing active note path with current date.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DayBoundaryStatus {
    pub is_new_day: bool,
    pub current_date: String,
}

/// Serialized version of the application configuration.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigResponse {
    pub notes_folder: String,
    pub locale: String,
    pub theme: String,
    pub remember_app_layout: bool,
    pub notes_list_layout: String,
    pub remember_settings: bool,
    pub search_mode: String,
    pub search_is_fuzzy: bool,
    pub search_selected_tag: Option<String>,
    pub sidebar_open: bool,
    pub control_center_width: f64,
    pub default_thread_name: Option<String>,
    pub use_default_thread_name: bool,
    pub identicon_style: String,
    pub thread_shortcuts_mode: String,
    pub date_format_style: String,
    pub text_copy_mode: String,
    pub floating_toolbar_enabled: bool,
    pub shortcuts: HashMap<String, ShortcutConfig>,
    pub font_family: Option<String>,
    pub use_custom_font: bool,
}

/// Metadata for a single note file in the list.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormattedNote {
    pub filename: String,
    pub formatted_name: String,
    pub preview: String,
    pub tags: Vec<String>,
    pub threads: Vec<String>,
    pub word_count: usize,
    pub has_code: bool,
}

/// A paginated response for note listing.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteListResponse {
    pub notes: Vec<FormattedNote>,
    pub total_count: usize,
}

/// A search match from the note archive.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub filename: String,
    pub formatted_name: String,
    pub excerpt: String,
    pub line_number: usize,
    pub score: i64,
    pub indices: Vec<u32>,
}

/// A unique thread name found during search.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadSearchResult {
    pub name: String,
    pub note_count: usize,
}

/// A unique tag found during search.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagSearchResult {
    pub name: String,
    pub note_count: usize,
}

/// An aggregated view of content from threads with the same name across notes.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadAggregationResult {
    pub thread_name: String,
    pub items: Vec<ThreadAggregationItem>,
}

/// Content of a single thread block.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadAggregationItem {
    pub filename: String,
    pub formatted_date: String,
    pub content: String,
    pub thread_id: String,
}

/// A pinned thread with its excerpt and source information.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PinnedThreadItem {
    pub thread_id: String,
    pub thread_name: String,
    pub filename: String,
    pub formatted_date: String,
    pub excerpt: String,
    pub line_number: usize,
}

/// The complete state payload sent to the frontend during initialization.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppPayload {
    pub notes_folder: Option<String>,
    pub locale: String,
    pub theme: String,
    pub remember_app_layout: bool,
    pub notes_list_layout: String,
    pub remember_settings: bool,
    pub search_mode: String,
    pub search_is_fuzzy: bool,
    pub search_selected_tag: Option<String>,
    pub sidebar_open: bool,
    pub control_center_width: f64,
    pub default_thread_name: Option<String>,
    pub use_default_thread_name: bool,
    pub identicon_style: String,
    pub thread_shortcuts_mode: String,
    pub date_format_style: String,
    pub text_copy_mode: String,
    pub floating_toolbar_enabled: bool,
    pub shortcuts: HashMap<String, ShortcutConfig>,
    pub font_family: Option<String>,
    pub use_custom_font: bool,
    pub available_locales: Vec<LocaleInfo>,
    pub available_themes: Vec<ThemeInfo>,
    pub translations: HashMap<String, String>,
    pub theme_colors: HashMap<String, String>,
    pub today_note_path: Option<String>,
    pub today_note_content: Option<NoteContentResponse>,
    pub is_mac: bool,
}

/// Result of a folder validation check.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderValidation {
    pub is_valid: bool,
    pub is_writable: bool,
    pub exists: bool,
    pub note_count: usize,
    pub error: Option<String>,
}

/// Result of importing notes from a backup archive.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    pub imported: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

/// Statistics about the note collection.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatistics {
    pub total_notes: usize,
    pub total_tags: usize,
    pub total_threads: usize,
    pub total_characters: usize,
    pub total_words: usize,
    pub current_streak: usize,
    pub best_streak: usize,
    pub top_tags: Vec<TagStat>,
    pub top_threads: Vec<ThreadStat>,
    pub daily_stats: Vec<DailyStat>,
    pub weekday_distribution: Vec<usize>, // 0=Mon, 6=Sun
    pub insights: Vec<InsightResponse>,
}

/// A structured insight with a translation key and parameters.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsightResponse {
    pub key: String,
    pub params: HashMap<String, String>,
}

/// A tag with its usage count.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagStat {
    pub name: String,
    pub count: usize,
}

/// A thread with its usage count.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadStat {
    pub name: String,
    pub count: usize,
}

/// Statistics for a single day/note.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyStat {
    pub date: String,
    pub character_count: usize,
    pub word_count: usize,
}

/// Metadata for an available locale.
#[derive(Debug, Serialize, Deserialize)]
pub struct LocaleInfo {
    pub id: String,
    pub name: String,
}

/// Metadata for an available theme.
#[derive(Debug, Serialize, Deserialize)]
pub struct ThemeInfo {
    pub id: String,
    pub name: String,
}

/// Structured metadata for a note.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteMetadata {
    pub formatted_date: String,
    pub tags: Vec<String>,
    pub raw: HashMap<String, String>,
}

/// Structured response for note content.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteContentResponse {
    pub path: String,
    pub content: String,
    pub metadata: NoteMetadata,
    pub threads: Vec<NoteThread>,
}

impl NoteContentResponse {
    /// Helper to create a NoteContentResponse from session data and services.
    pub fn from_session(
        session: &NoteSession,
        note_manager: &NoteManager,
        tag_manager: &TagManager,
    ) -> Self {
        let path = session
            .path
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();

        let filename = session
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|f| f.to_str())
            .unwrap_or("");

        let formatted_date = note_manager.format_note_name(filename);
        let tags = tag_manager.get_tags_from_session(session);
        let raw_metadata = session.get_metadata();

        let content_start = session.get_content_start_index();

        // Map absolute thread indices to relative content indices
        let threads: Vec<NoteThread> = session
            .threads
            .iter()
            .map(|s| {
                let rel_start = if s.start_line >= content_start {
                    s.start_line - content_start
                } else {
                    0
                };
                let rel_end = if s.end_line >= content_start {
                    s.end_line - content_start
                } else {
                    0
                };
                NoteThread {
                    id: s.id.clone(),
                    name: s.name.clone(),
                    start_line: rel_start,
                    end_line: rel_end,
                    pinned: s.pinned,
                }
            })
            .collect();

        // Get content lines (excluding frontmatter) and join them into a single string
        let content_lines: Vec<String> =
            session.lines[session.get_content_start_index()..].to_vec();
        let content = content_lines.join("\n");

        Self {
            content,
            metadata: NoteMetadata {
                formatted_date,
                tags,
                raw: raw_metadata,
            },
            threads,
            path,
        }
    }
}
