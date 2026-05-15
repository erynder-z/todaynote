//! Note search functionality using fuzzy and exact matching.

use crate::models::response_types::{
    SearchResult, ThreadAggregationItem, ThreadAggregationResult, ThreadSearchResult,
};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use std::collections::HashMap;
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

    /// Searches a single note file for matches against the query.
    /// Returns all SearchResult matches found in the file.
    fn search_file(
        &self,
        path: &std::path::Path,
        is_fuzzy: bool,
        query_utf32: &Utf32Str,
        query_normalized: &str,
        matcher: &mut Matcher,
    ) -> Vec<SearchResult> {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let (frontmatter_len, _) = Self::extract_frontmatter(&content);
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        let formatted_name = self.note_manager.format_note_name(&filename);
        let mut results = Vec::new();

        for (i, line) in content.lines().enumerate().skip(frontmatter_len) {
            let stripped = crate::utils::markdown::strip_markdown(line);
            if stripped.is_empty() {
                continue;
            }

            if let Some((score, indices)) =
                self.find_match(matcher, query_utf32, query_normalized, is_fuzzy, &stripped)
            {
                let (excerpt, adjusted) =
                    crate::utils::markdown::generate_excerpt(&stripped, &indices, 100);
                results.push(SearchResult {
                    filename: filename.to_string(),
                    formatted_name: formatted_name.clone(),
                    excerpt,
                    line_number: i,
                    score,
                    indices: adjusted,
                });
            }
        }
        results
    }

    /// Finds match score and character indices for a line.
    fn find_match(
        &self,
        matcher: &mut Matcher,
        query_utf32: &Utf32Str,
        query_normalized: &str,
        is_fuzzy: bool,
        line: &str,
    ) -> Option<(i64, Vec<u32>)> {
        if is_fuzzy {
            Self::fuzzy_match(matcher, line, query_utf32, query_normalized)
        } else {
            Self::find_exact_match_indices(line, query_normalized)
        }
    }

    /// Finds character indices for an exact match.
    fn find_exact_match_indices(line: &str, query: &str) -> Option<(i64, Vec<u32>)> {
        line.to_lowercase().find(query).map(|pos| {
            let chars: Vec<char> = line.chars().collect();
            let mut current_byte = 0;
            let start_idx = chars
                .iter()
                .enumerate()
                .find_map(|(idx, c)| {
                    if current_byte == pos {
                        Some(idx)
                    } else {
                        current_byte += c.len_utf8();
                        None
                    }
                })
                .unwrap_or(0);
            let indices: Vec<u32> = (start_idx..start_idx + query.chars().count())
                .map(|i| i as u32)
                .collect();
            (0, indices)
        })
    }

    /// Searches all notes in the notes folder for the given query.
    /// Returns up to 100 results, sorted by score (fuzzy) or filename (exact).
    pub fn search(&self, query: &str, is_fuzzy: bool) -> Result<Vec<SearchResult>, String> {
        if query.trim().is_empty() || !self.note_manager.notes_folder.exists() {
            return Ok(vec![]);
        }

        let mut matcher = Matcher::new(Config::DEFAULT);
        let query_normalized = query.to_lowercase();
        let mut query_buf = Vec::new();
        let query_utf32 = Utf32Str::new(&query_normalized, &mut query_buf);

        let entries = fs::read_dir(&self.note_manager.notes_folder)
            .map_err(|e| format!("Failed to read directory: {}", e))?;

        let mut all_results: Vec<SearchResult> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
            .flat_map(|e| {
                self.search_file(
                    &e.path(),
                    is_fuzzy,
                    &query_utf32,
                    &query_normalized,
                    &mut matcher,
                )
            })
            .collect();

        if is_fuzzy {
            all_results.sort_by(|a, b| b.score.cmp(&a.score));
        } else {
            all_results.sort_by(|a, b| b.filename.cmp(&a.filename));
        }

        all_results.truncate(100);
        Ok(all_results)
    }

    /// Searches for unique thread names (H1 headings) across all notes.
    pub fn search_threads(
        &self,
        query: &str,
        is_fuzzy: bool,
    ) -> Result<Vec<ThreadSearchResult>, String> {
        if !self.note_manager.notes_folder.exists() {
            return Ok(vec![]);
        }

        let mut thread_counts: HashMap<String, usize> = HashMap::new();
        let entries = fs::read_dir(&self.note_manager.notes_folder)
            .map_err(|e| format!("Failed to read directory: {}", e))?;

        for entry in entries.filter_map(|e| e.ok()) {
            if entry.path().extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }

            let content = match fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let (frontmatter_len, _) = Self::extract_frontmatter(&content);
            let mut seen_in_file = std::collections::HashSet::new();

            for line in content.lines().skip(frontmatter_len) {
                if line.starts_with("# ") {
                    let name = line[2..].trim().to_string();
                    if !name.is_empty() {
                        seen_in_file.insert(name);
                    }
                }
            }

            for name in seen_in_file {
                *thread_counts.entry(name).or_insert(0) += 1;
            }
        }

        let mut results: Vec<ThreadSearchResult> = Vec::new();

        if query.is_empty() {
            results = thread_counts
                .into_iter()
                .map(|(name, count)| ThreadSearchResult {
                    name,
                    note_count: count,
                })
                .collect();
        } else {
            let mut matcher = Matcher::new(Config::DEFAULT);
            let query_normalized = query.to_lowercase();
            let mut query_buf = Vec::new();
            let query_utf32 = Utf32Str::new(&query_normalized, &mut query_buf);

            for (name, count) in thread_counts {
                let matched = if is_fuzzy {
                    Self::fuzzy_match(&mut matcher, &name, &query_utf32, &query_normalized)
                        .is_some()
                } else {
                    name.to_lowercase().contains(&query_normalized)
                };

                if matched {
                    results.push(ThreadSearchResult {
                        name,
                        note_count: count,
                    });
                }
            }
        }

        // Sort by note count descending, then by name
        results.sort_by(|a, b| b.note_count.cmp(&a.note_count).then(a.name.cmp(&b.name)));

        Ok(results)
    }

    /// Aggregates content from all threads matching the given name across all notes.
    pub fn aggregate_thread(&self, thread_name: &str) -> Result<ThreadAggregationResult, String> {
        if !self.note_manager.notes_folder.exists() {
            return Err("Notes folder does not exist".to_string());
        }

        let mut items = Vec::new();
        let entries = fs::read_dir(&self.note_manager.notes_folder)
            .map_err(|e| format!("Failed to read directory: {}", e))?;

        // Collect matching files and sort them (assuming filename is date YYYY-MM-DD)
        let mut files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
            .collect();

        // Sort by filename descending (newest first)
        files.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

        for entry in files {
            let content = match fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let filename = entry
                .file_name()
                .into_string()
                .unwrap_or_else(|_| "".to_string());
            let formatted_date = self.note_manager.format_note_name(&filename);

            let (frontmatter_len, _) = Self::extract_frontmatter(&content);
            let lines: Vec<&str> = content.lines().collect();

            let mut thread_content = Vec::new();
            let mut in_thread = false;

            for line in lines.iter().skip(frontmatter_len) {
                if line.starts_with("# ") {
                    let name = line[2..].trim();
                    if name == thread_name {
                        in_thread = true;
                        continue;
                    } else if in_thread {
                        // Reached another H1, end extraction
                        break;
                    }
                }

                if in_thread {
                    thread_content.push(*line);
                }
            }

            if in_thread {
                // Trim trailing empty lines
                while thread_content.last().map(|l| l.trim().is_empty()) == Some(true) {
                    thread_content.pop();
                }

                if !thread_content.is_empty() {
                    items.push(ThreadAggregationItem {
                        filename,
                        formatted_date,
                        content: thread_content.join("\n").trim().to_string(),
                    });
                }
            }
        }

        Ok(ThreadAggregationResult {
            thread_name: thread_name.to_string(),
            items,
        })
    }
}
