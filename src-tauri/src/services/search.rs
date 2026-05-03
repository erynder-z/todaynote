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

    /// Strips common markdown characters from a line for cleaner search and display.
    fn strip_markdown(line: &str) -> String {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        // 1. Strip leading markers
        let mut s = trimmed;

        // Headings (e.g., ### Title)
        if s.starts_with('#') {
            s = s.trim_start_matches('#').trim_start();
        }

        // Blockquotes (e.g., > Quote)
        if s.starts_with("> ") {
            s = &s[2..];
        }

        // Tasks and Lists (e.g., - [ ] Task, * Item)
        if s.starts_with("- [ ] ") || s.starts_with("- [x] ") {
            s = &s[6..];
        } else if s.starts_with("- ") || s.starts_with("* ") || s.starts_with("+ ") {
            s = &s[2..];
        }

        // 2. Strip inline markers (basic)
        // We remove longer markers first to avoid leaving orphans
        let mut result = s
            .replace("***", "")
            .replace("___", "")
            .replace("**", "")
            .replace("__", "")
            .replace("~~", "")
            .replace("`", "")
            .replace("*", "")
            .replace("_", "");

        // 3. Strip trailing line-break backslashes
        if result.ends_with('\\') {
            result.pop();
        }

        result.trim().to_string()
    }

    /// Generates an excerpt from a matching line, centered around the matching indices.
    /// Returns (excerpt_string, adjusted_indices).
    fn generate_excerpt(line: &str, indices: &[u32], max_length: usize) -> (String, Vec<u32>) {
        let chars: Vec<char> = line.chars().collect();
        if chars.len() <= max_length {
            return (line.to_string(), indices.to_vec());
        }

        let first = *indices.first().unwrap_or(&0) as usize;
        let last = *indices.last().unwrap_or(&(chars.len() as u32 - 1)) as usize;
        let match_len = last - first + 1;

        let start = if match_len >= max_length {
            first
        } else {
            let surplus = max_length - match_len;
            first.saturating_sub(surplus / 2)
        };
        let end = (start + max_length).min(chars.len());
        let start = end.saturating_sub(max_length);

        let excerpt: String = chars[start..end].iter().collect();
        let mut adjusted_indices = Vec::new();
        for &idx in indices {
            let idx_usize = idx as usize;
            if idx_usize >= start && idx_usize < end {
                adjusted_indices.push((idx_usize - start) as u32);
            }
        }

        let mut final_excerpt = String::new();
        if start > 0 {
            final_excerpt.push_str("...");
        }
        final_excerpt.push_str(&excerpt);
        if end < chars.len() {
            final_excerpt.push_str("...");
        }

        // Adjust indices for the "..." prefix
        let prefix_offset = if start > 0 { 3 } else { 0 };
        let final_indices = adjusted_indices
            .into_iter()
            .map(|i| i + prefix_offset)
            .collect();

        (final_excerpt, final_indices)
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
                let stripped_line = Self::strip_markdown(line);
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
                        Self::generate_excerpt(&stripped_line, &indices, 100);

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
