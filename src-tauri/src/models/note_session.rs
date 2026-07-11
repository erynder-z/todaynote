//! Transient state for the active note being edited.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Represents a named block or thread within a note, typically defined by a `#` heading.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteThread {
    /// Unique identifier for the thread.
    pub id: String,
    /// The display name of the thread (e.g., "Work").
    pub name: String,
    /// The absolute line index where the thread header starts.
    pub start_line: usize,
    /// The absolute line index where the thread ends (exclusive).
    pub end_line: usize,
    /// The heading level (1 for #, 2 for ##, etc.).
    pub level: usize,
}

/// Represents an active note session, maintaining an in-memory
/// representation of the file's lines for real-time editing.
pub struct NoteSession {
    /// File path of the currently open note.
    pub path: Option<PathBuf>,
    /// Each line of the note as an individual string in the vector.
    pub lines: Vec<String>,
    /// The start and end indices (inclusive) of the YAML frontmatter block.
    pub frontmatter_range: Option<(usize, usize)>,
    /// List of detected threads in the note.
    pub threads: Vec<NoteThread>,
}

impl NoteSession {
    /// Creates a new, empty note session.
    pub fn new() -> Self {
        Self {
            path: None,
            lines: Vec::new(),
            frontmatter_range: None,
            threads: Vec::new(),
        }
    }

    /// Loads the content and path of a note into the session.
    pub fn load(&mut self, path: PathBuf, content: String) {
        self.path = Some(path);
        self.lines = content.split('\n').map(|s| s.to_string()).collect();
        self.detect_frontmatter();
        self.ensure_trailing_empty_line();
        self.migrate_html_comments_to_frontmatter();
        self.detect_threads();
    }

    /// Detects YAML frontmatter (delimited by '---' on the first line and another '---').
    pub fn detect_frontmatter(&mut self) {
        if self.lines.is_empty() {
            self.frontmatter_range = None;
            return;
        }

        if self.lines[0].trim() == "---" {
            for (i, line) in self.lines.iter().enumerate().skip(1) {
                if line.trim() == "---" {
                    self.frontmatter_range = Some((0, i));
                    return;
                }
            }
        }
        self.frontmatter_range = None;
    }
    /// Scans the lines to identify threads based on `#` headings.
    /// Only top-level headings (#) are considered threads.
    /// Uses thread IDs from frontmatter if available, otherwise generates new UUIDs.
    pub fn detect_threads(&mut self) {
        use uuid::Uuid;
        
        self.threads.clear();
        let content_start = self.get_content_start_index();
        
        // Try to parse thread IDs from frontmatter first
        let frontmatter_threads = self.parse_threads_from_frontmatter();
        
        // Build a map of line -> ID from frontmatter
        let mut id_map: std::collections::HashMap<usize, String> = std::collections::HashMap::new();
        for (id, relative_line) in &frontmatter_threads {
            let absolute_line = content_start + relative_line;
            id_map.insert(absolute_line, id.clone());
        }

        for i in content_start..self.lines.len() {
            let line = &self.lines[i];
            // Only match top-level headings (# followed by space, not ##)
            if line.starts_with("# ") {
                let name = line[2..].trim().to_string();

                if !name.is_empty() {
                    // Update previous thread's end_line
                    if let Some(prev) = self.threads.last_mut() {
                        prev.end_line = i;
                    }

                    // Use ID from frontmatter if available, otherwise generate new UUID
                    let id = if let Some(existing_id) = id_map.get(&i) {
                        existing_id.clone()
                    } else {
                        // Generate new UUID for this thread
                        Uuid::new_v4().to_string()
                    };

                    self.threads.push(NoteThread {
                        id,
                        name,
                        start_line: i,
                        end_line: self.lines.len(),
                        level: 1,
                    });
                }
            }
        }
        
        // Update frontmatter with current thread metadata
        self.update_threads_in_frontmatter();
    }



    /// Ensures that the frontmatter block exists.
    pub fn ensure_frontmatter(&mut self) {
        if self.frontmatter_range.is_none() {
            self.lines.insert(0, "---".to_string());
            self.lines.insert(1, "---".to_string());
            self.frontmatter_range = Some((0, 1));
        }
    }

    /// Finds the index of a metadata line starting with the given key.
    pub fn find_metadata_line(&self, key: &str) -> Option<usize> {
        let (start, end) = self.frontmatter_range?;
        let prefix = format!("{}:", key);
        self.lines[start + 1..end]
            .iter()
            .position(|l| l.trim().starts_with(&prefix))
            .map(|relative_idx| start + 1 + relative_idx)
    }

