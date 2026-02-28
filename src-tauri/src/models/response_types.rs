use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub notes_folder: String,
    pub locale: String,
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

use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct InitialAppState {
    pub notes_folder: Option<String>,
    pub locale: String,
    pub translations: HashMap<String, String>,
    pub today_note_path: Option<String>,
    pub today_note_content: Option<String>,
}
