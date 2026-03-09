//! Serializable data structures for frontend communication.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Serialized version of the application configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub notes_folder: String,
    pub locale: String,
    pub theme: String,
    pub remember_window_size: bool,
}

/// Metadata for a single note file in the list.
#[derive(Debug, Serialize, Deserialize)]
pub struct FormattedNote {
    pub filename: String,
    pub formatted_name: String,
}

/// A search match from the note archive.
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub filename: String,
    pub excerpt: String,
}

/// The complete state payload sent to the frontend during initialization.
#[derive(Debug, Serialize, Deserialize)]
pub struct InitialAppState {
    pub notes_folder: Option<String>,
    pub locale: String,
    pub theme: String,
    pub remember_window_size: bool,
    pub available_locales: Vec<LocaleInfo>,
    pub available_themes: Vec<ThemeInfo>,
    pub translations: HashMap<String, String>,
    pub theme_colors: HashMap<String, String>,
    pub today_note_path: Option<String>,
    pub today_note_content: Option<NoteContentResponse>,
}

/// Result of a folder validation check.
#[derive(Debug, Serialize, Deserialize)]
pub struct FolderValidation {
    pub is_valid: bool,
    pub is_writable: bool,
    pub exists: bool,
    pub note_count: usize,
    pub error: Option<String>,
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

/// Structured response for note content.
#[derive(Debug, Serialize, Deserialize)]
pub struct NoteContentResponse {
    pub lines: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub metadata_range: Option<(usize, usize)>,
}
