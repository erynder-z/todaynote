//! Shared, mtime-validated index of a notes folder.
//!
//! [`FolderIndex`] maps each note filename to its parsed metadata (frontmatter
//! fields, tags, thread names, counts) so the read-only folder-wide
//! aggregations (tag/thread search, notes-by-tag, thread aggregation, pinned
//! threads, statistics, purge scans) do not each re-read and re-parse every
//! `.md` file on disk.
//!
//! Entries are validated against each file's `(mtime, len)` on every refresh,
//! so writes and deletes invalidate them lazily: unchanged entries are reused,
//! changed files are re-parsed, deleted files are dropped. The whole index is
//! dropped when the notes folder itself changes.

use crate::models::app_state::AppState;
use crate::models::note_header::{NoteHeader, ThreadRef};
use crate::services::note_manager::NoteManager;
use crate::utils::{markdown, tag_parser, text};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Parsed per-file metadata cached in the [`FolderIndex`].
#[derive(Debug, Clone)]
pub struct NoteMeta {
    /// The note's filename (the index key as well, for convenience).
    pub filename: String,
    /// File modification time when the entry was built.
    pub mtime: SystemTime,
    /// File length in bytes when the entry was built.
    pub len: u64,
    /// `created` frontmatter value as `YYYY-MM-DD`, if the header parses.
    pub created: Option<String>,
    /// Tags from the frontmatter.
    pub tags: Vec<String>,
    /// Thread references (`id`, `line`, `pinned`) from the frontmatter.
    pub thread_refs: Vec<ThreadRef>,
    /// Thread names from body `!!! ` markers (duplicates kept, like the search scan).
    pub thread_names: Vec<String>,
    /// Number of lines occupied by the frontmatter block (0 when none).
    pub body_start: usize,
    /// Word count of the body (see [`text::count_words`]).
    pub word_count: usize,
    /// Character count of the whole file.
    pub char_count: usize,
    /// Image filenames referenced from the note body (images dir only).
    pub image_files: Vec<String>,
    /// Whether the note has no content beyond frontmatter, headings, and thread markers.
    pub is_empty: bool,
    /// First non-empty body line (raw, trimmed), used for excerpts.
    pub first_body_line: String,
}

/// In-memory index of a notes folder: filename → parsed metadata.
#[derive(Debug, Clone)]
pub struct FolderIndex {
    /// The notes folder this index was built for.
    pub folder: PathBuf,
    /// Per-file metadata, keyed by filename.
    pub entries: HashMap<String, NoteMeta>,
}

impl FolderIndex {
    /// An empty index for `folder`.
    pub fn empty(folder: PathBuf) -> Self {
        Self {
            folder,
            entries: HashMap::new(),
        }
    }

    /// Builds a full index by reading every note file in `folder`.
    pub fn scan(folder: &Path) -> Result<Self, String> {
        let listing = list_note_files_meta(folder)?;
        let (index, _) = refresh_from(None, folder, listing)?;
        Ok(index)
    }
}

/// Lists all valid Markdown note files in `folder` with their `(mtime, len)`,
/// sorted by name descending (most recent first). Dotfiles and non-`.md`
/// files are skipped, consistent with `NoteManager::get_sorted_note_files`.
fn list_note_files_meta(folder: &Path) -> Result<Vec<(String, SystemTime, u64)>, String> {
    if !folder.exists() {
        return Ok(Vec::new());
    }
    let entries = fs::read_dir(folder).map_err(|e| format!("Failed to read directory: {}", e))?;
    let mut files: Vec<(String, SystemTime, u64)> = Vec::new();
    for entry in entries.filter_map(|e| e.ok()) {
        let name = match entry.file_name().into_string() {
            Ok(n) => n,
            Err(_) => continue,
        };
        if !name.ends_with(".md") || name.starts_with(".") {
            continue;
        }
        if let Ok(meta) = entry.metadata() {
            if let Ok(mtime) = meta.modified() {
                files.push((name, mtime, meta.len()));
            }
        }
    }
    files.sort_by(|a, b| b.0.cmp(&a.0));
    Ok(files)
}

