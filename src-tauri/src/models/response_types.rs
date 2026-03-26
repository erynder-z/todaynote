//! Serializable data structures for frontend communication.

use crate::models::note_session::NoteSession;
use crate::services::note_manager::NoteManager;
use crate::services::tag_manager::TagManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Serialized version of the application configuration.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigResponse {
    pub notes_folder: String,
    pub locale: String,
    pub theme: String,
    pub remember_window_size: bool,
}

/// Metadata for a single note file in the list.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct AppPayload {
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
    pub lines: Vec<String>,
    pub metadata: NoteMetadata,
    pub metadata_range: Option<(usize, usize)>,
}

impl NoteContentResponse {
    /// Helper to create a NoteContentResponse from session data and services.
    pub fn from_session(
        session: &NoteSession,
        note_manager: &NoteManager,
        tag_manager: &TagManager,
    ) -> Self {
        let filename = session
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|f| f.to_str())
            .unwrap_or("");

        let formatted_date = note_manager.format_note_name(filename);
        let tags = tag_manager.get_tags_from_session(session);
        let raw_metadata = session.get_metadata();

        Self {
            lines: session.lines.clone(),
            metadata: NoteMetadata {
                formatted_date,
                tags,
                raw: raw_metadata,
            },
            metadata_range: session.frontmatter_range,
        }
    }
}
