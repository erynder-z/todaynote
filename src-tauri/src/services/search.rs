//! Note search functionality using fuzzy and exact matching.

use crate::models::response_types::{
    PinnedThreadItem, SearchResult, TagSearchResult, ThreadAggregationItem,
    ThreadAggregationResult, ThreadSearchResult,
};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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

    /// Returns the number of lines occupied by the YAML frontmatter block
    /// (0 when the content has no closed frontmatter block). Allocation-free,
    /// for use on hot paths that only need the body start offset.
    fn frontmatter_line_count(content: &str) -> usize {
        let mut lines = content.lines();
        if lines.next().map(|l| l.trim()) != Some("---") {
            return 0;
        }
        for (i, line) in lines.enumerate() {
            if line.trim() == "---" {
                // `lines.next()` above consumed line 0, so the closing fence
                // is at line index i + 1 and the block covers i + 2 lines.
                return i + 2;
            }
        }
        0
    }

    /// Extracts frontmatter from file content.
    /// Returns (frontmatter_lines_count, frontmatter_string).
    fn extract_frontmatter(file_content: &str) -> (usize, String) {
        let count = Self::frontmatter_line_count(file_content);
        if count == 0 {
            return (0, String::new());
        }
        let frontmatter = file_content
            .lines()
            .take(count)
            .collect::<Vec<_>>()
            .join("\n");
        (count, frontmatter)
    }

    /// Performs a fuzzy match using nucleo-matcher.
    /// Returns Some((score, indices)) if matched, None otherwise.
    ///
    /// `buf` is a reusable scratch buffer for the line's UTF-32 encoding
    /// (`Utf32Str::new` clears it when needed), avoiding a per-line
    /// allocation. Panics from the matcher are caught once per file by the
    /// caller, not per line.
    fn fuzzy_match(
        matcher: &mut Matcher,
        line: &str,
        query: &Utf32Str,
        query_str: &str,
        buf: &mut Vec<char>,
    ) -> Option<(i64, Vec<u32>)> {
        let trimmed_line = line.trim();

        if query_str.len() > trimmed_line.len() {
            return None;
        }

        let line_utf32 = Utf32Str::new(trimmed_line, buf);

        let mut indices = Vec::new();
        match matcher.fuzzy_indices(line_utf32, *query, &mut indices) {
            Some(score) => {
                indices.sort_unstable();
                Some((score as i64, indices))
            }
            None => None,
        }
    }

    /// Searches a single note file for matches against the query.
    /// Returns all SearchResult matches found in the file.
    ///
    /// `line_buf` is a reusable scratch buffer for UTF-32 encoding. A panic
    /// inside the matcher aborts only this file; results collected so far are
    /// kept and the matcher is reset by the caller before the next file.
    fn search_file(
        note_manager: &NoteManager,
        path: &Path,
        is_fuzzy: bool,
        query_utf32: &Utf32Str,
        query_normalized: &str,
        matcher: &mut Matcher,
        line_buf: &mut Vec<char>,
    ) -> Vec<SearchResult> {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let frontmatter_len = Self::frontmatter_line_count(&content);
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        let filename_owned = filename.to_string();
        let formatted_name = note_manager.format_note_name(filename);
        let created = note_manager.extract_created_date(&content);
        let mut results = Vec::new();

        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            for (i, line) in content.lines().enumerate().skip(frontmatter_len) {
                let stripped = crate::utils::markdown::strip_markdown_line(line);
                if stripped.is_empty() {
                    continue;
                }

                if let Some((score, indices)) = Self::find_match(
                    matcher,
                    query_utf32,
                    query_normalized,
                    is_fuzzy,
                    &stripped,
                    line_buf,
                ) {
                    let (excerpt, adjusted) =
                        crate::utils::text::generate_excerpt(&stripped, &indices, 100);
                    results.push(SearchResult {
                        filename: filename_owned.clone(),
                        formatted_name: formatted_name.clone(),
                        created: created.clone(),
                        excerpt,
                        line_number: i,
                        score,
                        indices: adjusted,
                    });
                }
            }
        }));
        if outcome.is_err() {
            // The matcher may be in an inconsistent state after a panic;
            // reset it before it is reused for the next file.
            *matcher = Matcher::new(Config::DEFAULT);
        }
        results
    }

    /// Finds match score and character indices for a line.
    fn find_match(
        matcher: &mut Matcher,
        query_utf32: &Utf32Str,
        query_normalized: &str,
        is_fuzzy: bool,
        line: &str,
        line_buf: &mut Vec<char>,
    ) -> Option<(i64, Vec<u32>)> {
        if is_fuzzy {
            Self::fuzzy_match(matcher, line, query_utf32, query_normalized, line_buf)
        } else {
            Self::find_exact_match_indices(line, query_normalized)
        }
    }

    /// Finds character indices for an exact match.
    fn find_exact_match_indices(line: &str, query: &str) -> Option<(i64, Vec<u32>)> {
        let pos = line.to_lowercase().find(query)?;
        // Map the byte position in the lowercased string back to a char index
        // in the original line. `get` returns None when lowercasing changed
        // the byte layout before the match (multibyte expansion); fall back
        // to 0 like the previous implementation did.
        let start_idx = line
            .get(..pos)
            .map(|prefix| prefix.chars().count())
            .unwrap_or(0);
        let indices: Vec<u32> = (start_idx..start_idx + query.chars().count())
            .map(|i| i as u32)
            .collect();
        Some((0, indices))
    }

    /// Retrieves all markdown note files, sorted by filename descending.
    /// Dotfiles are skipped, consistent with `NoteManager::get_sorted_note_files`.
    fn get_note_files(&self) -> Result<Vec<PathBuf>, String> {
        let files = self
            .note_manager
            .get_sorted_note_files()?
            .into_iter()
            .map(|name| self.note_manager.notes_folder.join(name))
            .collect();
        Ok(files)
    }

    /// Runs `chunk_fn` over the note files in parallel using scoped threads
    /// and merges the per-chunk results in file (descending name) order.
    ///
    /// Each chunk gets one invocation of `chunk_fn`, so per-chunk scratch
    /// state (e.g. a nucleo `Matcher` and UTF-32 buffers) is created once per
    /// chunk instead of once per file.
    fn par_map_files<T, F>(&self, chunk_fn: F) -> Result<Vec<T>, String>
    where
        T: Send,
        F: Fn(&NoteManager, &[PathBuf]) -> Vec<T> + Sync,
    {
        let files = self.get_note_files()?;
        if files.is_empty() {
            return Ok(Vec::new());
        }
        let workers = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
            .min(files.len())
            .max(1);
        let chunk_size = files.len().div_ceil(workers);
        let note_manager = self.note_manager;
        let chunk_fn = &chunk_fn;
        let results = std::thread::scope(|scope| {
            let handles: Vec<_> = files
                .chunks(chunk_size)
                .map(|chunk| scope.spawn(move || chunk_fn(note_manager, chunk)))
                .collect();
            handles
                .into_iter()
                .flat_map(|h| h.join().unwrap_or_default())
                .collect::<Vec<_>>()
        });
        Ok(results)
    }

    /// Extracts all unique thread names (lines starting with !!!) from markdown content.
    fn extract_thread_names(content: &str) -> Vec<String> {
        let frontmatter_len = Self::frontmatter_line_count(content);
        let mut names = Vec::new();

        for line in content.lines().skip(frontmatter_len) {
            if let Some(name) = line.strip_prefix("!!! ") {
                let name = name.trim();
                if !name.is_empty() {
                    names.push(name.to_string());
                }
            }
        }
        names
    }

    /// Filters search results based on various criteria.
    pub fn filter_search_results(
        results: Vec<SearchResult>,
        min_score: Option<i64>,
        max_results: Option<usize>,
        filename_filter: Option<&str>,
    ) -> Vec<SearchResult> {
        let mut filtered = results;

        // Apply minimum score filter
        if let Some(score) = min_score {
            filtered.retain(|r| r.score >= score);
        }

        // Apply filename filter (case-insensitive)
        if let Some(filter) = filename_filter {
            let filter_lower = filter.to_lowercase();
            filtered.retain(|r| r.filename.to_lowercase().contains(&filter_lower));
        }

        // Limit maximum results
        if let Some(max) = max_results {
            if filtered.len() > max {
                filtered.truncate(max);
            }
        }

        filtered
    }

    /// Sorts search results by relevance (score) and other criteria.
    pub fn sort_search_results(results: Vec<SearchResult>, sort_by: &str) -> Vec<SearchResult> {
        let mut sorted = results;

        match sort_by {
            "score" => {
                sorted.sort_by(|a, b| b.score.cmp(&a.score));
            }
            "filename" => {
                sorted.sort_by(|a, b| a.filename.cmp(&b.filename));
            }
            "date" => {
                // Sort by filename (which contains date) in reverse chronological order
                sorted.sort_by(|a, b| {
                    let a_date = a.filename.split('_').next().unwrap_or("");
                    let b_date = b.filename.split('_').next().unwrap_or("");
                    b_date.cmp(a_date) // Newest first
                });
            }
            _ => {
                // Default: sort by score descending
                sorted.sort_by(|a, b| b.score.cmp(&a.score));
            }
        }

        sorted
    }

    /// Processes search results with filtering and sorting.
    pub fn process_search_results(
        &self,
        results: Vec<SearchResult>,
        min_score: Option<i64>,
        max_results: Option<usize>,
        filename_filter: Option<&str>,
        sort_by: &str,
    ) -> Vec<SearchResult> {
        let filtered =
            Self::filter_search_results(results, min_score, max_results, filename_filter);
        Self::sort_search_results(filtered, sort_by)
    }

    /// Extracts all thread blocks with their IDs from note content.
    /// Returns a vector of (content, thread_id) tuples for all matching threads.
    fn extract_all_thread_blocks_with_id(
        content: &str,
        thread_name: &str,
    ) -> Vec<(String, String)> {
        let (frontmatter_len, frontmatter_content) = Self::extract_frontmatter(content);
        let lines: Vec<&str> = content.lines().collect();

        // Parse thread IDs and their line numbers from frontmatter
        // Note: line numbers in frontmatter are content-relative (0 = first content line)
        let thread_map = Self::parse_thread_map_from_frontmatter(&frontmatter_content);

        let mut blocks: Vec<(String, String)> = Vec::new();
        let mut thread_lines: Vec<&str> = Vec::new();
        let mut in_thread = false;
        let mut thread_start_line = 0; // Content-relative line number

        for (i, line) in lines.iter().skip(frontmatter_len).enumerate() {
            // i is the content-relative line number (0 = first line after frontmatter)
            // This matches the line numbers stored in frontmatter
            if line.starts_with("!!! ") {
                let name = line[4..].trim();
                if name == thread_name {
                    // If we were already in a thread, save the previous one
                    if in_thread && !thread_lines.is_empty() {
                        // Trim trailing empty lines
                        let mut trimmed_lines = thread_lines.clone();
                        while trimmed_lines.last().map(|l| l.trim().is_empty()) == Some(true) {
                            trimmed_lines.pop();
                        }
                        if !trimmed_lines.is_empty() {
                            let thread_id = thread_map
                                .get(&thread_start_line)
                                .cloned()
                                .unwrap_or_default();
                            blocks.push((trimmed_lines.join("\n").trim().to_string(), thread_id));
                        }
                    }

                    // Start new thread
                    in_thread = true;
                    thread_start_line = i; // Use content-relative line number
                    thread_lines.clear();
                    continue;
                } else if in_thread {
                    // End of current thread, save it
                    if !thread_lines.is_empty() {
                        // Trim trailing empty lines
                        let mut trimmed_lines = thread_lines.clone();
                        while trimmed_lines.last().map(|l| l.trim().is_empty()) == Some(true) {
                            trimmed_lines.pop();
                        }
                        if !trimmed_lines.is_empty() {
                            let thread_id = thread_map
                                .get(&thread_start_line)
                                .cloned()
                                .unwrap_or_default();
                            blocks.push((trimmed_lines.join("\n").trim().to_string(), thread_id));
                        }
                    }
                    in_thread = false;
                    thread_lines.clear();
                }
            }

            if in_thread {
                thread_lines.push(*line);
            }
        }

        // Don't forget the last thread if we're still in one
        if in_thread && !thread_lines.is_empty() {
            // Trim trailing empty lines
            let mut trimmed_lines = thread_lines;
            while trimmed_lines.last().map(|l| l.trim().is_empty()) == Some(true) {
                trimmed_lines.pop();
            }
            if !trimmed_lines.is_empty() {
                let thread_id = thread_map
                    .get(&thread_start_line)
                    .cloned()
                    .unwrap_or_default();
                blocks.push((trimmed_lines.join("\n").trim().to_string(), thread_id));
            }
        }

        blocks
    }

    /// Parses the frontmatter to create a map of line numbers to thread IDs.
    fn parse_thread_map_from_frontmatter(
        frontmatter: &str,
    ) -> std::collections::HashMap<usize, String> {
        use std::collections::HashMap;

        let mut map = HashMap::new();

        for line in frontmatter.lines() {
            let line = line.trim();
            if line.starts_with("threads:") {
                let value = line[8..].trim();
                for part in value.split(',') {
                    let part = part.trim();
                    if !part.is_empty() {
                        let parts: Vec<&str> = part.split(':').collect();
                        if parts.len() == 2 {
                            if let Ok(line_num) = parts[1].parse::<usize>() {
                                map.insert(line_num, parts[0].to_string());
                            }
                        }
                    }
                }
                break;
            }
        }

        map
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
            let mut line_buf = Vec::new();

            // One catch_unwind for the whole loop: a matcher panic skips the
            // remaining names instead of aborting per name.
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                for (name, count) in thread_counts {
                    let matched = if is_fuzzy {
                        Self::fuzzy_match(
                            &mut matcher,
                            &name,
                            &query_utf32,
                            &query_normalized,
                            &mut line_buf,
                        )
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
            }));
        }

        // Sort by note count descending, then by name
        results.sort_by(|a, b| b.note_count.cmp(&a.note_count).then(a.name.cmp(&b.name)));
        results
    }

    /// Keeps at most `max` results. For fuzzy search this is a bounded
    /// top-`max` selection by score (O(n) `select_nth_unstable` instead of
    /// sorting everything); for exact search the merged results are already in
    /// descending filename order because files are scanned newest-first, so a
    /// plain truncation suffices.
    fn cap_results(
        mut results: Vec<SearchResult>,
        is_fuzzy: bool,
        max: usize,
    ) -> Vec<SearchResult> {
        if is_fuzzy {
            if results.len() > max {
                results.select_nth_unstable_by(max - 1, |a, b| b.score.cmp(&a.score));
                results.truncate(max);
            }
            // Deterministic order: score desc, then newest file, then line.
            results.sort_by(|a, b| {
                b.score
                    .cmp(&a.score)
                    .then_with(|| b.filename.cmp(&a.filename))
                    .then_with(|| a.line_number.cmp(&b.line_number))
            });
        } else {
            results.truncate(max);
        }
        results
    }

    /// Searches all notes in the notes folder for the given query.
    /// Returns up to 100 results, sorted by score (fuzzy) or filename (exact).
    /// Files are scanned in parallel across scoped threads.
    pub fn search(&self, query: &str, is_fuzzy: bool) -> Result<Vec<SearchResult>, String> {
        if query.trim().is_empty() {
            return Ok(vec![]);
        }

        let query_normalized = query.to_lowercase();
        let results = self.par_map_files(|note_manager, chunk| {
            let mut matcher = Matcher::new(Config::DEFAULT);
            let mut query_buf = Vec::new();
            let query_utf32 = Utf32Str::new(query_normalized.as_str(), &mut query_buf);
            let mut line_buf = Vec::new();
            let mut out = Vec::new();
            for path in chunk {
                out.extend(Self::search_file(
                    note_manager,
                    path,
                    is_fuzzy,
                    &query_utf32,
                    &query_normalized,
                    &mut matcher,
                    &mut line_buf,
                ));
            }
            out
        })?;

        Ok(Self::cap_results(results, is_fuzzy, 100))
    }

    /// Searches for unique thread names (lines starting with !!!) across all notes.
    pub fn search_threads(
        &self,
        query: &str,
        is_fuzzy: bool,
    ) -> Result<Vec<ThreadSearchResult>, String> {
        let names = self.par_map_files(|_note_manager, chunk| {
            let mut names = Vec::new();
            for path in chunk {
                if let Ok(content) = fs::read_to_string(path) {
                    names.extend(Self::extract_thread_names(&content));
                }
            }
            names
        })?;

        let mut thread_counts: HashMap<String, usize> = HashMap::new();
        for name in names {
            *thread_counts.entry(name).or_insert(0) += 1;
        }

        Ok(self.filter_thread_results(thread_counts, query, is_fuzzy))
    }

    /// Searches for unique tags across all notes.
    pub fn search_tags(&self, query: &str, is_fuzzy: bool) -> Result<Vec<TagSearchResult>, String> {
        let tags = self.par_map_files(|_note_manager, chunk| {
            let mut tags = Vec::new();
            for path in chunk {
                if let Ok(content) = fs::read_to_string(path) {
                    tags.extend(crate::utils::tag_parser::parse_tags_from_content(&content));
                }
            }
            tags
        })?;

        let mut tag_counts: HashMap<String, usize> = HashMap::new();
        for tag in tags {
            *tag_counts.entry(tag).or_insert(0) += 1;
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
            let mut line_buf = Vec::new();

            // One catch_unwind for the whole loop (see filter_thread_results).
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                for (name, count) in tag_counts {
                    let matched = if is_fuzzy {
                        Self::fuzzy_match(
                            &mut matcher,
                            &name,
                            &query_utf32,
                            &query_normalized,
                            &mut line_buf,
                        )
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
            }));
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
        let query_normalized = query.to_lowercase();
        let tag_lower = tag.to_lowercase();
        let query_empty = query.trim().is_empty();

        let mut results = self.par_map_files(|note_manager, chunk| {
            let mut matcher = Matcher::new(Config::DEFAULT);
            let mut query_buf = Vec::new();
            let query_utf32 = Utf32Str::new(query_normalized.as_str(), &mut query_buf);
            let mut line_buf = Vec::new();
            let mut out = Vec::new();

            for path in chunk {
                let content = match fs::read_to_string(path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let tags = crate::utils::tag_parser::parse_tags_from_content(&content);
                if !tags.iter().any(|t| t.to_lowercase() == tag_lower) {
                    continue;
                }

                if query_empty {
                    let filename = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or_default();
                    let formatted_name = note_manager.format_note_name(filename);

                    let fm_len = Self::frontmatter_line_count(&content);
                    // Find first non-empty line after frontmatter
                    let first_line = content
                        .lines()
                        .skip(fm_len)
                        .find(|l| !l.trim().is_empty())
                        .map(|l| l.trim().to_string())
                        .unwrap_or_else(|| formatted_name.clone());

                    out.push(SearchResult {
                        filename: filename.to_string(),
                        formatted_name,
                        created: note_manager.extract_created_date(&content),
                        excerpt: first_line,
                        line_number: fm_len,
                        score: 0,
                        indices: vec![],
                    });
                } else {
                    out.extend(Self::search_file(
                        note_manager,
                        path,
                        is_fuzzy,
                        &query_utf32,
                        &query_normalized,
                        &mut matcher,
                        &mut line_buf,
                    ));
                }
            }
            out
        })?;

        if is_fuzzy && !query_empty {
            results.sort_by(|a, b| b.score.cmp(&a.score));
        } else {
            // Sort by filename descending (newest notes first)
            results.sort_by(|a, b| b.filename.cmp(&a.filename));
        }

        Ok(results)
    }

    /// Aggregates content from all threads matching the given name across all notes.
    pub fn aggregate_thread(&self, thread_name: &str) -> Result<ThreadAggregationResult, String> {
        let items = self.par_map_files(|note_manager, chunk| {
            let mut items = Vec::new();

            for path in chunk {
                let content = match fs::read_to_string(path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                // Extract all thread blocks with the matching name from this note
                let thread_blocks = Self::extract_all_thread_blocks_with_id(&content, thread_name);

                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default()
                    .to_string();

                for (block_content, thread_id) in thread_blocks {
                    items.push(ThreadAggregationItem {
                        filename: filename.clone(),
                        formatted_date: note_manager.format_note_name(&filename),
                        created: note_manager.extract_created_date(&content),
                        content: block_content,
                        thread_id,
                    });
                }
            }
            items
        })?;

        Ok(ThreadAggregationResult {
            thread_name: thread_name.to_string(),
            items,
        })
    }

    /// Returns all pinned threads across all notes with excerpts.
    pub fn get_pinned_threads(&self) -> Result<Vec<PinnedThreadItem>, String> {
        use crate::models::note_session::NoteSession;

        let mut pinned_threads = self.par_map_files(|note_manager, chunk| {
            let mut pinned = Vec::new();

            for path in chunk {
                let content = match fs::read_to_string(path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default()
                    .to_string();
                let formatted_date = note_manager.format_note_name(&filename);

                // Load the session to get thread metadata including pinned status
                let mut session = NoteSession::new();
                session.load(path.to_path_buf(), content.clone());

                // Find pinned threads
                for thread in &session.threads {
                    if thread.pinned {
                        // Extract excerpt from the thread content
                        let frontmatter_len = Self::frontmatter_line_count(&content);
                        let lines: Vec<&str> = content.lines().collect();

                        // Get the first non-empty line of the thread for excerpt
                        let mut excerpt = String::new();
                        let start = thread.start_line.max(frontmatter_len);
                        let end = thread.end_line.min(lines.len());

                        for i in start..end {
                            let line = lines[i].trim();
                            if !line.is_empty() && !line.starts_with("!!!") {
                                let stripped = crate::utils::markdown::strip_markdown_line(line);
                                if !stripped.is_empty() {
                                    excerpt = stripped;
                                    break;
                                }
                            }
                        }

                        // If no content found, use thread name as excerpt
                        if excerpt.is_empty() {
                            excerpt = thread.name.clone();
                        }

                        pinned.push(PinnedThreadItem {
                            thread_id: thread.id.clone(),
                            thread_name: thread.name.clone(),
                            filename: filename.clone(),
                            formatted_date: formatted_date.clone(),
                            excerpt,
                            line_number: thread.start_line,
                        });
                    }
                }
            }
            pinned
        })?;

        // Sort by filename descending (newest first), then by thread name
        pinned_threads.sort_by(|a, b| {
            b.filename
                .cmp(&a.filename)
                .then_with(|| a.thread_name.cmp(&b.thread_name))
        });

        Ok(pinned_threads)
    }

    /// Returns the content of a specific thread from a specific note file.
    pub fn get_thread_content(
        &self,
        filename: &str,
        thread_id: &str,
    ) -> Result<Option<String>, String> {
        use crate::models::note_session::NoteSession;

        let notes_folder = &self.note_manager.notes_folder;
        let path = notes_folder.join(filename);

        if !path.exists() {
            return Ok(None);
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Ok(None),
        };

        // Load session to get thread metadata
        let mut session = NoteSession::new();
        session.load(path, content.clone());

        // Find the thread by ID
        if let Some(thread) = session.threads.iter().find(|t| t.id == thread_id) {
            let (frontmatter_len, _) = Self::extract_frontmatter(&content);
            let lines: Vec<&str> = content.lines().collect();

            let start = thread.start_line.max(frontmatter_len);
            let end = thread.end_line.min(lines.len());

            let mut thread_lines: Vec<&str> = Vec::new();
            for i in start..end {
                thread_lines.push(lines[i]);
            }

            // Trim trailing empty lines
            while thread_lines.last().map(|l| l.trim().is_empty()) == Some(true) {
                thread_lines.pop();
            }

            if thread_lines.is_empty() {
                return Ok(None);
            }

            Ok(Some(thread_lines.join("\n")))
        } else {
            Ok(None)
        }
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
    /// Frontmatter Extraction Tests
    ///
    #[test]
    fn test_extract_frontmatter_no_frontmatter() {
        let content = "This is just regular content\nWith multiple lines";
        let (count, frontmatter) = SearchService::extract_frontmatter(content);
        assert_eq!(count, 0);
        assert_eq!(frontmatter, "");
    }

    #[test]
    fn test_extract_frontmatter_with_frontmatter() {
        let content = "---\ntitle: Test\ncreated: 2024-01-01\n---\n\nThis is the body";
        let (count, frontmatter) = SearchService::extract_frontmatter(content);
        assert_eq!(count, 4);
        assert!(frontmatter.contains("---"));
        assert!(frontmatter.contains("title: Test"));
        assert!(frontmatter.contains("created: 2024-01-01"));
    }

    #[test]
    fn test_extract_frontmatter_incomplete() {
        let content = "---\ntitle: Test\n";
        let (count, frontmatter) = SearchService::extract_frontmatter(content);
        assert_eq!(count, 0);
        assert_eq!(frontmatter, "");
    }

    ///
    /// Exact Match Tests
    ///
    #[test]
    fn test_find_exact_match_indices_found() {
        let line = "This is a test line";
        let query = "test";
        let result = SearchService::find_exact_match_indices(line, query);
        assert!(result.is_some());
        let (score, indices) = result.unwrap();
        assert_eq!(score, 0);
        assert!(!indices.is_empty());
    }

    #[test]
    fn test_find_exact_match_indices_not_found() {
        let line = "This is a test line";
        let query = "missing";
        let result = SearchService::find_exact_match_indices(line, query);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_exact_match_indices_case_insensitive() {
        let line = "This is a TEST line";
        let query = "test";
        let result = SearchService::find_exact_match_indices(line, query);
        assert!(result.is_some());
    }

    #[test]
    fn test_find_exact_match_indices_multibyte_characters() {
        let line = "Hello 世界";
        let query = "世界";
        let result = SearchService::find_exact_match_indices(line, query);
        assert!(result.is_some());
        let (_, indices) = result.unwrap();
        assert_eq!(indices.len(), 2); // 2 characters in "世界"
    }

    ///
    /// Thread Name Extraction Tests
    ///
    #[test]
    fn test_extract_thread_names_empty_content() {
        let content = "";
        let result = SearchService::extract_thread_names(content);
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_thread_names_no_threads() {
        let content = "---\ntitle: Test\n---\nThis is just content\nNo threads here";
        let result = SearchService::extract_thread_names(content);
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_thread_names_single_thread() {
        let content = "---\ntitle: Test\n---\n!!! Main Thread\nSome content";
        let result = SearchService::extract_thread_names(content);
        assert_eq!(result, vec!["Main Thread"]);
    }

    #[test]
    fn test_extract_thread_names_multiple_threads() {
        let content = "---\ntitle: Test\n---\n!!! Thread 1\nContent 1\n!!! Thread 2\nContent 2";
        let result = SearchService::extract_thread_names(content);
        assert_eq!(result, vec!["Thread 1", "Thread 2"]);
    }

    #[test]
    fn test_extract_thread_names_skips_empty() {
        let content = "!!! \n!!! Valid\n!!!   \n!!! Also Valid";
        let result = SearchService::extract_thread_names(content);
        assert_eq!(result, vec!["Valid", "Also Valid"]);
    }

    #[test]
    fn test_extract_thread_names_with_frontmatter() {
        let content = "---
title: Test\nthreads: ThreadA:5,ThreadB:10\n---\n!!! Thread A\nContent\n!!! Thread B\nMore";
        let result = SearchService::extract_thread_names(content);
        assert_eq!(result, vec!["Thread A", "Thread B"]);
    }

    ///
    /// Thread Map Parsing Tests
    ///
    #[test]
    fn test_parse_thread_map_from_frontmatter_empty() {
        let frontmatter = "---\ntitle: Test\n---";
        let result = SearchService::parse_thread_map_from_frontmatter(frontmatter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_thread_map_from_frontmatter_no_threads() {
        let frontmatter = "---\ntitle: Test\ncreated: 2024-01-01\n---";
        let result = SearchService::parse_thread_map_from_frontmatter(frontmatter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_thread_map_from_frontmatter_single() {
        let frontmatter = "---\ntitle: Test\nthreads: ThreadA:5\n---";
        let result = SearchService::parse_thread_map_from_frontmatter(frontmatter);
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(&5), Some(&"ThreadA".to_string()));
    }

    #[test]
    fn test_parse_thread_map_from_frontmatter_multiple() {
        let frontmatter = "---\ntitle: Test\nthreads: ThreadA:5,ThreadB:10,ThreadC:15\n---";
        let result = SearchService::parse_thread_map_from_frontmatter(frontmatter);
        assert_eq!(result.len(), 3);
        assert_eq!(result.get(&5), Some(&"ThreadA".to_string()));
        assert_eq!(result.get(&10), Some(&"ThreadB".to_string()));
        assert_eq!(result.get(&15), Some(&"ThreadC".to_string()));
    }

    #[test]
    fn test_parse_thread_map_from_frontmatter_with_spaces() {
        let frontmatter = "---\ntitle: Test\nthreads: Thread A:5, Thread B:10\n---";
        let result = SearchService::parse_thread_map_from_frontmatter(frontmatter);
        assert_eq!(result.len(), 2);
        assert_eq!(result.get(&5), Some(&"Thread A".to_string()));
        assert_eq!(result.get(&10), Some(&"Thread B".to_string()));
    }

    ///
    /// Filter and Sort Tests
    ///
    #[test]
    fn test_filter_search_results_empty() {
        let results: Vec<SearchResult> = vec![];
        let filtered =
            SearchService::filter_search_results(results, Some(50), Some(10), Some("test"));
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_filter_search_results_by_min_score() {
        let results = vec![
            SearchResult {
                filename: "note1.md".to_string(),
                formatted_name: "Note 1".to_string(),
                created: None,
                excerpt: "content".to_string(),
                line_number: 0,
                score: 80,
                indices: vec![],
            },
            SearchResult {
                filename: "note2.md".to_string(),
                formatted_name: "Note 2".to_string(),
                created: None,
                excerpt: "content".to_string(),
                line_number: 0,
                score: 60,
                indices: vec![],
            },
        ];
        let filtered = SearchService::filter_search_results(results, Some(70), None, None);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].filename, "note1.md");
    }

    #[test]
    fn test_filter_search_results_by_filename() {
        let results = vec![
            SearchResult {
                filename: "test-note.md".to_string(),
                formatted_name: "Test Note".to_string(),
                created: None,
                excerpt: "content".to_string(),
                line_number: 0,
                score: 0,
                indices: vec![],
            },
            SearchResult {
                filename: "other-note.md".to_string(),
                formatted_name: "Other Note".to_string(),
                created: None,
                excerpt: "content".to_string(),
                line_number: 0,
                score: 0,
                indices: vec![],
            },
        ];
        let filtered = SearchService::filter_search_results(results, None, None, Some("test"));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].filename, "test-note.md");
    }

    #[test]
    fn test_filter_search_results_by_max_results() {
        let results = vec![
            SearchResult {
                filename: "note1.md".to_string(),
                formatted_name: "Note 1".to_string(),
                created: None,
                excerpt: "content".to_string(),
                line_number: 0,
                score: 0,
                indices: vec![],
            },
            SearchResult {
                filename: "note2.md".to_string(),
                formatted_name: "Note 2".to_string(),
                created: None,
                excerpt: "content".to_string(),
                line_number: 0,
                score: 0,
                indices: vec![],
            },
            SearchResult {
                filename: "note3.md".to_string(),
                formatted_name: "Note 3".to_string(),
                created: None,
                excerpt: "content".to_string(),
                line_number: 0,
                score: 0,
                indices: vec![],
            },
        ];
        let filtered = SearchService::filter_search_results(results, None, Some(2), None);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_sort_search_results_by_score() {
        let results = vec![
            SearchResult {
                filename: "note1.md".to_string(),
                formatted_name: "Note 1".to_string(),
                created: None,
                excerpt: "content".to_string(),
                line_number: 0,
                score: 60,
                indices: vec![],
            },
            SearchResult {
                filename: "note2.md".to_string(),
                formatted_name: "Note 2".to_string(),
                created: None,
                excerpt: "content".to_string(),
                line_number: 0,
                score: 80,
                indices: vec![],
            },
            SearchResult {
                filename: "note3.md".to_string(),
                formatted_name: "Note 3".to_string(),
                created: None,
                excerpt: "content".to_string(),
                line_number: 0,
                score: 70,
                indices: vec![],
            },
        ];
        let sorted = SearchService::sort_search_results(results, "score");
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].filename, "note2.md"); // Highest score first
        assert_eq!(sorted[0].score, 80);
        assert_eq!(sorted[1].filename, "note3.md");
        assert_eq!(sorted[1].score, 70);
        assert_eq!(sorted[2].filename, "note1.md");
        assert_eq!(sorted[2].score, 60);
    }

    #[test]
    fn test_sort_search_results_by_filename() {
        let results = vec![
            SearchResult {
                filename: "b-note.md".to_string(),
                formatted_name: "B Note".to_string(),
                created: None,
                excerpt: "content".to_string(),
                line_number: 0,
                score: 0,
                indices: vec![],
            },
            SearchResult {
                filename: "a-note.md".to_string(),
                formatted_name: "A Note".to_string(),
                created: None,
                excerpt: "content".to_string(),
                line_number: 0,
                score: 0,
                indices: vec![],
            },
            SearchResult {
                filename: "c-note.md".to_string(),
                formatted_name: "C Note".to_string(),
                created: None,
                excerpt: "content".to_string(),
                line_number: 0,
                score: 0,
                indices: vec![],
            },
        ];
        let sorted = SearchService::sort_search_results(results, "filename");
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].filename, "a-note.md");
        assert_eq!(sorted[1].filename, "b-note.md");
        assert_eq!(sorted[2].filename, "c-note.md");
    }

    #[test]
    fn test_sort_search_results_by_date() {
        let results = vec![
            SearchResult {
                filename: "2024-01-01_test.md".to_string(),
                formatted_name: "2024-01-01".to_string(),
                created: None,
                excerpt: "content".to_string(),
                line_number: 0,
                score: 0,
                indices: vec![],
            },
            SearchResult {
                filename: "2024-01-03_test.md".to_string(),
                formatted_name: "2024-01-03".to_string(),
                created: None,
                excerpt: "content".to_string(),
                line_number: 0,
                score: 0,
                indices: vec![],
            },
            SearchResult {
                filename: "2024-01-02_test.md".to_string(),
                formatted_name: "2024-01-02".to_string(),
                created: None,
                excerpt: "content".to_string(),
                line_number: 0,
                score: 0,
                indices: vec![],
            },
        ];
        let sorted = SearchService::sort_search_results(results, "date");
        assert_eq!(sorted.len(), 3);
        // Newest first
        assert_eq!(sorted[0].filename, "2024-01-03_test.md");
        assert_eq!(sorted[1].filename, "2024-01-02_test.md");
        assert_eq!(sorted[2].filename, "2024-01-01_test.md");
    }

    #[test]
    fn test_sort_search_results_default() {
        let results = vec![
            SearchResult {
                filename: "note1.md".to_string(),
                formatted_name: "Note 1".to_string(),
                created: None,
                excerpt: "content".to_string(),
                line_number: 0,
                score: 60,
                indices: vec![],
            },
            SearchResult {
                filename: "note2.md".to_string(),
                formatted_name: "Note 2".to_string(),
                created: None,
                excerpt: "content".to_string(),
                line_number: 0,
                score: 80,
                indices: vec![],
            },
        ];
        let sorted = SearchService::sort_search_results(results, "unknown");
        assert_eq!(sorted.len(), 2);
        // Default is by score descending
        assert_eq!(sorted[0].filename, "note2.md");
        assert_eq!(sorted[0].score, 80);
    }

    ///
    /// Filter Tag Results Tests
    ///
    #[test]
    fn test_filter_tag_results_empty_query() {
        let mut tag_counts = HashMap::new();
        tag_counts.insert("rust".to_string(), 5);
        tag_counts.insert("python".to_string(), 3);

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();
        let note_manager = NoteManager::new(notes_path, "en".to_string());
        let search_service = SearchService::new(&note_manager);

        let results = search_service.filter_tag_results(tag_counts, "", true);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_filter_tag_results_with_query() {
        let mut tag_counts = HashMap::new();
        tag_counts.insert("rust".to_string(), 5);
        tag_counts.insert("python".to_string(), 3);
        tag_counts.insert("javascript".to_string(), 2);

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();
        let note_manager = NoteManager::new(notes_path, "en".to_string());
        let search_service = SearchService::new(&note_manager);

        let results = search_service.filter_tag_results(tag_counts, "rust", false);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "rust");
        assert_eq!(results[0].note_count, 5);
    }

    #[test]
    fn test_filter_tag_results_sorted_by_count() {
        let mut tag_counts = HashMap::new();
        tag_counts.insert("rust".to_string(), 5);
        tag_counts.insert("python".to_string(), 10);
        tag_counts.insert("javascript".to_string(), 2);

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();
        let note_manager = NoteManager::new(notes_path, "en".to_string());
        let search_service = SearchService::new(&note_manager);

        let results = search_service.filter_tag_results(tag_counts, "", false);
        assert_eq!(results.len(), 3);
        // Sorted by count descending
        assert_eq!(results[0].name, "python");
        assert_eq!(results[0].note_count, 10);
        assert_eq!(results[1].name, "rust");
        assert_eq!(results[1].note_count, 5);
        assert_eq!(results[2].name, "javascript");
        assert_eq!(results[2].note_count, 2);
    }

    ///
    /// Filter Thread Results Tests
    ///
    #[test]
    fn test_filter_thread_results_empty_query() {
        let mut thread_counts = HashMap::new();
        thread_counts.insert("ThreadA".to_string(), 5);
        thread_counts.insert("ThreadB".to_string(), 3);

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();
        let note_manager = NoteManager::new(notes_path, "en".to_string());
        let search_service = SearchService::new(&note_manager);

        let results = search_service.filter_thread_results(thread_counts, "", true);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_filter_thread_results_with_query() {
        let mut thread_counts = HashMap::new();
        thread_counts.insert("ThreadA".to_string(), 5);
        thread_counts.insert("ThreadB".to_string(), 3);
        thread_counts.insert("OtherItem".to_string(), 2);

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();
        let note_manager = NoteManager::new(notes_path, "en".to_string());
        let search_service = SearchService::new(&note_manager);

        let results = search_service.filter_thread_results(thread_counts, "Thread", false);
        assert_eq!(results.len(), 2);
        // Both ThreadA and ThreadB contain "Thread"
        assert!(results.iter().any(|r| r.name == "ThreadA"));
        assert!(results.iter().any(|r| r.name == "ThreadB"));
    }

    #[test]
    fn test_filter_thread_results_sorted() {
        let mut thread_counts = HashMap::new();
        thread_counts.insert("Alpha".to_string(), 5);
        thread_counts.insert("Beta".to_string(), 10);
        thread_counts.insert("Gamma".to_string(), 2);

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();
        let note_manager = NoteManager::new(notes_path, "en".to_string());
        let search_service = SearchService::new(&note_manager);

        let results = search_service.filter_thread_results(thread_counts, "", false);
        assert_eq!(results.len(), 3);
        // Sorted by count descending, then by name
        assert_eq!(results[0].name, "Beta");
        assert_eq!(results[0].note_count, 10);
        assert_eq!(results[1].name, "Alpha");
        assert_eq!(results[1].note_count, 5);
        assert_eq!(results[2].name, "Gamma");
        assert_eq!(results[2].note_count, 2);
    }

    ///
    /// Get Note Files Tests
    ///
    #[test]
    fn test_get_note_files_empty_directory() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();
        fs::create_dir_all(&notes_path).expect("Failed to create notes dir");

        let note_manager = NoteManager::new(notes_path, "en".to_string());
        let search_service = SearchService::new(&note_manager);

        let result = search_service.get_note_files();
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_get_note_files_non_existent_directory() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().join("nonexistent");

        let note_manager = NoteManager::new(notes_path, "en".to_string());
        let search_service = SearchService::new(&note_manager);

        let result = search_service.get_note_files();
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_get_note_files_filters_markdown() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();
        fs::create_dir_all(&notes_path).expect("Failed to create notes dir");

        fs::write(notes_path.join("2024-01-01.md"), "# Note 1").expect("Failed to write file");
        fs::write(notes_path.join("2024-01-02.md"), "# Note 2").expect("Failed to write file");
        fs::write(notes_path.join("readme.txt"), "not a markdown file")
            .expect("Failed to write file");

        let note_manager = NoteManager::new(notes_path, "en".to_string());
        let search_service = SearchService::new(&note_manager);

        let result = search_service.get_note_files();
        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_get_note_files_sorts_descending() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();
        fs::create_dir_all(&notes_path).expect("Failed to create notes dir");

        fs::write(notes_path.join("2024-01-01.md"), "# Note 1").expect("Failed to write file");
        fs::write(notes_path.join("2024-01-02.md"), "# Note 2").expect("Failed to write file");
        fs::write(notes_path.join("2024-01-03.md"), "# Note 3").expect("Failed to write file");

        let note_manager = NoteManager::new(notes_path, "en".to_string());
        let search_service = SearchService::new(&note_manager);

        let result = search_service.get_note_files();
        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 3);
        // Sorted descending (newest first)
        assert_eq!(
            files[0].file_name().unwrap().to_string_lossy(),
            "2024-01-03.md"
        );
        assert_eq!(
            files[1].file_name().unwrap().to_string_lossy(),
            "2024-01-02.md"
        );
        assert_eq!(
            files[2].file_name().unwrap().to_string_lossy(),
            "2024-01-01.md"
        );
    }

    #[test]
    fn test_get_note_files_skips_dotfiles() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();
        fs::create_dir_all(&notes_path).expect("Failed to create notes dir");

        fs::write(notes_path.join("2024-01-01.md"), "# Note").expect("Failed to write file");
        fs::write(notes_path.join(".hidden.md"), "# Hidden").expect("Failed to write file");
        fs::write(notes_path.join(".DS_Store.md"), "junk").expect("Failed to write file");

        let note_manager = NoteManager::new(notes_path, "en".to_string());
        let search_service = SearchService::new(&note_manager);

        let files = search_service
            .get_note_files()
            .expect("get_note_files failed");
        assert_eq!(files.len(), 1);
        assert_eq!(
            files[0].file_name().unwrap().to_string_lossy(),
            "2024-01-01.md"
        );
    }

    ///
    /// Search Tests (parallel scan, bounded top-100)
    ///
    #[test]
    fn test_search_exact_caps_at_100() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();
        fs::create_dir_all(&notes_path).expect("Failed to create notes dir");

        let content = (0..150)
            .map(|i| format!("test line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(notes_path.join("2024-01-01.md"), content).expect("Failed to write file");

        let note_manager = NoteManager::new(notes_path, "en".to_string());
        let search_service = SearchService::new(&note_manager);

        let results = search_service.search("test", false).expect("Search failed");
        assert_eq!(results.len(), 100, "Exact search should cap at 100 results");
        // Files are scanned newest-first and lines in order, so the first 100
        // matching lines are kept.
        assert_eq!(results[0].line_number, 0);
        assert_eq!(results[99].line_number, 99);
    }

    #[test]
    fn test_search_fuzzy_caps_at_100_and_sorted_by_score() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();
        fs::create_dir_all(&notes_path).expect("Failed to create notes dir");

        let content = (0..150)
            .map(|i| format!("test line number {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(notes_path.join("2024-01-01.md"), content).expect("Failed to write file");

        let note_manager = NoteManager::new(notes_path, "en".to_string());
        let search_service = SearchService::new(&note_manager);

        let results = search_service.search("tst", true).expect("Search failed");
        assert_eq!(results.len(), 100, "Fuzzy search should cap at 100 results");
        assert!(
            results.windows(2).all(|w| w[0].score >= w[1].score),
            "Results should be sorted by score descending"
        );
    }

    #[test]
    fn test_search_exact_orders_newest_file_first_across_files() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();
        fs::create_dir_all(&notes_path).expect("Failed to create notes dir");

        for day in 1..=5 {
            let content = format!("needle in note {}", day);
            fs::write(notes_path.join(format!("2024-01-{:02}.md", day)), content)
                .expect("Failed to write file");
        }

        let note_manager = NoteManager::new(notes_path, "en".to_string());
        let search_service = SearchService::new(&note_manager);

        let results = search_service
            .search("needle", false)
            .expect("Search failed");
        assert_eq!(results.len(), 5);
        let filenames: Vec<&str> = results.iter().map(|r| r.filename.as_str()).collect();
        assert_eq!(
            filenames,
            vec![
                "2024-01-05.md",
                "2024-01-04.md",
                "2024-01-03.md",
                "2024-01-02.md",
                "2024-01-01.md"
            ],
            "Parallel scan must merge results in descending filename order"
        );
    }

    #[test]
    fn test_search_skips_dotfiles() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();
        fs::create_dir_all(&notes_path).expect("Failed to create notes dir");

        fs::write(notes_path.join("2024-01-01.md"), "visible needle")
            .expect("Failed to write file");
        fs::write(notes_path.join(".hidden.md"), "hidden needle").expect("Failed to write file");

        let note_manager = NoteManager::new(notes_path, "en".to_string());
        let search_service = SearchService::new(&note_manager);

        let results = search_service
            .search("needle", false)
            .expect("Search failed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "2024-01-01.md");
    }

    ///
    /// Process Search Results Tests
    ///
    #[test]
    fn test_process_search_results_applies_filters_and_sort() {
        let results = vec![
            SearchResult {
                filename: "note1.md".to_string(),
                formatted_name: "Note 1".to_string(),
                created: None,
                excerpt: "content".to_string(),
                line_number: 0,
                score: 80,
                indices: vec![],
            },
            SearchResult {
                filename: "note2.md".to_string(),
                formatted_name: "Note 2".to_string(),
                created: None,
                excerpt: "content".to_string(),
                line_number: 0,
                score: 60,
                indices: vec![],
            },
        ];

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let notes_path = temp_dir.path().to_path_buf();
        let note_manager = NoteManager::new(notes_path, "en".to_string());
        let search_service = SearchService::new(&note_manager);

        let processed =
            search_service.process_search_results(results, Some(70), None, None, "score");
        assert_eq!(processed.len(), 1);
        assert_eq!(processed[0].filename, "note1.md");
    }

    ///
    /// Edge Cases
    ///
    #[test]
    fn test_extract_frontmatter_empty_content() {
        let content = "";
        let (count, frontmatter) = SearchService::extract_frontmatter(content);
        assert_eq!(count, 0);
        assert_eq!(frontmatter, "");
    }

    #[test]
    fn test_extract_thread_names_empty_lines() {
        let content = "!!! \n\n!!! Valid\n\n!!!   \n";
        let result = SearchService::extract_thread_names(content);
        assert_eq!(result, vec!["Valid"]);
    }

    #[test]
    fn test_filter_search_results_combined_filters() {
        let results = vec![
            SearchResult {
                filename: "test-note1.md".to_string(),
                formatted_name: "Test Note 1".to_string(),
                created: None,
                excerpt: "content".to_string(),
                line_number: 0,
                score: 80,
                indices: vec![],
            },
            SearchResult {
                filename: "test-note2.md".to_string(),
                formatted_name: "Test Note 2".to_string(),
                created: None,
                excerpt: "content".to_string(),
                line_number: 0,
                score: 85,
                indices: vec![],
            },
            SearchResult {
                filename: "other-note.md".to_string(),
                formatted_name: "Other Note".to_string(),
                created: None,
                excerpt: "content".to_string(),
                line_number: 0,
                score: 90,
                indices: vec![],
            },
        ];
        let filtered =
            SearchService::filter_search_results(results, Some(70), Some(2), Some("test"));
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].filename, "test-note1.md");
        assert_eq!(filtered[1].filename, "test-note2.md");
    }

    #[test]
    fn test_sort_search_results_empty() {
        let results: Vec<SearchResult> = vec![];
        let sorted = SearchService::sort_search_results(results, "score");
        assert!(sorted.is_empty());
    }

    #[test]
    fn test_parse_thread_map_from_frontmatter_malformed() {
        let frontmatter = "---\ntitle: Test\nthreads: ThreadA:not_a_number\n---";
        let result = SearchService::parse_thread_map_from_frontmatter(frontmatter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_thread_map_from_frontmatter_no_colon() {
        let frontmatter = "---\ntitle: Test\nthreads: ThreadA ThreadB\n---";
        let result = SearchService::parse_thread_map_from_frontmatter(frontmatter);
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_thread_names_with_frontmatter_and_content() {
        let content = "---\ntitle: Test\ntags: [a, b]\n---\n!!! Thread 1\nContent line 1\n!!! Thread 2\nContent line 2";
        let result = SearchService::extract_thread_names(content);
        assert_eq!(result, vec!["Thread 1", "Thread 2"]);
    }
}
