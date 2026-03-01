use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub notes_folder: String,
    pub locale: String,
    pub theme: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FormattedNote {
    pub filename: String,
    pub formatted_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub filename: String,
    pub excerpt: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitialAppState {
    pub notes_folder: Option<String>,
    pub locale: String,
    pub theme: String,
    pub available_locales: Vec<LocaleInfo>,
    pub available_themes: Vec<ThemeInfo>,
    pub translations: HashMap<String, String>,
    pub theme_colors: HashMap<String, String>,
    pub today_note_path: Option<String>,
    pub today_note_content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FolderValidation {
    pub is_valid: bool,
    pub is_writable: bool,
    pub exists: bool,
    pub note_count: usize,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocaleInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThemeInfo {
    pub id: String,
    pub name: String,
}
