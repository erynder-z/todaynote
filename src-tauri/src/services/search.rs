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

    /// Generates an excerpt from a matching line, centered around the query.
    fn generate_excerpt(line: &str, query: &str, max_length: usize) -> String {
        let trimmed_line = line.trim();
        let query_normalized = query.to_lowercase();

        if trimmed_line.len() <= max_length {
            return trimmed_line.to_string();
        }

        if let Some(pos) = trimmed_line.to_lowercase().find(&query_normalized) {
            let start = pos.saturating_sub(40);
            let end = (pos + query_normalized.len() + 40).min(trimmed_line.len());
            format!(
                "{}{}{}",
                if start > 0 { "..." } else { "" },
                &trimmed_line[start..end],
                if end < trimmed_line.len() { "..." } else { "" }
            )
        } else {
            format!("{}...", &trimmed_line[..max_length.min(trimmed_line.len())])
        }
    }

    /// Performs a fuzzy match using nucleo-matcher.
    /// Returns Some(score) if matched, None otherwise.
    /// Wrapped in panic catch for nucleo-matcher stability.
    fn fuzzy_match(
        matcher: &mut Matcher,
        line: &str,
        query: &Utf32Str,
        query_str: &str,
    ) -> Option<i64> {
        let trimmed_line = line.trim();

        // Guard against nucleo-matcher panic: don't match if query is longer than line
        if query_str.len() > trimmed_line.len() {
            return None;
        }

        let mut line_buf = Vec::new();
        let line_utf32 = Utf32Str::new(trimmed_line, &mut line_buf);

        // CRITICAL SAFETY: nucleo-matcher has known internal panics.
        // We wrap the call in catch_unwind to ensure a library bug doesn't kill our app.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            matcher.fuzzy_match(line_utf32, *query)
        }));

        match result {
            Ok(Some(score)) => Some(score as i64),
            Ok(None) => None,
            Err(_) => None, // If it panics, we just skip this line
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

            // Iterate lines directly to avoid large Vec allocation
            for (i, line) in content.lines().enumerate().skip(frontmatter_len) {
                let trimmed_line = line.trim();
                if trimmed_line.is_empty() {
                    continue;
                }

                let (matched, score) = if is_fuzzy {
                    match Self::fuzzy_match(
                        &mut matcher,
                        trimmed_line,
                        &query_utf32,
                        &query_normalized,
                    ) {
                        Some(s) => (true, s),
                        None => (false, 0),
                    }
                } else {
                    (trimmed_line.to_lowercase().contains(&query_normalized), 0)
                };

                if matched {
                    let excerpt = Self::generate_excerpt(trimmed_line, &query_normalized, 100);

                    results.push(SearchResult {
                        filename: filename.clone(),
                        formatted_name: formatted_name.clone(),
                        excerpt,
                        line_number: i,
                        score,
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
