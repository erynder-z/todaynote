use crate::models::response_types::FormattedNote;
use crate::utils;
use chrono::{Locale, NaiveDate};
use std::fs;
use std::path::PathBuf;

pub struct NoteManager {
    pub notes_folder: PathBuf,
    pub locale: String,
}

impl NoteManager {
    pub fn new(notes_folder: PathBuf, locale: String) -> Self {
        Self {
            notes_folder,
            locale,
        }
    }

    pub fn update_config(&mut self, notes_folder: PathBuf, locale: String) {
        self.notes_folder = notes_folder;
        self.locale = locale;
    }

    pub fn ensure_notes_folder_exists(&self) -> Result<(), String> {
        if !self.notes_folder.exists() {
            fs::create_dir_all(&self.notes_folder)
                .map_err(|e| format!("Failed to create notes folder: {}", e))?;
        }
        Ok(())
    }

    pub fn get_today_note_path(&self) -> PathBuf {
        let current_date = utils::date::get_current_date();
        let file_name = format!("{}.md", current_date);
        self.notes_folder.join(&file_name)
    }

    pub fn create_todays_note(&self, note_header: &str) -> Result<PathBuf, String> {
        self.ensure_notes_folder_exists()?;
        let file_path = self.get_today_note_path();

        if file_path.exists() {
            return Ok(file_path);
        }

        let current_date = utils::date::get_current_date();
        let note_content = format!("# {}: {}", note_header, current_date);

        fs::write(&file_path, note_content).map_err(|e| format!("Failed to create note: {}", e))?;

        Ok(file_path)
    }

    pub fn list_notes(&self) -> Result<Vec<FormattedNote>, String> {
        if !self.notes_folder.exists() {
            return Ok(vec![]);
        }

        let entries = fs::read_dir(&self.notes_folder)
            .map_err(|e| format!("Failed to read directory: {}", e))?;

        let mut notes: Vec<FormattedNote> = entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let file_name = entry.file_name().into_string().ok()?;
                if file_name.ends_with(".md") && !file_name.starts_with(".") {
                    Some(FormattedNote {
                        filename: file_name.clone(),
                        formatted_name: self.format_note_name(&file_name),
                    })
                } else {
                    None
                }
            })
            .collect();

        notes.sort_by(|a, b| a.filename.cmp(&b.filename));
        notes.reverse();

        Ok(notes)
    }

    pub fn format_note_name(&self, note_name: &str) -> String {
        let without_ext = note_name.replace(".md", "");

        if let Ok(date) = NaiveDate::parse_from_str(&without_ext, "%Y-%m-%d") {
            let locale = match self.locale.as_str() {
                "de" => Locale::de_DE,
                "jp" => Locale::ja_JP,
                _ => Locale::en_US,
            };

            match self.locale.as_str() {
                "de" => format!("{}", date.format_localized("%A, %e. %B %Y", locale)),
                "jp" => format!("{}", date.format_localized("%Y年%m月%d日 (%A)", locale)),
                _ => format!("{}", date.format_localized("%A, %B %e, %Y", locale)),
            }
        } else {
            without_ext
        }
    }

    pub fn read_note_content(&self, path: &PathBuf) -> Result<String, String> {
        fs::read_to_string(path).map_err(|e| format!("Failed to read note content: {}", e))
    }
}
