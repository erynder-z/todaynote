//! Transient state for the active note being edited.

use std::collections::HashMap;
use std::path::PathBuf;

/// Represents an active note session, maintaining an in-memory
/// representation of the file's lines for real-time editing.
pub struct NoteSession {
    /// File path of the currently open note.
    pub path: Option<PathBuf>,
    /// Each line of the note as an individual string in the vector.
    pub lines: Vec<String>,
    /// The start and end indices (inclusive) of the YAML frontmatter block.
    pub frontmatter_range: Option<(usize, usize)>,
}

impl NoteSession {
    /// Creates a new, empty note session.
    pub fn new() -> Self {
        Self {
            path: None,
            lines: Vec::new(),
            frontmatter_range: None,
        }
    }

    /// Loads the content and path of a note into the session.
    pub fn load(&mut self, path: PathBuf, content: String) {
        self.path = Some(path);
        self.lines = content.split('\n').map(|s| s.to_string()).collect();
        self.detect_frontmatter();
    }

    /// Detects YAML frontmatter (delimited by '---' on the first line and another '---').
    fn detect_frontmatter(&mut self) {
        if self.lines.len() < 2 {
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

    /// Ensures that the frontmatter block exists.
    fn ensure_frontmatter(&mut self) {
        if self.frontmatter_range.is_none() {
            self.lines.insert(0, "---".to_string());
            self.lines.insert(1, "---".to_string());
            self.frontmatter_range = Some((0, 1));
        }
    }

    /// Finds the index of a metadata line starting with the given key.
    fn find_metadata_line(&self, key: &str) -> Option<usize> {
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

    /// Parses tags from the metadata if they exist.
    /// Expects a format like `[tag1, tag2]` or just a comma-separated list.
    pub fn get_tags(&self) -> Vec<String> {
        let metadata = self.get_metadata();
        if let Some(tags_str) = metadata.get("tags") {
            let trimmed = tags_str.trim();
            if trimmed.is_empty() {
                return Vec::new();
            }

            // Remove brackets if present: [tag1, tag2] -> tag1, tag2
            let inner = if trimmed.starts_with('[') && trimmed.ends_with(']') {
                &trimmed[1..trimmed.len() - 1]
            } else {
                trimmed
            };

            return inner
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        Vec::new()
    }

    /// Replaces the content of a specific line in the session.
    pub fn update_line(&mut self, index: usize, content: String) {
        if index < self.lines.len() {
            self.lines[index] = content;
            // If the first line changed, or we are within a frontmatter block, re-detect
            if index == 0 || self.frontmatter_range.is_some() {
                self.detect_frontmatter();
            }
        }
    }

    /// Inserts a new line at the specified index.
    pub fn insert_line(&mut self, index: usize, content: String) {
        if index <= self.lines.len() {
            self.lines.insert(index, content);
            self.detect_frontmatter();
        }
    }

    /// Removes a specific line from the session.
    pub fn delete_line(&mut self, index: usize) {
        if index < self.lines.len() && self.lines.len() > 1 {
            self.lines.remove(index);
            self.detect_frontmatter();
        }
    }

    /// Reconstructs the full note content by joining the lines with newlines.
    pub fn get_full_content(&self) -> String {
        self.lines.join("\n")
    }

    /// Adds a tag to the note's frontmatter.
    /// Creates frontmatter if it doesn't exist.
    pub fn add_tag(&mut self, tag: String) {
        self.ensure_frontmatter();

        if let Some(absolute_idx) = self.find_metadata_line("tags") {
            let mut tags = self.get_tags();
            if !tags.contains(&tag) {
                tags.push(tag);
                self.lines[absolute_idx] = format!("tags: [{}]", tags.join(", "));
            }
        } else {
            let (_, end) = self.frontmatter_range.expect("Frontmatter should exist");
            self.lines.insert(end, format!("tags: [{}]", tag));
            self.detect_frontmatter();
        }
    }
}
