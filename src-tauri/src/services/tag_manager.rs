//! Manager for folder-wide tag operations and aggregation.

use crate::models::note_session::NoteSession;
use crate::utils::tag_parser;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Service for handling high-level tag domain operations.
pub struct TagManager;

impl TagManager {
    /// Creates a new `TagManager`.
    pub fn new() -> Self {
        Self
    }

    /// Aggregates all unique tags from notes in the specified folder,
    /// sorted by usage frequency (descending) and alphabetical tie-breaker.
    pub fn get_all_tags(&self, notes_folder: &Path) -> Result<Vec<String>, String> {
        if !notes_folder.exists() {
            return Ok(vec![]);
        }

        let entries =
            fs::read_dir(notes_folder).map_err(|e| format!("Failed to read directory: {}", e))?;

        let mut tag_counts: HashMap<String, usize> = HashMap::new();

        for entry in entries.filter_map(|e| e.ok()) {
            let file_name = entry.file_name().into_string().unwrap_or_default();
            if file_name.ends_with(".md") && !file_name.starts_with(".") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    let tags = tag_parser::parse_tags_from_content(&content);
                    for tag in tags {
                        *tag_counts.entry(tag).or_insert(0) += 1;
                    }
                }
            }
        }

        let mut tags: Vec<(String, usize)> = tag_counts.into_iter().collect();
        // Sort by frequency (desc), then alphabetically (asc)
        tags.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        Ok(tags.into_iter().map(|(tag, _)| tag).collect())
    }

    /// Returns tags from the session metadata.
    pub fn get_tags_from_session(&self, session: &NoteSession) -> Vec<String> {
        let metadata = session.get_metadata();
        if let Some(tags_str) = metadata.get("tags") {
            tag_parser::parse_tags_from_yaml_value(tags_str)
        } else {
            Vec::new()
        }
    }

    /// Adds a tag to the session frontmatter.
    pub fn add_tag_to_session(&self, session: &mut NoteSession, tag: String) {
        session.ensure_frontmatter();

        if let Some(absolute_idx) = session.find_metadata_line("tags") {
            let mut tags = self.get_tags_from_session(session);
            if !tags.contains(&tag) {
                tags.push(tag);
                session.lines[absolute_idx] = format!("tags: [{}]", tags.join(", "));
            }
        } else {
            let (_, end) = session.frontmatter_range.expect("Frontmatter should exist");
            session.lines.insert(end, format!("tags: [{}]", tag));
            session.detect_frontmatter();
        }
    }

    /// Removes a tag from the session frontmatter.
    pub fn remove_tag_from_session(&self, session: &mut NoteSession, tag: String) {
        if let Some(absolute_idx) = session.find_metadata_line("tags") {
            let mut tags = self.get_tags_from_session(session);
            if let Some(pos) = tags.iter().position(|t| t == &tag) {
                tags.remove(pos);
                session.lines[absolute_idx] = format!("tags: [{}]", tags.join(", "));
            }
        }
    }
}