/// Parses a single note file into a [`NoteMeta`] in one pass over its content.
/// Returns `None` when the file cannot be read.
fn parse_note_meta(
    folder: &Path,
    file_name: &str,
    mtime: SystemTime,
    len: u64,
) -> Option<NoteMeta> {
    let content = fs::read_to_string(folder.join(file_name)).ok()?;

    let (created, tags, thread_refs) = match NoteHeader::parse(&content) {
        Ok(header) => (
            Some(header.created.format("%Y-%m-%d").to_string()),
            header.tags,
            header.threads,
        ),
        Err(_) => (
            None,
            tag_parser::parse_tags_from_content(&content),
            parse_thread_refs_lenient(&content),
        ),
    };

    let body_start = markdown::frontmatter_line_count(&content);
    let thread_names = markdown::thread_names(&content);
    let (word_count, _has_code) = text::count_words(&content);
    let image_files: Vec<String> = markdown::extract_image_urls(&content)
        .into_iter()
        .filter_map(|url| crate::commands::images::image_filename_from_url(&url))
        .collect();
    let first_body_line = content
        .lines()
        .skip(body_start)
        .find(|l| !l.trim().is_empty())
        .map(|l| l.trim().to_string())
        .unwrap_or_default();

    Some(NoteMeta {
        filename: file_name.to_string(),
        mtime,
        len,
        created,
        tags,
        thread_refs,
        thread_names,
        body_start,
        word_count,
        char_count: content.chars().count(),
        image_files,
        is_empty: NoteManager::is_note_empty(&content),
        first_body_line,
    })
}

/// Extracts the `threads` value leniently (without requiring the whole
/// frontmatter to parse) so pinned metadata survives a malformed header.
fn parse_thread_refs_lenient(content: &str) -> Vec<ThreadRef> {
    let mut lines = content.lines();
    if lines.next().map(|l| l.trim()) != Some("---") {
        return Vec::new();
    }
    for line in lines {
        let trimmed = line.trim();
        if trimmed == "---" {
            break;
        }
        if let Some((key, value)) = trimmed.split_once(':') {
            if key.trim() == "threads" {
                return NoteHeader::parse_threads_value(value.trim());
            }
        }
    }
    Vec::new()
}

