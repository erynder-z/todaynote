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
}