    /// Parses the frontmatter block into a HashMap.
    pub fn get_metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        if let Some((start, end)) = self.frontmatter_range {
            for i in (start + 1)..end {
                let line = &self.lines[i];
                if let Some((key, value)) = line.split_once(':') {
                    metadata.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }
        metadata
    }

    /// Replaces the content of a specific line in the session.
    pub fn update_line(&mut self, index: usize, content: String) {
        if index < self.lines.len() {
            self.lines[index] = content;
            // If the first line changed, or we are within a frontmatter block, re-detect
            if index == 0 || self.frontmatter_range.is_some() {
                self.detect_frontmatter();
            }
            self.detect_threads();
        }
    }

    /// Inserts a new line at the specified index.
    pub fn insert_line(&mut self, index: usize, content: String) {
        if index <= self.lines.len() {
            self.lines.insert(index, content);
            self.detect_frontmatter();
            self.detect_threads();
        }
    }

    /// Removes a specific line from the session.
    pub fn delete_line(&mut self, index: usize) {
        if index < self.lines.len() && self.lines.len() > 1 {
            self.lines.remove(index);
            self.detect_frontmatter();
            self.detect_threads();
        }
    }

    /// Removes a range of lines from the session.
    pub fn delete_line_range(&mut self, start: usize, end: usize) {
        if start < end && end <= self.lines.len() {
            self.lines.drain(start..end);
            self.detect_frontmatter();
            self.ensure_trailing_empty_line();
            self.detect_threads();
        }
    }

    /// Returns the index where the actual content (after frontmatter) starts.
    pub fn get_content_start_index(&self) -> usize {
        match self.frontmatter_range {
            Some((_, end)) => end + 1,
            None => 0,
        }
    }

    /// Maps a relative index (from the editor's perspective) to the absolute line index.
    pub fn to_absolute_index(&self, relative_index: usize) -> usize {
        self.get_content_start_index() + relative_index
    }



    /// Replaces the content of a specific line in the session using a relative index.
    pub fn update_content_line(&mut self, relative_index: usize, content: String) {
        let abs_index = self.to_absolute_index(relative_index);
        self.update_line(abs_index, content);
    }

    /// Inserts a new line at the specified relative index.
    pub fn insert_content_line(&mut self, relative_index: usize, content: String) {
        let abs_index = self.to_absolute_index(relative_index);
        self.insert_line(abs_index, content);
    }

    /// Removes a specific line from the session using a relative index.
    pub fn delete_content_line(&mut self, relative_index: usize) {
        let abs_index = self.to_absolute_index(relative_index);
        self.delete_line(abs_index);
    }

    /// Ensures that the session ends with enough empty lines to allow for a new paragraph.
    pub fn ensure_trailing_empty_line(&mut self) {
        let content_start = self.get_content_start_index();

        let last_non_empty = self.lines.iter().rposition(|l| !l.trim().is_empty());

        match last_non_empty {
            Some(idx) if idx >= content_start => {
                let trailing_empty_count = self.lines.len() - 1 - idx;
                if trailing_empty_count < 2 {
                    for _ in 0..(2 - trailing_empty_count) {
                        self.lines.push("".to_string());
                    }
                }
            }
            _ => {
                // No content or only empty lines.
                // Ensure at least two lines to be safe for Milkdown's parser.
                while self.lines.len() < content_start + 2 {
                    self.lines.push("".to_string());
                }
            }
        }
    }

    /// Reconstructs the full note content by joining the lines with newlines.
    pub fn get_full_content(&self) -> String {
        self.lines.join("\n")
    }

    /// Migrates thread IDs from HTML comments to frontmatter metadata.
    /// This is a one-time migration for notes that were using HTML comments for thread IDs.
    /// Removes HTML comments after extracting the IDs.
    pub fn migrate_html_comments_to_frontmatter(&mut self) {
        let content_start = self.get_content_start_index();
        let mut has_html_comments = false;
        let mut html_thread_ids: Vec<(String, usize)> = Vec::new(); // (id, line_index)
        
        // Scan for HTML thread ID comments
        for i in content_start..self.lines.len() {
            if i > content_start && self.lines[i].trim().starts_with("<!--") && self.lines[i].contains("thread-id:") {
                // Parse the thread ID from the comment
                if let Some(id) = self.parse_thread_id_from_html_comment(&self.lines[i]) {
                    // The thread header is the previous line
                    if i > 0 && self.lines[i - 1].starts_with("# ") {
                        let relative_line = i - 1 - content_start;
                        html_thread_ids.push((id, relative_line));
                        has_html_comments = true;
                    }
                }
            }
        }
        
        // If we found HTML comments, remove them and update frontmatter
        if has_html_comments {
            // Ensure frontmatter exists
            self.ensure_frontmatter();
            let (start, end) = self.frontmatter_range.unwrap();
            
            // Build the threads metadata line
            let threads_meta: String = html_thread_ids.iter()
                .map(|(id, line)| format!("{}:{}", id, line))
                .collect::<Vec<_>>()
                .join(",");
            
            // Add or update threads in frontmatter
            let threads_key = "threads:";
            let mut found = false;
            for i in (start + 1)..end {
                if self.lines[i].trim().starts_with(threads_key) {
                    self.lines[i] = format!("{} {}", threads_key, threads_meta);
                    found = true;
                    break;
                }
            }
            if !found && end > start + 1 {
                self.lines.insert(end, format!("{} {}", threads_key, threads_meta));
                self.frontmatter_range = Some((start, end + 1));
            }
            
            // Remove HTML comment lines
            let mut i = 0;
            while i < self.lines.len() {
                if self.lines[i].trim().starts_with("<!--") && self.lines[i].contains("thread-id:") {
                    self.lines.remove(i);
                } else {
                    i += 1;
                }
            }
            
            // Re-detect frontmatter after modification
            self.detect_frontmatter();
        }
    }

    /// Parses a thread ID from an HTML comment line.
    /// Expected format: <!-- thread-id: uuid -->
    fn parse_thread_id_from_html_comment(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        if trimmed.starts_with("<!--") && trimmed.ends_with("-->") {
            if let Some(content) = trimmed.strip_prefix("<!--").and_then(|s| s.strip_suffix("-->")) {
                let content = content.trim();
                if content.starts_with("thread-id:") {
                    let id_part = content.strip_prefix("thread-id:").unwrap_or("").trim();
                    if !id_part.is_empty() {
                        return Some(id_part.to_string());
                    }
                }
            }
        }
        None
    }

    /// Updates the threads metadata in the frontmatter.
    /// Stores thread IDs and their relative line positions (from content start).
    pub fn update_threads_in_frontmatter(&mut self) {
        self.ensure_frontmatter();
        let (start, end) = self.frontmatter_range.unwrap();
        let content_start = self.get_content_start_index();
        
        // Build the threads metadata as a comma-separated list of id:line pairs
        let threads_meta: String = self.threads.iter()
            .map(|t| {
                let relative_line = if t.start_line >= content_start {
                    t.start_line - content_start
                } else {
                    0
                };
                format!("{}:{}", t.id, relative_line)
            })
            .collect::<Vec<_>>()
            .join(",");
        
        // Find and update or add the threads line in frontmatter
        let threads_key = "threads:";
        let mut found = false;
        
        for i in (start + 1)..end {
            if self.lines[i].trim().starts_with(threads_key) {
                self.lines[i] = format!("{} {}", threads_key, threads_meta);
                found = true;
                break;
            }
        }
        
        // If not found, add it before the closing ---
        if !found && end > start + 1 {
            self.lines.insert(end, format!("{} {}", threads_key, threads_meta));
            self.frontmatter_range = Some((start, end + 1));
        }
    }

    /// Parses thread metadata from the frontmatter.
    /// Expected format: threads: id1:line1,id2:line2,...
    pub fn parse_threads_from_frontmatter(&self) -> Vec<(String, usize)> {
        let (start, end) = match self.frontmatter_range {
            Some(range) => range,
            None => return Vec::new(),
        };
        
        for i in (start + 1)..end {
            let line = self.lines[i].trim();
            if line.starts_with("threads:") {
                let value = line[8..].trim();
                return value.split(',')
                    .filter(|s| !s.is_empty())
                    .filter_map(|s| {
                        let parts: Vec<&str> = s.split(':').collect();
                        if parts.len() == 2 {
                            if let Ok(line_num) = parts[1].parse() {
                                return Some((parts[0].to_string(), line_num));
                            }
                        }
                        None
                    })
                    .collect();
            }
        }
        
        Vec::new()
    }

    /// Finds a thread by its ID.
    pub fn find_thread_by_id(&self, id: &str) -> Option<&NoteThread> {
        self.threads.iter().find(|t| t.id == id)
    }


}
