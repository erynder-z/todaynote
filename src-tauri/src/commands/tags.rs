//! Tauri commands for tag-related operations.

use crate::commands::notes::reconstruct_full_content;
use crate::models::app_state::AppState;
use crate::models::note_session::NoteSession;
use crate::models::response_types::NoteContentResponse;
use crate::services::tag_manager::TagManager;
use std::fs;
use tauri::State;

/// Helper to sync frontend content, perform a tag operation, and save the result.
async fn perform_tag_operation<F>(
    current_content: String,
    state: State<'_, AppState>,
    operation: F,
) -> Result<NoteContentResponse, String>
where
    F: FnOnce(&TagManager, &mut NoteSession),
{
    let mut session = state.note_session()?;

    let path = session
        .path
        .clone()
        .ok_or_else(|| "No active note session".to_string())?;

    // Sync session with frontend content first
    // Use the session's current frontmatter combined with the frontend's content
    // to ensure we don't lose any backend-made changes (like thread ID comments)
    let full_content = if let Some((_, end)) = session.frontmatter_range {
        let frontmatter_lines = &session.lines[..=end];
        let frontmatter = frontmatter_lines.join("\n");
        format!("{}\n{}", frontmatter, current_content)
    } else {
        // No frontmatter in session, use reconstruct_full_content as fallback
        reconstruct_full_content(&path, &current_content)?
    };
    session.load(path.clone(), full_content);

    let mut tag_manager = state.tag_manager()?;
    operation(&tag_manager, &mut session);
    tag_manager.invalidate_cache();

    session.update_last_modified();
    let full_content = session.get_full_content();
    fs::write(&path, &full_content).map_err(|e| format!("Failed to save note: {}", e))?;

    let note_manager = state.note_manager()?;
    Ok(NoteContentResponse::from_session(
        &session,
        &note_manager,
        &tag_manager,
    ))
}

/// Adds a tag to the current note session and writes it to disk.
#[tauri::command]
pub async fn add_note_tag(
    tag: String,
    current_content: String,
    state: State<'_, AppState>,
) -> Result<NoteContentResponse, String> {
    perform_tag_operation(current_content, state, |tm, session| {
        tm.add_tag_to_session(session, tag);
    })
    .await
}

/// Removes a tag from the current note session and writes it to disk.
#[tauri::command]
pub async fn remove_note_tag(
    tag: String,
    current_content: String,
    state: State<'_, AppState>,
) -> Result<NoteContentResponse, String> {
    perform_tag_operation(current_content, state, |tm, session| {
        tm.remove_tag_from_session(session, tag);
    })
    .await
}

/// Returns all unique tags from all notes, sorted by usage frequency.
#[tauri::command]
pub async fn get_all_tags(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let folder_path = {
        let note_manager = state.note_manager()?;
        note_manager.notes_folder.clone()
    };
    let mut tag_manager = state.tag_manager()?;
    tag_manager.get_all_tags(&folder_path)
}

/// Returns a filtered list of tag suggestions.
#[tauri::command]
pub async fn get_tag_suggestions(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let folder_path = {
        let note_manager = state.note_manager()?;
        note_manager.notes_folder.clone()
    };

    let session = state.note_session()?;
    let mut tag_manager = state.tag_manager()?;

    let active_tags = tag_manager.get_tags_from_session(&session);
    Ok(tag_manager.suggest_tags(&folder_path, &query, &active_tags, 20))
}

