//! Manager to handle note operations and formatting.

use crate::models::response_types::{AppStatistics, FormattedNote, NoteListResponse};
use crate::utils;
use chrono::{Locale, NaiveDate};
use std::fs;
use std::path::PathBuf;

/// Manager responsible for note-related operations like creation, listing, and formatting.
pub struct NoteManager {
    /// The root directory where notes are stored.
    pub notes_folder: PathBuf,
    /// The currently active locale for date formatting.
    pub locale: String,
}

impl NoteManager {
    /// Creates a new `NoteManager` with the specified configuration.
    pub fn new(notes_folder: PathBuf, locale: String) -> Self {
        Self {
            notes_folder,
            locale,
        }
    }

    /// Updates the manager's configuration when settings change.
    pub fn update_config(&mut self, notes_folder: PathBuf, locale: String) {
        self.notes_folder = notes_folder;
        self.locale = locale;
    }

    /// Ensures that the configured notes folder exists, creating it if necessary.
    pub fn ensure_notes_folder_exists(&self) -> Result<(), String> {
        if !self.notes_folder.exists() {
            fs::create_dir_all(&self.notes_folder)
                .map_err(|e| format!("Failed to create notes folder: {}", e))?;
        }
        Ok(())
    }

    /// Returns the absolute path to today's daily note file.
    pub fn get_today_note_path(&self) -> PathBuf {
        let current_date = utils::date::get_current_date();
        let file_name = format!("{}.md", current_date);
        self.notes_folder.join(&file_name)
    }

    /// Checks if today's daily note file already exists.
    pub fn todays_note_exists(&self) -> bool {
        self.get_today_note_path().exists()
    }

    /// Creates today's daily note if it doesn't already exist.
    ///
    /// The note is initialized with YAML metadata and a default thread header.
    pub fn create_todays_note(&self, note_header: &str) -> Result<PathBuf, String> {
        self.ensure_notes_folder_exists()?;
        let file_path = self.get_today_note_path();

        if self.todays_note_exists() {
            return Ok(file_path);
        }

        let current_date = utils::date::get_current_date();
        let note_content = format!(
            "---\ncreated: {}\ntags: []\n---\n# {}\n",
            current_date, note_header
        );

        fs::write(&file_path, note_content).map_err(|e| format!("Failed to create note: {}", e))?;

        Ok(file_path)
    }

    /// Retrieves all valid Markdown note filenames from the notes folder,
    /// sorted by name descending (most recent first).
    pub fn get_sorted_note_files(&self) -> Result<Vec<String>, String> {
        if !self.notes_folder.exists() {
            return Ok(vec![]);
        }
        let entries = fs::read_dir(&self.notes_folder)
            .map_err(|e| format!("Failed to read directory: {}", e))?;
        let mut files: Vec<String> = entries
            .filter_map(|e| {
                let e = e.ok()?;
                let name = e.file_name().into_string().ok()?;
                if name.ends_with(".md") && !name.starts_with(".") {
                    Some(name)
                } else {
                    None
                }
            })
            .collect();
        files.sort_by(|a, b| b.cmp(a));
        Ok(files)
    }

    /// Transforms a note filename into a FormattedNote by reading and processing its content.
    fn format_note_file(&self, file_name: &str) -> Option<FormattedNote> {
        let path = self.notes_folder.join(file_name);
        let content = fs::read_to_string(&path).unwrap_or_default();
        Some(FormattedNote {
            filename: file_name.to_string(),
            formatted_name: self.format_note_name(file_name),
            preview: self.extract_preview(&content),
            tags: crate::utils::tag_parser::parse_tags_from_content(&content),
            threads: self.extract_threads(&content, 5),
            word_count: crate::utils::markdown::count_words(&content),
        })
    }

    /// Lists all valid Markdown notes in the configured notes folder.
    ///
    /// Notes are returned as `FormattedNote` objects with localized display names.
    /// If a limit is provided, only the most recent N notes are fully processed.
    pub fn list_notes(&self, limit: Option<usize>) -> Result<NoteListResponse, String> {
        let all_files = self.get_sorted_note_files()?;
        let total_count = all_files.len();
        let files_to_process: Vec<String> = match limit {
            Some(l) => all_files[..l.min(all_files.len())].to_vec(),
            None => all_files,
        };
        let notes = files_to_process
            .iter()
            .filter_map(|f| self.format_note_file(f))
            .collect();
        Ok(NoteListResponse { notes, total_count })
    }

