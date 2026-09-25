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
    /// The note is initialized with YAML metadata. If the note header is not empty, add it below the metadata
    pub fn create_todays_note(&self, note_header: &str) -> Result<PathBuf, String> {
        self.ensure_notes_folder_exists()?;
        let file_path = self.get_today_note_path();

        if self.todays_note_exists() {
            return Ok(file_path);
        }

        let current_date = utils::date::get_current_date();
        let note_content = if note_header.is_empty() {
            format!(
                "---\ncreated: {}\nlast-modified: {}\ntags: []\n---\n",
                current_date, current_date
            )
        } else {
            format!(
                "---\ncreated: {}\nlast-modified: {}\ntags: []\n---\n!!! {}\n",
                current_date, current_date, note_header
            )
        };

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
            word_count: crate::utils::text::count_words(&content).0,
            has_code: crate::utils::text::count_words(&content).1,
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

    /// Extracts the first N thread names (lines starting with !!!) from the content.
    pub fn extract_threads(&self, content: &str, limit: usize) -> Vec<String> {
        content
            .lines()
            .filter(|l| l.starts_with("!!! "))
            .take(limit)
            .map(|l| l[4..].trim().to_string())
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
            let stripped = crate::utils::markdown::strip_markdown_line(line);
            if stripped.is_empty() {
                continue;
            }
            preview_text.push(stripped);
            if preview_text.len() > 3 {
                break;
            }
        }

        let joined = preview_text.join(" ");
        let (preview, _) = crate::utils::text::generate_excerpt(&joined, &[], 150);
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

    /// Deletes all notes that have no content (only frontmatter, headings, and thread markers).
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

    /// Determines if a note is considered "empty" (no tags and no content beyond headings and thread markers).
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
            if trimmed.is_empty() || trimmed.starts_with("# ") || trimmed.starts_with("!!! ") {
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

///
/// Unit Tests
///
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    ///
    /// NoteManager Construction and Configuration Tests
    ///
    #[test]
    fn test_new_creates_manager_with_correct_fields() {
        let notes_folder = PathBuf::from("/tmp/notes");
        let locale = "en".to_string();

        let manager = NoteManager::new(notes_folder.clone(), locale.clone());

        assert_eq!(manager.notes_folder, notes_folder);
        assert_eq!(manager.locale, locale);
    }

    #[test]
    fn test_update_config_changes_fields() {
        let mut manager = NoteManager::new(PathBuf::from("/tmp/notes"), "en".to_string());

        let new_folder = PathBuf::from("/tmp/new_notes");
        let new_locale = "de".to_string();
        manager.update_config(new_folder.clone(), new_locale.clone());

        assert_eq!(manager.notes_folder, new_folder);
        assert_eq!(manager.locale, new_locale);
    }

    ///
    /// Folder and Path Tests
    ///
    #[test]
    fn test_ensure_notes_folder_exists_creates_directory() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().join("notes");

        let manager = NoteManager::new(notes_path.clone(), "en".to_string());

        assert!(!notes_path.exists());
        manager
            .ensure_notes_folder_exists()
            .expect("Failed to create folder");
        assert!(notes_path.exists());
        assert!(notes_path.is_dir());
    }

    #[test]
    fn test_ensure_notes_folder_exists_no_error_if_exists() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().join("notes");
        fs::create_dir_all(&notes_path).expect("Failed to pre-create folder");

        let manager = NoteManager::new(notes_path.clone(), "en".to_string());

        let result = manager.ensure_notes_folder_exists();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_today_note_path_returns_correct_path() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();

        let manager = NoteManager::new(notes_path.clone(), "en".to_string());
        let today_path = manager.get_today_note_path();

        let current_date = crate::utils::date::get_current_date();
        let expected_path = notes_path.join(format!("{}.md", current_date));

        assert_eq!(today_path, expected_path);
    }

    #[test]
    fn test_todays_note_exists_returns_false_when_not_exists() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();

        let manager = NoteManager::new(notes_path, "en".to_string());

        assert!(!manager.todays_note_exists());
    }

    #[test]
    fn test_todays_note_exists_returns_true_when_exists() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();
        let today_path = notes_path.join(format!("{}.md", crate::utils::date::get_current_date()));

        fs::create_dir_all(&notes_path).expect("Failed to create notes dir");
        fs::write(&today_path, "---\n").expect("Failed to create today's note");

        let manager = NoteManager::new(notes_path, "en".to_string());

        assert!(manager.todays_note_exists());
    }

    ///
    /// Note Creation Tests
    ///
    #[test]
    fn test_create_todays_note_creates_file_with_empty_header() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();

        let manager = NoteManager::new(notes_path.clone(), "en".to_string());

        let result = manager.create_todays_note("");
        assert!(result.is_ok());

        let file_path = result.unwrap();
        assert!(file_path.exists());

        let content = fs::read_to_string(&file_path).expect("Failed to read created note");
        let current_date = crate::utils::date::get_current_date();
        let expected = format!(
            "---\ncreated: {}\nlast-modified: {}\ntags: []\n---\n",
            current_date, current_date
        );
        assert_eq!(content, expected);
    }

    #[test]
    fn test_create_todays_note_creates_file_with_header() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();

        let manager = NoteManager::new(notes_path.clone(), "en".to_string());

        let result = manager.create_todays_note("My Thread");
        assert!(result.is_ok());

        let file_path = result.unwrap();
        let content = fs::read_to_string(&file_path).expect("Failed to read created note");
        let current_date = crate::utils::date::get_current_date();
        let expected = format!(
            "---\ncreated: {}\nlast-modified: {}\ntags: []\n---\n!!! My Thread\n",
            current_date, current_date
        );
        assert_eq!(content, expected);
    }

    #[test]
    fn test_create_todays_note_does_not_overwrite_existing() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();
        let today_path = notes_path.join(format!("{}.md", crate::utils::date::get_current_date()));

        fs::create_dir_all(&notes_path).expect("Failed to create notes dir");
        fs::write(&today_path, "existing content\n").expect("Failed to write existing note");

        let manager = NoteManager::new(notes_path.clone(), "en".to_string());

        let result = manager.create_todays_note("New Header");
        assert!(result.is_ok());

        let content = fs::read_to_string(&today_path).expect("Failed to read note");
        assert_eq!(content, "existing content\n");
    }

    #[test]
    fn test_create_todays_note_creates_folder_if_not_exists() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().join("notes");

        assert!(!notes_path.exists());

        let manager = NoteManager::new(notes_path.clone(), "en".to_string());
        let result = manager.create_todays_note("");
        assert!(result.is_ok());

        assert!(notes_path.exists());
        assert!(notes_path.is_dir());
    }

    ///
    /// File Listing Tests
    ///
    #[test]
    fn test_get_sorted_note_files_returns_empty_for_empty_dir() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();

        fs::create_dir_all(&notes_path).expect("Failed to create notes dir");

        let manager = NoteManager::new(notes_path, "en".to_string());
        let result = manager.get_sorted_note_files();

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_get_sorted_note_files_returns_empty_for_nonexistent_dir() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().join("nonexistent");

        let manager = NoteManager::new(notes_path, "en".to_string());
        let result = manager.get_sorted_note_files();

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_get_sorted_note_files_filters_markdown_files() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();

        fs::create_dir_all(&notes_path).expect("Failed to create notes dir");
        fs::write(notes_path.join("2024-01-01.md"), "# Note 1").expect("Failed to write file");
        fs::write(notes_path.join("2024-01-02.md"), "# Note 2").expect("Failed to write file");
        fs::write(notes_path.join("readme.txt"), "not a markdown file")
            .expect("Failed to write file");
        fs::write(notes_path.join(".hidden.md"), "hidden").expect("Failed to write file");

        let manager = NoteManager::new(notes_path, "en".to_string());
        let result = manager.get_sorted_note_files().unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.contains(&"2024-01-01.md".to_string()));
        assert!(result.contains(&"2024-01-02.md".to_string()));
        assert!(!result.contains(&"readme.txt".to_string()));
        assert!(!result.contains(&".hidden.md".to_string()));
    }

    #[test]
    fn test_get_sorted_note_files_sorts_descending() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();

        fs::create_dir_all(&notes_path).expect("Failed to create notes dir");
        fs::write(notes_path.join("2024-01-01.md"), "# Note 1").expect("Failed to write file");
        fs::write(notes_path.join("2024-01-02.md"), "# Note 2").expect("Failed to write file");
        fs::write(notes_path.join("2024-01-03.md"), "# Note 3").expect("Failed to write file");

        let manager = NoteManager::new(notes_path, "en".to_string());
        let result = manager.get_sorted_note_files().unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "2024-01-03.md");
        assert_eq!(result[1], "2024-01-02.md");
        assert_eq!(result[2], "2024-01-01.md");
    }

    ///
    /// Thread Extraction Tests
    ///
    #[test]
    fn test_extract_threads_empty_content() {
        let manager = NoteManager::new(PathBuf::from("/tmp"), "en".to_string());
        let result = manager.extract_threads("", 5);
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_threads_no_threads() {
        let manager = NoteManager::new(PathBuf::from("/tmp"), "en".to_string());
        let content = "This is just regular content\nWith multiple lines\nNo threads here";
        let result = manager.extract_threads(content, 5);
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_threads_single_thread() {
        let manager = NoteManager::new(PathBuf::from("/tmp"), "en".to_string());
        let content = "!!! Main Thread\nSome content";
        let result = manager.extract_threads(content, 5);
        assert_eq!(result, vec!["Main Thread"]);
    }

    #[test]
    fn test_extract_threads_multiple_threads() {
        let manager = NoteManager::new(PathBuf::from("/tmp"), "en".to_string());
        let content = "!!! Thread 1\n!!! Thread 2\n!!! Thread 3";
        let result = manager.extract_threads(content, 5);
        assert_eq!(result, vec!["Thread 1", "Thread 2", "Thread 3"]);
    }

    #[test]
    fn test_extract_threads_respects_limit() {
        let manager = NoteManager::new(PathBuf::from("/tmp"), "en".to_string());
        let content =
            "!!! Thread 1\n!!! Thread 2\n!!! Thread 3\n!!! Thread 4\n!!! Thread 5\n!!! Thread 6";
        let result = manager.extract_threads(content, 3);
        assert_eq!(result, vec!["Thread 1", "Thread 2", "Thread 3"]);
    }

    #[test]
    fn test_extract_threads_skips_empty_threads() {
        let manager = NoteManager::new(PathBuf::from("/tmp"), "en".to_string());
        let content = "!!! \n!!! Valid\n!!!   \n!!! Also Valid";
        let result = manager.extract_threads(content, 5);
        assert_eq!(result, vec!["Valid", "Also Valid"]);
    }

    #[test]
    fn test_extract_threads_trims_whitespace() {
        let manager = NoteManager::new(PathBuf::from("/tmp"), "en".to_string());
        let content = "!!!   Thread with spaces   \n!!! Another thread";
        let result = manager.extract_threads(content, 5);
        assert_eq!(result, vec!["Thread with spaces", "Another thread"]);
    }

    ///
    /// Preview Extraction Tests
    ///
    #[test]
    fn test_extract_preview_empty_content() {
        let manager = NoteManager::new(PathBuf::from("/tmp"), "en".to_string());
        let result = manager.extract_preview("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_preview_skips_frontmatter() {
        let manager = NoteManager::new(PathBuf::from("/tmp"), "en".to_string());
        let content = "---\ntitle: Test\n---\nThis is the preview text";
        let result = manager.extract_preview(content);
        assert!(result.contains("preview text"));
        assert!(!result.contains("title: Test"));
    }

    #[test]
    fn test_extract_preview_takes_first_few_lines() {
        let manager = NoteManager::new(PathBuf::from("/tmp"), "en".to_string());
        let content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6";
        let result = manager.extract_preview(content);
        // Should take first 4 non-empty lines (up to limit of 3 + 1)
        assert!(result.contains("Line 1"));
        assert!(result.contains("Line 2"));
        assert!(result.contains("Line 3"));
    }

    #[test]
    fn test_extract_preview_handles_markdown() {
        let manager = NoteManager::new(PathBuf::from("/tmp"), "en".to_string());
        let content = "---\ntags: []\n---\n# Heading\n**Bold text** and *italic text*";
        let result = manager.extract_preview(content);
        // strip_markdown_line should remove markdown formatting
        assert!(!result.contains("**"));
        assert!(!result.contains("*"));
        assert!(!result.contains("#"));
    }

    ///
    /// Note Name Formatting Tests
    ///
    #[test]
    fn test_format_note_name_date_file_en_locale() {
        let manager = NoteManager::new(PathBuf::from("/tmp"), "en".to_string());
        let result = manager.format_note_name("2024-01-15.md");
        // Should be formatted as localized date
        assert!(!result.contains("2024-01-15"));
        assert!(result.contains("Monday") || result.contains("January") || result.contains("15"));
    }

    #[test]
    fn test_format_note_name_date_file_de_locale() {
        let manager = NoteManager::new(PathBuf::from("/tmp"), "de".to_string());
        let result = manager.format_note_name("2024-01-15.md");
        // German locale should have German month/day names
        assert!(!result.contains("2024-01-15"));
    }

    #[test]
    fn test_format_note_name_date_file_ja_locale() {
        let manager = NoteManager::new(PathBuf::from("/tmp"), "ja".to_string());
        let result = manager.format_note_name("2024-01-15.md");
        // Japanese locale should have Japanese format
        assert!(!result.contains("2024-01-15"));
        assert!(result.contains("年") || result.contains("月") || result.contains("日"));
    }

    #[test]
    fn test_format_note_name_non_date_file() {
        let manager = NoteManager::new(PathBuf::from("/tmp"), "en".to_string());
        let result = manager.format_note_name("my-custom-note.md");
        assert_eq!(result, "my-custom-note");
    }

    #[test]
    fn test_format_note_name_invalid_date() {
        let manager = NoteManager::new(PathBuf::from("/tmp"), "en".to_string());
        let result = manager.format_note_name("not-a-date.md");
        assert_eq!(result, "not-a-date");
    }

    ///
    /// Note Reading Tests
    ///
    #[test]
    fn test_read_note_content_success() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();
        let file_path = notes_path.join("test.md");

        fs::create_dir_all(&notes_path).expect("Failed to create notes dir");
        fs::write(&file_path, "Test content").expect("Failed to write file");

        let manager = NoteManager::new(notes_path, "en".to_string());
        let result = manager.read_note_content(&file_path);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Test content");
    }

    #[test]
    fn test_read_note_content_not_found() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();
        let file_path = notes_path.join("nonexistent.md");

        let manager = NoteManager::new(notes_path, "en".to_string());
        let result = manager.read_note_content(&file_path);

        assert!(result.is_err());
    }

    ///
    /// List Notes Tests
    ///
    #[test]
    fn test_list_notes_empty_directory() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();

        fs::create_dir_all(&notes_path).expect("Failed to create notes dir");

        let manager = NoteManager::new(notes_path, "en".to_string());
        let result = manager.list_notes(None);

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.total_count, 0);
        assert!(response.notes.is_empty());
    }

    #[test]
    fn test_list_notes_with_limit() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();

        fs::create_dir_all(&notes_path).expect("Failed to create notes dir");
        fs::write(
            notes_path.join("2024-01-01.md"),
            "---\ntags: []\n---\n# Note 1",
        )
        .expect("Failed to write file");
        fs::write(
            notes_path.join("2024-01-02.md"),
            "---\ntags: []\n---\n# Note 2",
        )
        .expect("Failed to write file");
        fs::write(
            notes_path.join("2024-01-03.md"),
            "---\ntags: []\n---\n# Note 3",
        )
        .expect("Failed to write file");

        let manager = NoteManager::new(notes_path, "en".to_string());
        let result = manager.list_notes(Some(2));

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.total_count, 3);
        assert_eq!(response.notes.len(), 2);
    }

    #[test]
    fn test_list_notes_formatted_name() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();

        fs::create_dir_all(&notes_path).expect("Failed to create notes dir");
        let date = crate::utils::date::get_current_date();
        fs::write(
            notes_path.join(format!("{}.md", date)),
            "---\ntags: []\n---\n# Today",
        )
        .expect("Failed to write file");

        let manager = NoteManager::new(notes_path, "en".to_string());
        let result = manager.list_notes(None);

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.notes.len(), 1);
        // The formatted name should be a localized date, not the raw filename
        assert_ne!(response.notes[0].formatted_name, format!("{}.md", date));
    }

    ///
    /// Empty Note Detection Tests
    ///
    #[test]
    fn test_is_note_empty_with_only_frontmatter() {
        let manager = NoteManager::new(PathBuf::from("/tmp"), "en".to_string());
        let content = "---\ncreated: 2024-01-01\ntags: []\n---\n";
        assert!(manager.is_note_empty(content));
    }

    #[test]
    fn test_is_note_empty_with_headings() {
        let manager = NoteManager::new(PathBuf::from("/tmp"), "en".to_string());
        // Only level 1 headings (starting with "# ") are skipped
        let content = "---\ntags: []\n---\n# Heading 1\n# Heading 2\n";
        assert!(manager.is_note_empty(content));
    }

    #[test]
    fn test_is_note_empty_with_thread_markers() {
        let manager = NoteManager::new(PathBuf::from("/tmp"), "en".to_string());
        let content = "---\ntags: []\n---\n!!! Thread 1\n!!! Thread 2\n";
        assert!(manager.is_note_empty(content));
    }

    #[test]
    fn test_is_note_empty_with_tags_not_empty() {
        let manager = NoteManager::new(PathBuf::from("/tmp"), "en".to_string());
        let content = "---\ntags: [work]\n---\n";
        assert!(!manager.is_note_empty(content));
    }

    #[test]
    fn test_is_note_empty_with_content_not_empty() {
        let manager = NoteManager::new(PathBuf::from("/tmp"), "en".to_string());
        let content = "---\ntags: []\n---\n# Heading\nSome actual content here";
        assert!(!manager.is_note_empty(content));
    }

    #[test]
    fn test_is_note_empty_with_whitespace_only() {
        let manager = NoteManager::new(PathBuf::from("/tmp"), "en".to_string());
        let content = "---\ntags: []\n---\n   \n\n  \n";
        assert!(manager.is_note_empty(content));
    }

    ///
    /// Purge Empty Notes Tests
    ///
    #[test]
    fn test_purge_empty_notes_removes_empty_notes() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();

        fs::create_dir_all(&notes_path).expect("Failed to create notes dir");
        // Empty note
        fs::write(notes_path.join("empty.md"), "---\ntags: []\n---\n")
            .expect("Failed to write file");
        // Non-empty note
        fs::write(
            notes_path.join("nonempty.md"),
            "---\ntags: []\n---\nContent here",
        )
        .expect("Failed to write file");

        let manager = NoteManager::new(notes_path.clone(), "en".to_string());
        let result = manager.purge_empty_notes();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
        assert!(!notes_path.join("empty.md").exists());
        assert!(notes_path.join("nonempty.md").exists());
    }

    #[test]
    fn test_purge_empty_notes_with_tags_not_removed() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();

        fs::create_dir_all(&notes_path).expect("Failed to create notes dir");
        // Note with tags but no content
        fs::write(notes_path.join("tagged.md"), "---\ntags: [work]\n---\n")
            .expect("Failed to write file");

        let manager = NoteManager::new(notes_path.clone(), "en".to_string());
        let result = manager.purge_empty_notes();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert!(notes_path.join("tagged.md").exists());
    }

    #[test]
    fn test_purge_empty_notes_empty_directory() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();

        fs::create_dir_all(&notes_path).expect("Failed to create notes dir");

        let manager = NoteManager::new(notes_path, "en".to_string());
        let result = manager.purge_empty_notes();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    ///
    /// Edge Cases
    ///
    #[test]
    fn test_format_note_name_without_extension() {
        let manager = NoteManager::new(PathBuf::from("/tmp"), "en".to_string());
        let result = manager.format_note_name("custom-note");
        // Non-date filename is returned as-is without extension
        assert_eq!(result, "custom-note");
    }

    #[test]
    fn test_extract_threads_mixed_with_other_content() {
        let manager = NoteManager::new(PathBuf::from("/tmp"), "en".to_string());
        let content = "# Heading\n!!! Thread 1\nSome text\n!!! Thread 2\nMore text";
        let result = manager.extract_threads(content, 5);
        assert_eq!(result, vec!["Thread 1", "Thread 2"]);
    }

    #[test]
    fn test_get_sorted_note_files_handles_read_dir_error() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().join("nonexistent");

        let manager = NoteManager::new(notes_path, "en".to_string());
        let result = manager.get_sorted_note_files();

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