///
/// Unit Tests
///
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::note_session::NoteSession;
    use crate::services::note_manager::NoteManager;
    use crate::services::tag_manager::TagManager;
    use std::fs;
    use tempfile::tempdir;

    /// Creates a NoteManager backed by a temp dir populated with the given (filename, content) pairs.
    fn setup_notes(notes: &[(&str, &str)]) -> (tempfile::TempDir, NoteManager) {
        let dir = tempdir().expect("Failed to create temp dir");
        let note_manager = NoteManager::new(dir.path().to_path_buf(), "en".to_string());
        for (filename, content) in notes {
            fs::write(dir.path().join(filename), content).expect("Failed to write test note");
        }
        (dir, note_manager)
    }

    /// Loads a note file from disk into a fresh NoteSession.
    fn load_session(dir: &tempfile::TempDir, filename: &str) -> NoteSession {
        let path = dir.path().join(filename);
        let content = fs::read_to_string(&path).expect("Failed to read note");
        let mut session = NoteSession::new();
        session.load(path, content);
        session
    }

    ///
    /// Add Tag Tests
    ///
    #[test]
    fn test_add_tag_creates_frontmatter() {
        let (dir, _) = setup_notes(&[("2024-01-01.md", "# Just content")]);
        let mut session = load_session(&dir, "2024-01-01.md");
        let tm = TagManager::new();

        tm.add_tag_to_session(&mut session, "rust".to_string());

        assert_eq!(
            session.get_metadata().get("tags"),
            Some(&"[rust]".to_string()),
            "Should create frontmatter with the tag"
        );
    }

    #[test]
    fn test_add_tag_appends_and_dedupes() {
        let (dir, _) = setup_notes(&[("2024-01-01.md", "---\ntags: [rust]\n---\n# Content")]);
        let mut session = load_session(&dir, "2024-01-01.md");
        let tm = TagManager::new();

        tm.add_tag_to_session(&mut session, "programming".to_string());
        tm.add_tag_to_session(&mut session, "rust".to_string());

        assert_eq!(
            session.get_metadata().get("tags"),
            Some(&"[rust, programming]".to_string()),
            "Should append new tag and skip duplicate"
        );
    }

    ///
    /// Remove Tag Tests
    ///
    #[test]
    fn test_remove_tag_removes_and_cleans_line() {
        let (dir, _) = setup_notes(&[(
            "2024-01-01.md",
            "---\ntags: [rust, programming]\n---\n# Content",
        )]);
        let mut session = load_session(&dir, "2024-01-01.md");
        let tm = TagManager::new();

        tm.remove_tag_from_session(&mut session, "rust".to_string());
        assert_eq!(
            session.get_metadata().get("tags"),
            Some(&"[programming]".to_string()),
            "Should remove the tag"
        );

        tm.remove_tag_from_session(&mut session, "programming".to_string());
        assert!(
            session.find_metadata_line("tags").is_none(),
            "tags line should be removed when empty"
        );

        // No-op on absent tag
        tm.remove_tag_from_session(&mut session, "python".to_string());
    }

    ///
    /// Get Tags From Session Tests
    ///
    #[test]
    fn test_get_tags_from_session_variants() {
        let tm = TagManager::new();

        // Parses existing tags
        let (dir, _) = setup_notes(&[(
            "2024-01-01.md",
            "---\ntags: [rust, programming]\n---\n# Content",
        )]);
        let session = load_session(&dir, "2024-01-01.md");
        assert_eq!(
            tm.get_tags_from_session(&session),
            vec!["rust", "programming"]
        );

        // Empty when no frontmatter
        let (dir, _) = setup_notes(&[("2024-01-02.md", "# Just content")]);
        let session = load_session(&dir, "2024-01-02.md");
        assert!(tm.get_tags_from_session(&session).is_empty());

        // Empty when no tags key
        let (dir, _) =
            setup_notes(&[("2024-01-03.md", "---\ncreated: 2024-01-03\n---\n# Content")]);
        let session = load_session(&dir, "2024-01-03.md");
        assert!(tm.get_tags_from_session(&session).is_empty());

        // Empty when tags value is []
        let (dir, _) = setup_notes(&[("2024-01-04.md", "---\ntags: []\n---\n# Content")]);
        let session = load_session(&dir, "2024-01-04.md");
        assert!(tm.get_tags_from_session(&session).is_empty());
    }

    ///
    /// Get All Tags Tests
    ///
    #[test]
    fn test_get_all_tags_aggregates_sorts_and_ignores_extraneous() {
        let (dir, _) = setup_notes(&[
            (
                "2024-01-01.md",
                "---\ntags: [rust, programming]\n---\nNote 1",
            ),
            ("2024-01-02.md", "---\ntags: [rust, testing]\n---\nNote 2"),
            ("2024-01-03.md", "---\ntags: [python]\n---\nNote 3"),
            (".hidden.md", "---\ntags: [secret]\n---\nHidden"),
            ("readme.txt", "---\ntags: [ignored]\n---\nNot a note"),
        ]);
        let mut tm = TagManager::new();

        let tags = tm.get_all_tags(dir.path()).expect("get_all_tags failed");

        // rust (2) first by frequency, then alpha for the rest; dotfiles and non-md excluded
        assert_eq!(tags, vec!["rust", "programming", "python", "testing"]);
    }

    #[test]
    fn test_get_all_tags_cache_and_invalidation() {
        let (dir, _) = setup_notes(&[("2024-01-01.md", "---\ntags: [rust]\n---\nNote 1")]);
        let mut tm = TagManager::new();

        let first = tm.get_all_tags(dir.path()).expect("First call failed");
        fs::write(
            dir.path().join("2024-01-02.md"),
            "---\ntags: [python]\n---\nNote 2",
        )
        .expect("Failed to write new note");

        // Cached: new note's tag not visible
        let second = tm.get_all_tags(dir.path()).expect("Second call failed");
        assert_eq!(first, second, "Cache should return stale result");
        assert_eq!(second, vec!["rust"]);

        // After invalidation: new tag appears
        tm.invalidate_cache();
        let third = tm.get_all_tags(dir.path()).expect("Third call failed");
        assert!(
            third.contains(&"python".to_string()),
            "Rescan should find new tag"
        );
    }

    ///
    /// Suggest Tags Tests
    ///
    #[test]
    fn test_suggest_tags_filters_excludes_and_limits() {
        let (dir, _) = setup_notes(&[(
            "2024-01-01.md",
            "---\ntags: [rust, ruby, python, Pascal]\n---\nNote 1",
        )]);
        let mut tm = TagManager::new();

        // Case-insensitive substring filter
        assert_eq!(
            tm.suggest_tags(dir.path(), "ru", &[], 20),
            vec!["ruby", "rust"],
            "Should filter by substring, sorted by frequency then alpha"
        );

        assert_eq!(
            tm.suggest_tags(dir.path(), "pascal", &[], 20),
            vec!["Pascal"],
            "Query should match case-insensitively"
        );

        // Excludes active tags
        let suggestions = tm.suggest_tags(
            dir.path(),
            "",
            &["rust".to_string(), "python".to_string()],
            20,
        );
        assert!(
            !suggestions.contains(&"rust".to_string()),
            "Active tags should be excluded"
        );
        assert_eq!(suggestions, vec!["Pascal", "ruby"]);

        // Respects limit
        assert_eq!(
            tm.suggest_tags(dir.path(), "", &[], 2).len(),
            2,
            "Should respect the limit"
        );
    }

    ///
    /// NoteContentResponse Integration Tests
    ///
    #[test]
    fn test_from_session_reflects_tag_changes() {
        let (dir, note_manager) = setup_notes(&[(
            "2024-01-01.md",
            "---\ncreated: 2024-01-01\nlast-modified: 2024-01-01\ntags: [rust, python]\n---\n# Heading",
        )]);
        let tm = TagManager::new();

        // Remove a tag
        let mut session = load_session(&dir, "2024-01-01.md");
        tm.remove_tag_from_session(&mut session, "rust".to_string());
        let resp = NoteContentResponse::from_session(&session, &note_manager, &tm);
        assert_eq!(resp.metadata.tags, vec!["python"], "Should reflect removal");
        assert!(
            resp.content.contains("# Heading"),
            "Content should include body"
        );
        assert!(
            !resp.content.contains("---"),
            "Content should exclude frontmatter"
        );

        // Add a tag back
        tm.add_tag_to_session(&mut session, "rust".to_string());
        let resp = NoteContentResponse::from_session(&session, &note_manager, &tm);
        assert_eq!(
            resp.metadata.tags,
            vec!["python", "rust"],
            "Should reflect addition"
        );
        assert!(
            resp.path.ends_with("2024-01-01.md"),
            "Path should point to the note"
        );
    }

    ///
    /// Tag Persistence Tests
    ///
    #[test]
    fn test_tag_persistence_on_disk() {
        let (dir, _) = setup_notes(&[(
            "2024-01-01.md",
            "---\ncreated: 2024-01-01\nlast-modified: 2024-01-01\ntags: [rust, python]\n---\n# Content",
        )]);
        let path = dir.path().join("2024-01-01.md");
        let tm = TagManager::new();

        // Remove rust, write to disk
        let mut session = load_session(&dir, "2024-01-01.md");
        tm.remove_tag_from_session(&mut session, "rust".to_string());
        fs::write(&path, session.get_full_content()).expect("Failed to write note");
        let reread = fs::read_to_string(&path).expect("Failed to re-read note");
        assert!(
            reread.contains("tags: [python]"),
            "Removed tag should be gone"
        );
        assert!(!reread.contains("rust"), "Removed tag should not appear");

        // Add rust back, write to disk
        tm.add_tag_to_session(&mut session, "rust".to_string());
        fs::write(&path, session.get_full_content()).expect("Failed to write note");
        let reread = fs::read_to_string(&path).expect("Failed to re-read note");
        assert!(
            reread.contains("tags: [python, rust]"),
            "Added tag should persist"
        );
    }
}
