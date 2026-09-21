//! Transient state for the active note being edited.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Represents parsed thread metadata from frontmatter.
#[derive(Debug, Clone)]
pub struct FrontmatterThread {
    pub id: String,
    pub relative_line: usize,
    pub pinned: bool,
}

/// Represents a named block or thread within a note, defined by a `!!!` thread marker.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteThread {
    /// Unique identifier for the thread.
    pub id: String,
    /// The display name of the thread (e.g., "Work").
    pub name: String,
    /// The absolute line index where the thread marker starts.
    pub start_line: usize,
    /// The absolute line index where the thread ends (exclusive).
    pub end_line: usize,
    /// Whether the thread is pinned.
    #[serde(default)]
    pub pinned: bool,
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

        // Build a map of line -> FrontmatterThread from frontmatter
        let mut id_map: std::collections::HashMap<usize, &FrontmatterThread> =
            std::collections::HashMap::new();
        for meta in &frontmatter_threads {
            let absolute_line = content_start + meta.relative_line;
            id_map.insert(absolute_line, meta);
        }

        let mut thread_idx = 0;
        for i in content_start..self.lines.len() {
            let line = &self.lines[i];
            // Only match thread identifiers (!!! followed by space)
            if line.starts_with("!!! ") {
                let name = line[4..].trim().to_string();

                if !name.is_empty() {
                    // Update previous thread's end_line
                    if let Some(prev) = self.threads.last_mut() {
                        prev.end_line = i;
                    }

                    // Use ID and pinned status from frontmatter if available, otherwise generate new UUID
                    let (id, pinned) = if let Some(existing) = id_map.get(&i) {
                        (existing.id.clone(), existing.pinned)
                    } else if let Some(existing) = frontmatter_threads.get(thread_idx) {
                        (existing.id.clone(), existing.pinned)
                    } else {
                        // Generate new UUID for this thread
                        (Uuid::new_v4().to_string(), false)
                    };

                    self.threads.push(NoteThread {
                        id,
                        name,
                        start_line: i,
                        end_line: self.lines.len(),
                        pinned,
                    });
                    thread_idx += 1;
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

    /// Updates the threads metadata in the frontmatter.
    /// Stores thread IDs, relative line positions, and optional pinned flags.
    pub fn update_threads_in_frontmatter(&mut self) {
        self.ensure_frontmatter();
        let (start, end) = self.frontmatter_range.unwrap();
        let content_start = self.get_content_start_index();

        // Build the threads metadata as a comma-separated list of id:line[:pinned] pairs
        let threads_meta: String = self
            .threads
            .iter()
            .map(|t| {
                let relative_line = if t.start_line >= content_start {
                    t.start_line - content_start
                } else {
                    0
                };
                if t.pinned {
                    format!("{}:{}:pinned", t.id, relative_line)
                } else {
                    format!("{}:{}", t.id, relative_line)
                }
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
            self.lines
                .insert(end, format!("{} {}", threads_key, threads_meta));
            self.frontmatter_range = Some((start, end + 1));
        }
    }

    /// Parses thread metadata from the frontmatter.
    /// Expected format: threads: id1:line1,id2:line2:pinned,...
    pub fn parse_threads_from_frontmatter(&self) -> Vec<FrontmatterThread> {
        let (start, end) = match self.frontmatter_range {
            Some(range) => range,
            None => return Vec::new(),
        };

        for i in (start + 1)..end {
            let line = self.lines[i].trim();
            if line.starts_with("threads:") {
                let value = line[8..].trim();
                return value
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .filter_map(|s| {
                        let parts: Vec<&str> = s.split(':').collect();
                        if parts.len() >= 2 {
                            if let Ok(line_num) = parts[1].parse() {
                                let pinned = parts
                                    .get(2)
                                    .map(|&f| f == "pinned" || f == "p" || f == "true" || f == "1")
                                    .unwrap_or(false);
                                return Some(FrontmatterThread {
                                    id: parts[0].to_string(),
                                    relative_line: line_num,
                                    pinned,
                                });
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

    /// Updates the last-modified date in the frontmatter to the current date.
    pub fn update_last_modified(&mut self) {
        use crate::utils::date::get_current_date;

        self.ensure_frontmatter();
        let (start, end) = self.frontmatter_range.unwrap();
        let current_date = get_current_date();

        // Check if last-modified already exists
        let last_modified_key = "last-modified:";
        let mut found = false;

        for i in (start + 1)..end {
            if self.lines[i].trim().starts_with(last_modified_key) {
                self.lines[i] = format!("{} {}", last_modified_key, current_date);
                found = true;
                break;
            }
        }

        // If not found, add it before the closing ---
        if !found && end > start + 1 {
            self.lines
                .insert(end, format!("{} {}", last_modified_key, current_date));
            self.frontmatter_range = Some((start, end + 1));
        }
    }
}

///
/// Unit Tests
///
#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::date::get_current_date;

    /// Builds a session with frontmatter already present so that line operations
    /// (which auto-trigger detect_threads → ensure_frontmatter) don't shift indices.
    fn make_session(content_lines: &[&str]) -> NoteSession {
        let mut lines = vec![
            "---".to_string(),
            "threads: ".to_string(),
            "---".to_string(),
        ];
        lines.extend(content_lines.iter().map(|s| s.to_string()));
        NoteSession {
            path: None,
            lines,
            frontmatter_range: Some((0, 2)),
            threads: Vec::new(),
        }
    }
    ///
    /// new / get_full_content tests
    ///
    #[test]
    fn test_new_is_empty() {
        let session = NoteSession::new();

        assert!(session.path.is_none());
        assert!(session.lines.is_empty());
        assert!(session.frontmatter_range.is_none());
        assert!(session.threads.is_empty());
        assert_eq!(session.get_full_content(), "");
    }
    ///
    /// load tests
    ///
    #[test]
    fn test_load_detects_frontmatter_and_threads() {
        let content = "---\ntags: [work]\n---\n!!! Work\nsome task\n!!! Personal\nanother task";
        let mut session = NoteSession::new();
        session.load(PathBuf::from("/tmp/note.md"), content.to_string());

        assert_eq!(session.path, Some(PathBuf::from("/tmp/note.md")));
        // detect_threads inserts a threads: line, expanding frontmatter.
        assert_eq!(session.frontmatter_range, Some((0, 3)));
        assert_eq!(session.threads.len(), 2);
        assert_eq!(session.threads[0].name, "Work");
        assert_eq!(session.threads[1].name, "Personal");
        // Thread end_line should be set to the next thread's start.
        assert_eq!(session.threads[0].end_line, session.threads[1].start_line);
    }
    ///
    /// detect_frontmatter tests
    ///
    #[test]
    fn test_detect_frontmatter_present() {
        let mut session = NoteSession::new();
        session.lines = vec![
            "---".into(),
            "tags: [a]".into(),
            "---".into(),
            "body".into(),
        ];
        session.detect_frontmatter();
        assert_eq!(session.frontmatter_range, Some((0, 2)));
    }

    #[test]
    fn test_detect_frontmatter_absent() {
        let mut session = NoteSession::new();
        session.lines = vec!["just body".into()];
        session.detect_frontmatter();
        assert!(session.frontmatter_range.is_none());
    }
    ///
    /// ensure_frontmatter tests
    ///
    #[test]
    fn test_ensure_frontmatter_creates_if_missing() {
        let mut session = NoteSession::new();
        session.lines = vec!["body".into()];
        session.ensure_frontmatter();
        assert_eq!(session.frontmatter_range, Some((0, 1)));
        assert_eq!(session.lines[0], "---");
        assert_eq!(session.lines[1], "---");
    }

    #[test]
    fn test_ensure_frontmatter_noop_if_present() {
        let mut session = NoteSession::new();
        session.lines = vec!["---".into(), "tags: [a]".into(), "---".into()];
        session.frontmatter_range = Some((0, 2));
        session.ensure_frontmatter();
        assert_eq!(session.lines.len(), 3);
    }
    ///
    /// get_metadata / find_metadata_line tests
    ///
    #[test]
    fn test_get_metadata_and_find() {
        let mut session = NoteSession::new();
        session.lines = vec![
            "---".into(),
            "tags: [work]".into(),
            "title: My Note".into(),
            "---".into(),
        ];
        session.frontmatter_range = Some((0, 3));

        let metadata = session.get_metadata();
        assert_eq!(metadata.get("tags"), Some(&"[work]".to_string()));
        assert_eq!(metadata.get("title"), Some(&"My Note".to_string()));

        assert_eq!(session.find_metadata_line("tags"), Some(1));
        assert_eq!(session.find_metadata_line("missing"), None);
    }

    #[test]
    fn test_get_metadata_empty_without_frontmatter() {
        let session = NoteSession::new();
        assert!(session.get_metadata().is_empty());
        assert!(session.find_metadata_line("tags").is_none());
    }
    ///
    /// line operations tests
    ///
    #[test]
    fn test_update_insert_delete_line() {
        let mut session = make_session(&["a", "b", "c"]);

        session.update_line(4, "B".into());
        assert_eq!(session.lines[4], "B");

        session.insert_line(3, "first".into());
        assert_eq!(session.lines[3], "first");
        assert_eq!(session.lines.len(), 7);

        session.delete_line(3);
        assert_eq!(session.lines[3], "a");
    }

    #[test]
    fn test_delete_line_range() {
        let mut session = make_session(&["a", "b", "c", "d"]);
        session.delete_line_range(4, 6);
        assert_eq!(session.lines[3], "a");
        assert_eq!(session.lines[4], "d");
    }
    ///
    /// relative index operations tests
    ///
    #[test]
    fn test_content_relative_indexing() {
        let mut session = make_session(&["body"]);

        assert_eq!(session.get_content_start_index(), 3);
        assert_eq!(session.to_absolute_index(0), 3);

        session.insert_content_line(0, "new".into());
        assert_eq!(session.lines[3], "new");

        session.update_content_line(1, "updated".into());
        assert_eq!(session.lines[4], "updated");

        session.delete_content_line(0);
        assert_eq!(session.lines[3], "updated");
    }
    ///
    /// ensure_trailing_empty_line tests
    ///
    #[test]
    fn test_ensure_trailing_empty_line_adds_padding() {
        let mut session = NoteSession::new();
        session.lines = vec!["content".into()];
        session.ensure_trailing_empty_line();
        assert!(session.lines.len() >= 3);
        assert_eq!(session.lines[0], "content");
    }

    #[test]
    fn test_ensure_trailing_empty_line_noop_when_sufficient() {
        let mut session = NoteSession::new();
        session.lines = vec!["content".into(), "".into(), "".into()];
        session.ensure_trailing_empty_line();
        assert_eq!(session.lines.len(), 3);
    }
    ///
    /// detect_threads tests
    ///
    #[test]
    fn test_detect_threads_assigns_uuid_ids() {
        let mut session = make_session(&["!!! Work", "task", "!!! Personal", "task2"]);
        session.detect_threads();

        assert_eq!(session.threads.len(), 2);
        assert!(!session.threads[0].id.is_empty());
        assert_ne!(session.threads[0].id, session.threads[1].id);
        assert_eq!(session.threads[1].end_line, session.lines.len());
    }

    #[test]
    fn test_detect_threads_ignores_empty_name() {
        let mut session = NoteSession::new();
        session.lines = vec!["!!! ".into(), "!!! Real".into()];
        session.detect_threads();
        assert_eq!(session.threads.len(), 1);
        assert_eq!(session.threads[0].name, "Real");
    }
    ///
    /// parse_threads_from_frontmatter / update_threads_in_frontmatter tests
    ///
    #[test]
    fn test_parse_threads_from_frontmatter() {
        let mut session = NoteSession::new();
        session.lines = vec![
            "---".into(),
            "threads: abc:0,def:2:pinned".into(),
            "---".into(),
        ];
        session.frontmatter_range = Some((0, 2));

        let threads = session.parse_threads_from_frontmatter();
        assert_eq!(threads.len(), 2);
        assert_eq!(threads[0].id, "abc");
        assert_eq!(threads[0].relative_line, 0);
        assert!(!threads[0].pinned);
        assert_eq!(threads[1].id, "def");
        assert_eq!(threads[1].relative_line, 2);
        assert!(threads[1].pinned);
    }

    #[test]
    fn test_parse_threads_from_frontmatter_empty() {
        let session = NoteSession::new();
        assert!(session.parse_threads_from_frontmatter().is_empty());
    }

    #[test]
    fn test_update_threads_in_frontmatter_writes_line() {
        let mut session = NoteSession::new();
        session.lines = vec![
            "---".into(),
            "tags: [a]".into(),
            "---".into(),
            "!!! Work".into(),
        ];
        session.frontmatter_range = Some((0, 2));
        session.detect_threads();

        let threads_line = session.lines.iter().find(|l| l.starts_with("threads:"));
        assert!(threads_line.is_some(), "threads: line should be added");
    }
    ///
    /// find_thread_by_id tests
    ///
    #[test]
    fn test_find_thread_by_id() {
        let mut session = NoteSession::new();
        session.lines = vec!["!!! Work".into(), "task".into()];
        session.detect_threads();

        let id = session.threads[0].id.clone();
        let found = session.find_thread_by_id(&id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Work");

        assert!(session.find_thread_by_id("nonexistent").is_none());
    }
    ///
    /// update_last_modified tests
    ///
    #[test]
    fn test_update_last_modified_adds_and_updates() {
        let mut session = NoteSession::new();
        session.lines = vec![
            "---".into(),
            "tags: [a]".into(),
            "---".into(),
            "body".into(),
        ];
        session.frontmatter_range = Some((0, 2));

        session.update_last_modified();
        let found = session.find_metadata_line("last-modified");
        assert!(found.is_some(), "last-modified should be added");
        assert_eq!(
            session.lines[found.unwrap()],
            format!("last-modified: {}", get_current_date())
        );

        // A second call should update in place, not duplicate.
        let len_before = session.lines.len();
        session.update_last_modified();
        assert_eq!(session.lines.len(), len_before);
    }
}
