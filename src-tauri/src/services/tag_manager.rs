//! Manager for folder-wide tag operations and aggregation.

use crate::models::note_session::NoteSession;
use crate::utils::tag_parser;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Service for handling high-level tag domain operations.
pub struct TagManager {
    /// Cached list of all unique tags across all notes, sorted by frequency.
    pub cached_tags: Option<Vec<String>>,
}

impl TagManager {
    /// Creates a new `TagManager`.
    pub fn new() -> Self {
        Self { cached_tags: None }
    }

    /// Clears the tag cache, forcing a re-scan on the next request.
    pub fn invalidate_cache(&mut self) {
        self.cached_tags = None;
    }

    /// Aggregates all unique tags from notes in the specified folder,
    /// sorted by usage frequency (descending) and alphabetical tie-breaker.
    pub fn get_all_tags(&mut self, notes_folder: &Path) -> Result<Vec<String>, String> {
        if let Some(tags) = &self.cached_tags {
            return Ok(tags.clone());
        }

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

        let result: Vec<String> = tags.into_iter().map(|(tag, _)| tag).collect();
        self.cached_tags = Some(result.clone());

        Ok(result)
    }

    /// Returns a filtered list of tag suggestions based on a search query.
    pub fn suggest_tags(
        &mut self,
        notes_folder: &Path,
        query: &str,
        exclude: &[String],
        limit: usize,
    ) -> Vec<String> {
        let all_tags = self.get_all_tags(notes_folder).unwrap_or_default();
        let search = query.trim().to_lowercase();

        all_tags
            .into_iter()
            .filter(|tag| {
                !exclude.contains(tag)
                    && (search.is_empty() || tag.to_lowercase().contains(&search))
            })
            .take(limit)
            .collect()
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
        } else if let Some((_, end)) = session.frontmatter_range {
            session.lines.insert(end, format!("tags: [{}]", tag));
            session.detect_frontmatter();
        } else {
            // Fallback: should not happen after ensure_frontmatter
            session.lines.insert(0, "---".to_string());
            session.lines.insert(1, format!("tags: [{}]", tag));
            session.lines.insert(2, "---".to_string());
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
