//! Note search functionality using fuzzy and exact matching.

use crate::models::response_types::SearchResult;
use nucleo_matcher::{Config, Matcher, Utf32Str};
use std::fs;

use super::note_manager::NoteManager;

/// Service for searching note content.
pub struct SearchService<'a> {
    note_manager: &'a NoteManager,
}

impl<'a> SearchService<'a> {
    /// Creates a new SearchService with the given NoteManager.
    pub fn new(note_manager: &'a NoteManager) -> Self {
        Self { note_manager }
    }

    /// Extracts frontmatter from file content.
    /// Returns (frontmatter_lines_count, frontmatter_string).
    fn extract_frontmatter(file_content: &str) -> (usize, String) {
        let lines: Vec<&str> = file_content.split('\n').collect();

        if lines.first().map(|l| l.trim()) != Some("---") {
            return (0, String::new());
        }

        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.trim() == "---" {
                let count = i + 1;
                let frontmatter = lines[..count].join("\n");
                return (count, frontmatter);
            }
        }

        (0, String::new())
    }

    /// Performs a fuzzy match using nucleo-matcher.
    /// Returns Some((score, indices)) if matched, None otherwise.
    fn fuzzy_match(
        matcher: &mut Matcher,
        line: &str,
        query: &Utf32Str,
        query_str: &str,
    ) -> Option<(i64, Vec<u32>)> {
        let trimmed_line = line.trim();

        if query_str.len() > trimmed_line.len() {
            return None;
        }

        let mut line_buf = Vec::new();
        let line_utf32 = Utf32Str::new(trimmed_line, &mut line_buf);

        let mut indices = Vec::new();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            matcher.fuzzy_indices(line_utf32, *query, &mut indices)
        }));

        match result {
            Ok(Some(score)) => {
                indices.sort_unstable();
                Some((score as i64, indices))
            }
            Ok(None) => None,
            Err(_) => None,
        }
    }

    /// Searches all notes in the notes folder for the given query.
    /// Returns a vector of SearchResult, sorted and truncated to 100 results.
    pub fn search(&self, query: &str, is_fuzzy: bool) -> Result<Vec<SearchResult>, String> {
        if query.trim().is_empty() {
            return Ok(vec![]);
        }

        let notes_folder = &self.note_manager.notes_folder;

        if !notes_folder.exists() {
            return Ok(vec![]);
        }

        let mut matcher = Matcher::new(Config::DEFAULT);
        let query_normalized = query.to_lowercase();
        let mut query_buf = Vec::new();
        let query_utf32 = Utf32Str::new(&query_normalized, &mut query_buf);
        let mut results: Vec<SearchResult> = Vec::new();

        let entries =
            fs::read_dir(notes_folder).map_err(|e| format!("Failed to read directory: {}", e))?;

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();
            if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }

            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let (frontmatter_len, _) = Self::extract_frontmatter(&content);
            let filename = entry.file_name().into_string().unwrap_or_default();
            let formatted_name = self.note_manager.format_note_name(&filename);

            for (i, line) in content.lines().enumerate().skip(frontmatter_len) {
                let stripped_line = crate::utils::markdown::strip_markdown(line);
                if stripped_line.is_empty() {
                    continue;
                }

                let match_data = if is_fuzzy {
                    Self::fuzzy_match(
                        &mut matcher,
                        &stripped_line,
                        &query_utf32,
                        &query_normalized,
                    )
                } else if let Some(pos) = stripped_line.to_lowercase().find(&query_normalized) {
                    // For exact match, we need to convert byte position to codepoint indices
                    let chars: Vec<char> = stripped_line.chars().collect();
                    let mut current_byte = 0;
                    let mut start_idx = 0;
                    for (idx, c) in chars.iter().enumerate() {
                        if current_byte == pos {
                            start_idx = idx;
                            break;
                        }
                        current_byte += c.len_utf8();
                    }

                    let query_chars_len = query_normalized.chars().count();
                    let indices: Vec<u32> = (start_idx..start_idx + query_chars_len)
                        .map(|idx| idx as u32)
                        .collect();
                    Some((0, indices))
                } else {
                    None
                };

                if let Some((score, indices)) = match_data {
                    let (excerpt, adjusted_indices) =
                        crate::utils::markdown::generate_excerpt(&stripped_line, &indices, 100);

                    results.push(SearchResult {
                        filename: filename.clone(),
                        formatted_name: formatted_name.clone(),
                        excerpt,
                        line_number: i,
                        score,
                        indices: adjusted_indices,
                    });
                }
            }
        }

        if is_fuzzy {
            results.sort_by(|a, b| b.score.cmp(&a.score));
        } else {
            results.sort_by(|a, b| b.filename.cmp(&a.filename));
        }

        results.truncate(100);

        Ok(results)
    }
}