    /// Extracts the first N thread names (headings) from the content.
    pub fn extract_threads(&self, content: &str, limit: usize) -> Vec<String> {
        content
            .lines()
            .filter(|l| l.starts_with("# "))
            .take(limit)
            .map(|l| l[2..].trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Extracts a short preview from the note content, skipping frontmatter and headings.
    fn extract_preview(&self, content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut start_idx = 0;

        // Skip frontmatter
        if lines.first().map(|l| l.trim()) == Some("---") {
            for (i, line) in lines.iter().enumerate().skip(1) {
                if line.trim() == "---" {
                    start_idx = i + 1;
                    break;
                }
            }
        }

        let mut preview_text = Vec::new();
        for line in lines.iter().skip(start_idx) {
            let stripped = crate::utils::markdown::strip_markdown(line);
            // Skip headings (already handled by strip_markdown but we also skip empty results)
            if stripped.is_empty() {
                continue;
            }
            preview_text.push(stripped);
            if preview_text.len() > 3 {
                break;
            }
        }

        let joined = preview_text.join(" ");
        let (preview, _) = crate::utils::markdown::generate_excerpt(&joined, &[], 150);
        preview
    }

    /// Formats a note's filename into a human-readable, localized string.
    ///
    /// If the filename follows the `YYYY-MM-DD.md` pattern, it is transformed
    /// into a localized date string.
    pub fn format_note_name(&self, note_name: &str) -> String {
        let without_ext = note_name.replace(".md", "");

        if let Ok(date) = NaiveDate::parse_from_str(&without_ext, "%Y-%m-%d") {
            let locale = match self.locale.as_str() {
                "de" => Locale::de_DE,
                "ja" => Locale::ja_JP,
                _ => Locale::en_US,
            };

            match self.locale.as_str() {
                "de" => format!("{}", date.format_localized("%A, %e. %B %Y", locale)),
                "ja" => format!("{}", date.format_localized("%Y年%m月%d日 (%A)", locale)),
                _ => format!("{}", date.format_localized("%A, %B %e, %Y", locale)),
            }
        } else {
            without_ext
        }
    }

    /// Reads the content of a note file from the specified path.
    pub fn read_note_content(&self, path: &PathBuf) -> Result<String, String> {
        fs::read_to_string(path).map_err(|e| format!("Failed to read note content: {}", e))
    }

    /// Deletes all notes that have no content (only frontmatter and headings).
    pub fn purge_empty_notes(&self) -> Result<usize, String> {
        let all_files = self.get_sorted_note_files()?;
        let mut purged_count = 0;

        for file_name in all_files {
            let path = self.notes_folder.join(&file_name);
            if let Ok(content) = fs::read_to_string(&path) {
                if self.is_note_empty(&content) {
                    fs::remove_file(&path)
                        .map_err(|e| format!("Failed to delete empty note {}: {}", file_name, e))?;
                    purged_count += 1;
                }
            }
        }
        Ok(purged_count)
    }

    /// Determines if a note is considered "empty" (no tags and no content beyond headings).
    fn is_note_empty(&self, content: &str) -> bool {
        // If it has tags, it's not empty
        let tags = crate::utils::tag_parser::parse_tags_from_content(content);
        if !tags.is_empty() {
            return false;
        }

        let lines: Vec<&str> = content.lines().collect();
        let mut start_idx = 0;

        // Skip frontmatter
        if lines.first().map(|l| l.trim()) == Some("---") {
            for (i, line) in lines.iter().enumerate().skip(1) {
                if line.trim() == "---" {
                    start_idx = i + 1;
                    break;
                }
            }
        }

        // Check if there is any content other than headings and whitespace
        for line in lines.iter().skip(start_idx) {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("# ") {
                continue;
            }
            return false;
        }

        true
    }

    /// Gathers comprehensive statistics across all notes in the configured folder.
    pub fn get_statistics(&self) -> Result<AppStatistics, String> {
        crate::services::statistics::get_statistics(self)
    }
}
