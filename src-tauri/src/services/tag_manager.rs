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

    /// Internal helper to update the tags line in session frontmatter.
    fn set_tags_in_session(&self, session: &mut NoteSession, tags: Vec<String>) {
        if tags.is_empty() {
            if let Some(idx) = session.find_metadata_line("tags") {
                session.lines.remove(idx);
                session.detect_frontmatter();
            }
            return;
        }

        session.ensure_frontmatter();

        let tag_line = format!("tags: [{}]", tags.join(", "));

        if let Some(absolute_idx) = session.find_metadata_line("tags") {
            session.lines[absolute_idx] = tag_line;
        } else if let Some((_, end)) = session.frontmatter_range {
            session.lines.insert(end, tag_line);
            session.detect_frontmatter();
        }
    }

    /// Adds a tag to the session frontmatter.
    pub fn add_tag_to_session(&self, session: &mut NoteSession, tag: String) {
        let mut tags = self.get_tags_from_session(session);
        if !tags.contains(&tag) {
            tags.push(tag);
            self.set_tags_in_session(session, tags);
        }
    }

    /// Removes a tag from the session frontmatter.
    pub fn remove_tag_from_session(&self, session: &mut NoteSession, tag: String) {
        let mut tags = self.get_tags_from_session(session);
        if let Some(pos) = tags.iter().position(|t| t == &tag) {
            tags.remove(pos);
            self.set_tags_in_session(session, tags);
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
    /// Construction tests
    ///
    #[test]
    fn test_new_tag_manager() {
        let manager = TagManager::new();
        assert!(manager.cached_tags.is_none());
    }

    ///
    /// Cache invalidation tests
    ///
    #[test]
    fn test_invalidate_cache_clears_tags() {
        let mut manager = TagManager::new();
        manager.cached_tags = Some(vec!["tag1".to_string(), "tag2".to_string()]);
        manager.invalidate_cache();
        assert!(manager.cached_tags.is_none());
    }

    ///
    /// get_all_tags tests
    ///
    #[test]
    fn test_get_all_tags_nonexistent_folder() {
        let mut manager = TagManager::new();
        let path = Path::new("/nonexistent/path");
        let result = manager.get_all_tags(path);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_get_all_tags_empty_folder() {
        let mut manager = TagManager::new();
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();
        let result = manager.get_all_tags(path);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_get_all_tags_with_markdown_files() {
        let mut manager = TagManager::new();
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        fs::write(
            path.join("2024-01-01.md"),
            "---\ntags: [work, urgent]\n---\ncontent",
        )
        .unwrap();
        fs::write(
            path.join("2024-01-02.md"),
            "---\ntags: [work, personal]\n---\ncontent",
        )
        .unwrap();

        let result = manager.get_all_tags(path);
        assert!(result.is_ok());
        let tags = result.unwrap();
        assert_eq!(tags.len(), 3);
        assert!(tags.contains(&"work".to_string()));
        assert!(tags.contains(&"urgent".to_string()));
        assert!(tags.contains(&"personal".to_string()));
    }

    #[test]
    fn test_get_all_tags_sorted_by_frequency() {
        let mut manager = TagManager::new();
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        fs::write(path.join("2024-01-01.md"), "---\ntags: [work]\n---\n").unwrap();
        fs::write(path.join("2024-01-02.md"), "---\ntags: [work]\n---\n").unwrap();
        fs::write(
            path.join("2024-01-03.md"),
            "---\ntags: [work, personal]\n---\n",
        )
        .unwrap();
        fs::write(path.join("2024-01-04.md"), "---\ntags: [personal]\n---\n").unwrap();

        let result = manager.get_all_tags(path);
        assert!(result.is_ok());
        let tags = result.unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0], "work");
        assert_eq!(tags[1], "personal");
    }

    #[test]
    fn test_get_all_tags_alphabetical_tiebreaker() {
        let mut manager = TagManager::new();
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        fs::write(path.join("2024-01-01.md"), "---\ntags: [zebra]\n---\n").unwrap();
        fs::write(path.join("2024-01-02.md"), "---\ntags: [apple]\n---\n").unwrap();

        let result = manager.get_all_tags(path);
        assert!(result.is_ok());
        let tags = result.unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0], "apple");
        assert_eq!(tags[1], "zebra");
    }

    #[test]
    fn test_get_all_tags_ignores_non_md_files() {
        let mut manager = TagManager::new();
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        fs::write(path.join("2024-01-01.md"), "---\ntags: [work]\n---\n").unwrap();
        fs::write(path.join("2024-01-02.txt"), "---\ntags: [personal]\n---\n").unwrap();
        fs::write(path.join(".hidden.md"), "---\ntags: [hidden]\n---\n").unwrap();

        let result = manager.get_all_tags(path);
        assert!(result.is_ok());
        let tags = result.unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0], "work");
    }

    #[test]
    fn test_get_all_tags_caches_results() {
        let mut manager = TagManager::new();
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        fs::write(path.join("2024-01-01.md"), "---\ntags: [work]\n---\n").unwrap();

        let result1 = manager.get_all_tags(path);
        assert!(result1.is_ok());
        assert!(manager.cached_tags.is_some());

        let result2 = manager.get_all_tags(path);
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), result2.unwrap());
    }

    ///
    /// suggest_tags tests
    ///
    #[test]
    fn test_suggest_tags_empty_query() {
        let mut manager = TagManager::new();
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        fs::write(
            path.join("2024-01-01.md"),
            "---\ntags: [work, personal]\n---\n",
        )
        .unwrap();

        let suggestions = manager.suggest_tags(path, "", &[], 10);
        assert_eq!(suggestions.len(), 2);
        assert!(suggestions.contains(&"work".to_string()));
        assert!(suggestions.contains(&"personal".to_string()));
    }

    #[test]
    fn test_suggest_tags_with_query() {
        let mut manager = TagManager::new();
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        fs::write(
            path.join("2024-01-01.md"),
            "---\ntags: [work, personal]\n---\n",
        )
        .unwrap();

        let suggestions = manager.suggest_tags(path, "work", &[], 10);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0], "work");
    }

    #[test]
    fn test_suggest_tags_excludes_tags() {
        let mut manager = TagManager::new();
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        fs::write(
            path.join("2024-01-01.md"),
            "---\ntags: [work, personal]\n---\n",
        )
        .unwrap();

        let suggestions = manager.suggest_tags(path, "", &["work".to_string()], 10);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0], "personal");
    }

    #[test]
    fn test_suggest_tags_respects_limit() {
        let mut manager = TagManager::new();
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        fs::write(
            path.join("2024-01-01.md"),
            "---\ntags: [a, b, c, d, e]\n---\n",
        )
        .unwrap();

        let suggestions = manager.suggest_tags(path, "", &[], 3);
        assert!(suggestions.len() <= 3);
    }

    #[test]
    fn test_suggest_tags_case_insensitive() {
        let mut manager = TagManager::new();
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        fs::write(path.join("2024-01-01.md"), "---\ntags: [Work]\n---\n").unwrap();

        let suggestions = manager.suggest_tags(path, "WORK", &[], 10);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0], "Work");
    }

    ///
    /// Session tag operations tests
    ///
    #[test]
    fn test_get_tags_from_session_with_tags() {
        let manager = TagManager::new();
        let mut session = NoteSession::new();
        session.lines = vec![
            "---".into(),
            "tags: [work, personal]".into(),
            "---".into(),
            "content".into(),
        ];
        session.frontmatter_range = Some((0, 2));

        let tags = manager.get_tags_from_session(&session);
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"work".to_string()));
        assert!(tags.contains(&"personal".to_string()));
    }

    #[test]
    fn test_get_tags_from_session_no_tags() {
        let manager = TagManager::new();
        let session = NoteSession::new();

        let tags = manager.get_tags_from_session(&session);
        assert!(tags.is_empty());
    }

    #[test]
    fn test_get_tags_from_session_no_frontmatter() {
        let manager = TagManager::new();
        let mut session = NoteSession::new();
        session.lines = vec!["content".into()];

        let tags = manager.get_tags_from_session(&session);
        assert!(tags.is_empty());
    }

    #[test]
    fn test_add_tag_to_session_new_tag() {
        let manager = TagManager::new();
        let mut session = NoteSession::new();
        session.lines = vec![
            "---".into(),
            "title: Test".into(),
            "---".into(),
            "content".into(),
        ];
        session.frontmatter_range = Some((0, 2));

        manager.add_tag_to_session(&mut session, "work".to_string());

        let tags = manager.get_tags_from_session(&session);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0], "work");
    }

    #[test]
    fn test_add_tag_to_session_existing_tag() {
        let manager = TagManager::new();
        let mut session = NoteSession::new();
        session.lines = vec![
            "---".into(),
            "tags: [work]".into(),
            "---".into(),
            "content".into(),
        ];
        session.frontmatter_range = Some((0, 2));

        manager.add_tag_to_session(&mut session, "work".to_string());

        let tags = manager.get_tags_from_session(&session);
        assert_eq!(tags.len(), 1);
    }

    #[test]
    fn test_add_tag_to_session_no_frontmatter() {
        let manager = TagManager::new();
        let mut session = NoteSession::new();

        manager.add_tag_to_session(&mut session, "work".to_string());

        let tags = manager.get_tags_from_session(&session);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0], "work");
        assert!(session.frontmatter_range.is_some());
    }

    #[test]
    fn test_remove_tag_from_session() {
        let manager = TagManager::new();
        let mut session = NoteSession::new();
        session.lines = vec![
            "---".into(),
            "tags: [work, personal]".into(),
            "---".into(),
            "content".into(),
        ];
        session.frontmatter_range = Some((0, 2));

        manager.remove_tag_from_session(&mut session, "work".to_string());

        let tags = manager.get_tags_from_session(&session);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0], "personal");
    }

    #[test]
    fn test_remove_tag_from_session_nonexistent() {
        let manager = TagManager::new();
        let mut session = NoteSession::new();
        session.lines = vec![
            "---".into(),
            "tags: [work]".into(),
            "---".into(),
            "content".into(),
        ];
        session.frontmatter_range = Some((0, 2));

        manager.remove_tag_from_session(&mut session, "nonexistent".to_string());

        let tags = manager.get_tags_from_session(&session);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0], "work");
    }

    #[test]
    fn test_remove_tag_from_session_last_tag() {
        let manager = TagManager::new();
        let mut session = NoteSession::new();
        session.lines = vec![
            "---".into(),
            "tags: [work]".into(),
            "---".into(),
            "content".into(),
        ];
        session.frontmatter_range = Some((0, 2));

        manager.remove_tag_from_session(&mut session, "work".to_string());

        let tags = manager.get_tags_from_session(&session);
        assert!(tags.is_empty());
        assert!(session.find_metadata_line("tags").is_none());
    }
}
