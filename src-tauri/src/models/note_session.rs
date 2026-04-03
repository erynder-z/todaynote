//! Transient state for the active note being edited.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Represents a named block or section within a note, typically defined by a [#] marker.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteSection {
    /// The display name of the section (e.g., "Work").
    pub name: String,
    /// The absolute line index where the section header starts.
    pub start_line: usize,
    /// The absolute line index where the section ends (exclusive).
    pub end_line: usize,
    /// The header level (1 for [#], 2 for [##], etc.).
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
    /// List of detected sections in the note.
    pub sections: Vec<NoteSection>,
}

impl NoteSession {
    /// Creates a new, empty note session.
    pub fn new() -> Self {
        Self {
            path: None,
            lines: Vec::new(),
            frontmatter_range: None,
            sections: Vec::new(),
        }
    }

    /// Loads the content and path of a note into the session.
    pub fn load(&mut self, path: PathBuf, content: String) {
        self.path = Some(path);
        self.lines = content.split('\n').map(|s| s.to_string()).collect();
        self.detect_frontmatter();
        self.detect_sections();
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
    /// Scans the lines to identify sections based on [#] markers.
    /// Marker format: [#] Header Name
    pub fn detect_sections(&mut self) {
        self.sections.clear();
        let content_start = self.get_content_start_index();

        for i in content_start..self.lines.len() {
            let line = &self.lines[i];
            if line.starts_with("[#") {
                if let Some(close_bracket) = line.find(']') {
                    let level_str = &line[1..close_bracket];
                    if level_str.chars().all(|c| c == '#') {
                        let level = level_str.len();
                        let name = line[close_bracket + 1..].trim().to_string();

                        if !name.is_empty() {
                            // Update previous section's end_line
                            if let Some(prev) = self.sections.last_mut() {
                                prev.end_line = i;
                            }

                            self.sections.push(NoteSection {
                                name,
                                start_line: i,
                                end_line: self.lines.len(),
                                level,
                            });
                        }
                    }
                }
            }
        }
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
            self.detect_sections();
        }
    }

    /// Inserts a new line at the specified index.
    pub fn insert_line(&mut self, index: usize, content: String) {
        if index <= self.lines.len() {
            self.lines.insert(index, content);
            self.detect_frontmatter();
            self.detect_sections();
        }
    }

    /// Removes a specific line from the session.
    pub fn delete_line(&mut self, index: usize) {
        if index < self.lines.len() && self.lines.len() > 1 {
            self.lines.remove(index);
            self.detect_frontmatter();
            self.detect_sections();
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

    /// Returns a slice of lines that represent the actual content (excluding frontmatter).
    pub fn get_content_lines(&self) -> Vec<String> {
        let start = self.get_content_start_index();
        if start < self.lines.len() {
            self.lines[start..].to_vec()
        } else {
            Vec::new()
        }
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

    /// Reconstructs the full note content by joining the lines with newlines.
    pub fn get_full_content(&self) -> String {
        self.lines.join("\n")
    }
}
