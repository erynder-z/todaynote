//! Note search functionality using fuzzy and exact matching.

use crate::models::response_types::{
    SearchResult, TagSearchResult, ThreadAggregationItem, ThreadAggregationResult,
    ThreadSearchResult,
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
                    crate::utils::text::generate_excerpt(&stripped, &indices, 100);
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

    /// Retrieves all markdown note files, sorted by filename descending.
    fn get_note_files(&self) -> Result<Vec<std::path::PathBuf>, String> {
        if !self.note_manager.notes_folder.exists() {
            return Ok(vec![]);
        }

        let entries = fs::read_dir(&self.note_manager.notes_folder)
            .map_err(|e| format!("Failed to read directory: {}", e))?;

        let mut files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
            .map(|e| e.path())
            .collect();

        files.sort_by(|a, b| {
            b.file_name()
                .unwrap_or_default()
                .cmp(a.file_name().unwrap_or_default())
        });

        Ok(files)
    }

    /// Extracts all unique thread names (H1 headings) from markdown content.
    fn extract_thread_names(content: &str) -> Vec<String> {
        let (frontmatter_len, _) = Self::extract_frontmatter(content);
        let mut names = std::collections::HashSet::new();

        for line in content.lines().skip(frontmatter_len) {
            if line.starts_with("# ") {
                let name = line[2..].trim().to_string();
                if !name.is_empty() {
                    names.insert(name);
                }
            }
        }
        names.into_iter().collect()
    }

    /// Extracts the content of a specific thread from markdown content.
    fn extract_thread_block(content: &str, thread_name: &str) -> Option<String> {
        let (frontmatter_len, _) = Self::extract_frontmatter(content);
        let lines: Vec<&str> = content.lines().collect();

        let mut thread_lines = Vec::new();
        let mut in_thread = false;

        for line in lines.iter().skip(frontmatter_len) {
            if line.starts_with("# ") {
                let name = line[2..].trim();
                if name == thread_name {
                    in_thread = true;
                    continue;
                } else if in_thread {
                    break;
                }
            }

            if in_thread {
                thread_lines.push(*line);
            }
        }

        if in_thread {
            // Trim trailing empty lines
            while thread_lines.last().map(|l| l.trim().is_empty()) == Some(true) {
                thread_lines.pop();
            }

            if !thread_lines.is_empty() {
                return Some(thread_lines.join("\n").trim().to_string());
            }
        }

        None
    }

    /// Filters and sorts a map of thread names based on the search query.
    fn filter_thread_results(
        &self,
        thread_counts: HashMap<String, usize>,
        query: &str,
        is_fuzzy: bool,
    ) -> Vec<ThreadSearchResult> {
        let mut results = Vec::new();

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
        results
    }

    /// Searches all notes in the notes folder for the given query.
    /// Returns up to 100 results, sorted by score (fuzzy) or filename (exact).
    pub fn search(&self, query: &str, is_fuzzy: bool) -> Result<Vec<SearchResult>, String> {
        if query.trim().is_empty() {
            return Ok(vec![]);
        }

        let mut matcher = Matcher::new(Config::DEFAULT);
        let query_normalized = query.to_lowercase();
        let mut query_buf = Vec::new();
        let query_utf32 = Utf32Str::new(&query_normalized, &mut query_buf);

        let files = self.get_note_files()?;

        let mut all_results: Vec<SearchResult> = files
            .iter()
            .flat_map(|path| {
                self.search_file(
                    path,
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
        let files = self.get_note_files()?;
        let mut thread_counts: HashMap<String, usize> = HashMap::new();

        for path in files {
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            for name in Self::extract_thread_names(&content) {
                *thread_counts.entry(name).or_insert(0) += 1;
            }
        }

        Ok(self.filter_thread_results(thread_counts, query, is_fuzzy))
    }

    /// Searches for unique tags across all notes.
    pub fn search_tags(&self, query: &str, is_fuzzy: bool) -> Result<Vec<TagSearchResult>, String> {
        let files = self.get_note_files()?;
        let mut tag_counts: HashMap<String, usize> = HashMap::new();

        for path in files {
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let tags = crate::utils::tag_parser::parse_tags_from_content(&content);
            for tag in tags {
                *tag_counts.entry(tag).or_insert(0) += 1;
            }
        }

        Ok(self.filter_tag_results(tag_counts, query, is_fuzzy))
    }

    /// Filters and sorts a map of tag names based on the search query.
    fn filter_tag_results(
        &self,
        tag_counts: HashMap<String, usize>,
        query: &str,
        is_fuzzy: bool,
    ) -> Vec<TagSearchResult> {
        let mut results = Vec::new();

        if query.is_empty() {
            results = tag_counts
                .into_iter()
                .map(|(name, count)| TagSearchResult {
                    name,
                    note_count: count,
                })
                .collect();
        } else {
            let mut matcher = Matcher::new(Config::DEFAULT);
            let query_normalized = query.to_lowercase();
            let mut query_buf = Vec::new();
            let query_utf32 = Utf32Str::new(&query_normalized, &mut query_buf);

            for (name, count) in tag_counts {
                let matched = if is_fuzzy {
                    Self::fuzzy_match(&mut matcher, &name, &query_utf32, &query_normalized)
                        .is_some()
                } else {
                    name.to_lowercase().contains(&query_normalized)
                };

                if matched {
                    results.push(TagSearchResult {
                        name,
                        note_count: count,
                    });
                }
            }
        }

        // Sort by note count descending, then by name
        results.sort_by(|a, b| b.note_count.cmp(&a.note_count).then(a.name.cmp(&b.name)));
        results
    }

    /// Finds all notes that contain a specific tag, optionally filtered by a query.
    pub fn search_notes_by_tag(
        &self,
        tag: &str,
        query: &str,
        is_fuzzy: bool,
    ) -> Result<Vec<SearchResult>, String> {
        let files = self.get_note_files()?;
        let mut results = Vec::new();

        let mut matcher = Matcher::new(Config::DEFAULT);
        let query_normalized = query.to_lowercase();
        let mut query_buf = Vec::new();
        let query_utf32 = Utf32Str::new(&query_normalized, &mut query_buf);

        for path in files {
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let tags = crate::utils::tag_parser::parse_tags_from_content(&content);
            if !tags.iter().any(|t| t.to_lowercase() == tag.to_lowercase()) {
                continue;
            }

            if query.trim().is_empty() {
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default();
                let formatted_name = self.note_manager.format_note_name(filename);

                let (fm_len, _) = Self::extract_frontmatter(&content);
                // Find first non-empty line after frontmatter
                let first_line = content
                    .lines()
                    .skip(fm_len)
                    .find(|l| !l.trim().is_empty())
                    .map(|l| l.trim().to_string())
                    .unwrap_or_else(|| formatted_name.clone());

                results.push(SearchResult {
                    filename: filename.to_string(),
                    formatted_name,
                    excerpt: first_line,
                    line_number: fm_len,
                    score: 0,
                    indices: vec![],
                });
            } else {
                results.extend(self.search_file(
                    &path,
                    is_fuzzy,
                    &query_utf32,
                    &query_normalized,
                    &mut matcher,
                ));
            }
        }

        if is_fuzzy && !query.trim().is_empty() {
            results.sort_by(|a, b| b.score.cmp(&a.score));
        } else {
            // Sort by filename descending (newest notes first)
            results.sort_by(|a, b| b.filename.cmp(&a.filename));
        }

        Ok(results)
    }

    /// Aggregates content from all threads matching the given name across all notes.
    pub fn aggregate_thread(&self, thread_name: &str) -> Result<ThreadAggregationResult, String> {
        let files = self.get_note_files()?;
        let mut items = Vec::new();

        for path in files {
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            if let Some(block_content) = Self::extract_thread_block(&content, thread_name) {
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default()
                    .to_string();

                items.push(ThreadAggregationItem {
                    filename: filename.clone(),
                    formatted_date: self.note_manager.format_note_name(&filename),
                    content: block_content,
                });
            }
        }

        Ok(ThreadAggregationResult {
            thread_name: thread_name.to_string(),
            items,
        })
    }
}