/// Parses the given files in parallel using scoped threads.
fn parse_files_parallel(folder: &Path, files: &[(String, SystemTime, u64)]) -> Vec<NoteMeta> {
    if files.is_empty() {
        return Vec::new();
    }
    let workers = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(files.len())
        .max(1);
    let chunk_size = files.len().div_ceil(workers);
    std::thread::scope(|scope| {
        let handles: Vec<_> = files
            .chunks(chunk_size)
            .map(|chunk| {
                scope.spawn(|| {
                    chunk
                        .iter()
                        .filter_map(|(name, mtime, len)| {
                            parse_note_meta(folder, name, *mtime, *len)
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().unwrap_or_default())
            .collect()
    })
}

/// Rebuilds an index from `listing`, reusing cached entries whose
/// `(mtime, len)` still match. Returns the new index and the number of
/// entries that changed (re-parsed or removed).
fn refresh_from(
    cached: Option<HashMap<String, NoteMeta>>,
    folder: &Path,
    listing: Vec<(String, SystemTime, u64)>,
) -> Result<(FolderIndex, usize), String> {
    let cached = cached.unwrap_or_default();
    let mut entries: HashMap<String, NoteMeta> = HashMap::with_capacity(listing.len());
    let mut stale: Vec<(String, SystemTime, u64)> = Vec::new();
    let mut present = 0usize;

    for (name, mtime, len) in listing {
        match cached.get(&name) {
            Some(meta) if meta.mtime == mtime && meta.len == len => {
                entries.insert(name, meta.clone());
                present += 1;
            }
            Some(_) => {
                stale.push((name, mtime, len));
                present += 1;
            }
            None => stale.push((name, mtime, len)),
        }
    }
    // Cached entries whose file no longer exists are dropped, not re-parsed.
    let removed = cached.len() - present;
    let parsed = parse_files_parallel(folder, &stale);
    let changed = parsed.len() + removed;
    for meta in parsed {
        entries.insert(meta.filename.clone(), meta);
    }

    Ok((
        FolderIndex {
            folder: folder.to_path_buf(),
            entries,
        },
        changed,
    ))
}

/// Returns an up-to-date snapshot of the index for `folder`, reusing the
/// cached index in `state` when its entries still match the on-disk
/// `(mtime, len)` values and re-parsing only new, changed, or removed files.
pub fn get_or_refresh(state: &AppState, folder: &Path) -> Result<FolderIndex, String> {
    let cached = {
        let guard = state.folder_index()?;
        if guard.folder == folder {
            Some(guard.entries.clone())
        } else {
            None
        }
    };
    let (index, changed) = match cached {
        // Warm cache: stat the folder and re-parse only new/changed files.
        Some(entries) => {
            let listing = list_note_files_meta(folder)?;
            refresh_from(Some(entries), folder, listing)?
        }
        // Cold cache (or folder switched): one full parallel scan.
        None => (FolderIndex::scan(folder)?, usize::MAX),
    };
    if changed > 0 {
        // Store the refreshed index (including its folder). A concurrent
        // refresh for a different folder may overwrite it; the next caller
        // for the overwritten folder simply rebuilds from scratch.
        let mut guard = state.folder_index()?;
        *guard = index.clone();
    }
    Ok(index)
}

///
/// Unit Tests
///
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::config::AppConfig;
    use crate::models::note_session::NoteSession;
    use crate::services::tag_manager::TagManager;
    use std::sync::Mutex;
    use tempfile::tempdir;

    fn write(dir: &tempfile::TempDir, name: &str, content: &str) {
        fs::write(dir.path().join(name), content).expect("Failed to write test note");
    }

    #[test]
    fn test_scan_nonexistent_folder_is_empty() {
        let index = FolderIndex::scan(Path::new("/nonexistent/path")).unwrap();
        assert!(index.entries.is_empty());
    }

    #[test]
    fn test_scan_empty_folder() {
        let dir = tempdir().unwrap();
        let index = FolderIndex::scan(dir.path()).unwrap();
        assert!(index.entries.is_empty());
    }

    #[test]
    fn test_scan_collects_metadata_and_skips_non_notes() {
        let dir = tempdir().unwrap();
        write(
            &dir,
            "2024-01-02.md",
            "---\ncreated: 2024-01-02\nnote-type: manual\ntags: [work, urgent]\nthreads: [{id: t1, line: 0, pinned: true}]\n---\n!!! Ideas\nSome **bold** text here\n![pic](images/pic.png#small-center)\n",
        );
        write(&dir, "2024-01-01.md", "plain body without frontmatter");
        write(&dir, ".hidden.md", "---\ntags: [secret]\n---\nhidden");
        write(&dir, "readme.txt", "---\ntags: [nope]\n---\nnot a note");

        let index = FolderIndex::scan(dir.path()).unwrap();
        assert_eq!(index.entries.len(), 2);

        let meta = &index.entries["2024-01-02.md"];
        assert_eq!(meta.created.as_deref(), Some("2024-01-02"));
        assert_eq!(meta.tags, vec!["work", "urgent"]);
        assert_eq!(meta.thread_names, vec!["Ideas"]);
        assert_eq!(meta.thread_refs.len(), 1);
        assert!(meta.thread_refs[0].pinned);
        assert_eq!(meta.thread_refs[0].id, "t1");
        assert_eq!(meta.body_start, 6);
        assert!(meta.word_count > 0);
        assert!(meta.char_count > 0);
        assert_eq!(meta.image_files, vec!["pic.png"]);
        assert!(!meta.is_empty);
        assert_eq!(meta.first_body_line, "!!! Ideas");

        // No frontmatter: header fields fall back, tags empty, body starts at 0.
        let plain = &index.entries["2024-01-01.md"];
        assert_eq!(plain.created, None);
        assert!(plain.tags.is_empty());
        assert_eq!(plain.body_start, 0);
        assert_eq!(plain.first_body_line, "plain body without frontmatter");
    }

    #[test]
    fn test_scan_marks_empty_notes() {
        let dir = tempdir().unwrap();
        write(
            &dir,
            "empty.md",
            "---\ncreated: 2024-01-01\ntags: []\n---\n# Only a heading\n",
        );
        write(
            &dir,
            "coded.md",
            "---\ncreated: 2024-01-01\ntags: [x]\n---\n```rust\nlet a = 1;\n```\n",
        );

        let index = FolderIndex::scan(dir.path()).unwrap();
        assert!(index.entries["empty.md"].is_empty);
        assert!(!index.entries["coded.md"].is_empty); // has a tag
    }

    #[test]
    fn test_refresh_reuses_unchanged_entries() {
        let dir = tempdir().unwrap();
        write(&dir, "a.md", "---\ntags: [one]\n---\nbody a");
        write(&dir, "b.md", "---\ntags: [two]\n---\nbody b");

        let index = FolderIndex::scan(dir.path()).unwrap();
        let listing = list_note_files_meta(dir.path()).unwrap();
        let (refreshed, changed) =
            refresh_from(Some(index.entries.clone()), dir.path(), listing).unwrap();
        assert_eq!(changed, 0, "Nothing changed on disk");
        assert_eq!(refreshed.entries.len(), 2);
        // Entries are cloned, not re-read; identity of values is preserved.
        assert_eq!(refreshed.entries["a.md"].tags, vec!["one"]);
    }

    #[test]
    fn test_refresh_reparses_modified_and_drops_deleted() {
        let dir = tempdir().unwrap();
        write(&dir, "a.md", "---\ntags: [old]\n---\nbody a");
        write(&dir, "b.md", "---\ntags: [keep]\n---\nbody b");

        let index = FolderIndex::scan(dir.path()).unwrap();

        // Modify a.md (ensure mtime actually changes) and delete b.md.
        std::thread::sleep(std::time::Duration::from_millis(1100));
        write(&dir, "a.md", "---\ntags: [new]\n---\nbody a changed");
        fs::remove_file(dir.path().join("b.md")).unwrap();

        let listing = list_note_files_meta(dir.path()).unwrap();
        let (refreshed, changed) =
            refresh_from(Some(index.entries.clone()), dir.path(), listing).unwrap();
        assert_eq!(changed, 2, "One modified + one removed");
        assert_eq!(refreshed.entries.len(), 1);
        assert_eq!(refreshed.entries["a.md"].tags, vec!["new"]);
        assert!(!refreshed.entries.contains_key("b.md"));
    }

    #[test]
    fn test_refresh_detects_same_second_write_via_len() {
        let dir = tempdir().unwrap();
        write(&dir, "a.md", "---\ntags: [x]\n---\n12345");
        let index = FolderIndex::scan(dir.path()).unwrap();

        // Same length is fine when mtime changes; here force a same-length
        // rewrite and rely on mtime. To make the test deterministic, write a
        // different-length file instead and check the entry updates.
        write(&dir, "a.md", "---\ntags: [y]\n---\n1234567890");
        let listing = list_note_files_meta(dir.path()).unwrap();
        let (refreshed, changed) =
            refresh_from(Some(index.entries.clone()), dir.path(), listing).unwrap();
        assert_eq!(changed, 1);
        assert_eq!(refreshed.entries["a.md"].tags, vec!["y"]);
    }

    #[test]
    fn test_get_or_refresh_stores_and_reuses_cache() {
        let dir = tempdir().unwrap();
        write(&dir, "a.md", "---\ntags: [cached]\n---\nbody");

        let state = AppState {
            note_manager: Mutex::new(NoteManager::new(dir.path().to_path_buf(), "en".into())),
            tag_manager: Mutex::new(TagManager::new()),
            note_session: Mutex::new(NoteSession::new()),
            config: Mutex::new(AppConfig::default()),
            folder_index: Mutex::new(FolderIndex::empty(dir.path().to_path_buf())),
        };

        let index = get_or_refresh(&state, dir.path()).unwrap();
        assert_eq!(index.entries.len(), 1);
        assert!(state.folder_index().unwrap().entries.contains_key("a.md"));

        // Second call: cache is warm and fresh, nothing re-parsed.
        let listing = list_note_files_meta(dir.path()).unwrap();
        let cached = Some(state.folder_index().unwrap().entries.clone());
        let (_, changed) = refresh_from(cached, dir.path(), listing).unwrap();
        assert_eq!(changed, 0);

        // Folder switch invalidates: index for another folder is rebuilt empty.
        let other = tempdir().unwrap();
        let index2 = get_or_refresh(&state, other.path()).unwrap();
        assert!(index2.entries.is_empty());
        assert_eq!(state.folder_index().unwrap().folder, other.path());
    }

    #[test]
    fn test_get_or_refresh_picks_up_external_write() {
        let dir = tempdir().unwrap();
        write(&dir, "a.md", "---\ntags: [before]\n---\nbody");

        let state = AppState {
            note_manager: Mutex::new(NoteManager::new(dir.path().to_path_buf(), "en".into())),
            tag_manager: Mutex::new(TagManager::new()),
            note_session: Mutex::new(NoteSession::new()),
            config: Mutex::new(AppConfig::default()),
            folder_index: Mutex::new(FolderIndex::empty(dir.path().to_path_buf())),
        };

        let index = get_or_refresh(&state, dir.path()).unwrap();
        assert_eq!(index.entries["a.md"].tags, vec!["before"]);

        std::thread::sleep(std::time::Duration::from_millis(1100));
        write(&dir, "a.md", "---\ntags: [after]\n---\nbody changed");
        write(&dir, "b.md", "---\ntags: [extra]\n---\nnew note");

        let index = get_or_refresh(&state, dir.path()).unwrap();
        assert_eq!(index.entries["a.md"].tags, vec!["after"]);
        assert!(index.entries.contains_key("b.md"));
    }

    #[test]
    fn test_lenient_thread_refs_survive_malformed_header() {
        let content = "---\nthreads: [{id: t9, line: 2, pinned: true}]\nnot a pair\n---\n!!! T\nx";
        let refs = parse_thread_refs_lenient(content);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].id, "t9");
        assert!(refs[0].pinned);
    }
}
