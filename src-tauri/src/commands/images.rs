//! Tauri commands for managing images embedded in notes.
//!
//! Images are stored in an `images` directory inside the notes folder and
//! referenced from note markdown by a relative path (`images/<uuid>.<ext>`),
//! so notes remain portable outside the application.

use crate::models::app_state::AppState;
use crate::models::response_types::ImagePurgeReport;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;
use uuid::Uuid;

/// Maximum accepted size for downloaded remote images (25 MB).
const MAX_DOWNLOAD_BYTES: usize = 25 * 1024 * 1024;

/// Image extensions allowed in the images directory.
///
/// SVG is deliberately excluded because it can contain scripts that would
/// execute inside the webview.
const ALLOWED_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "bmp", "avif"];

/// Returns the absolute path of the images directory, creating it if needed.
pub fn ensure_images_dir(state: &AppState) -> Result<PathBuf, String> {
    let note_manager = state.note_manager()?;
    let dir = note_manager.notes_folder.join("images");
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create images directory: {}", e))?;
    Ok(dir)
}

/// Normalizes a file extension and validates it against the allow-list.
pub(crate) fn normalize_extension(ext: &str) -> Result<String, String> {
    let ext = ext.trim().trim_start_matches('.').to_lowercase();
    if ALLOWED_EXTENSIONS.contains(&ext.as_str()) {
        Ok(ext)
    } else {
        Err(format!("Unsupported image type: .{}", ext))
    }
}

/// Derives an image file extension from a MIME content type.
fn extension_from_content_type(content_type: &str) -> Option<&'static str> {
    match content_type.split(';').next().unwrap_or("").trim() {
        "image/png" => Some("png"),
        "image/jpeg" => Some("jpg"),
        "image/gif" => Some("gif"),
        "image/webp" => Some("webp"),
        "image/bmp" => Some("bmp"),
        "image/avif" => Some("avif"),
        _ => None,
    }
}

/// Copies a locally selected image file into the notes images directory.
///
/// Returns the stored path relative to the notes folder (e.g. `images/<uuid>.png`).
#[tauri::command]
pub async fn add_image_from_path(
    path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let source = Path::new(&path);
    if !source.is_file() {
        return Err("Selected file does not exist".to_string());
    }

    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| "Selected file has no file extension".to_string())?;
    let ext = normalize_extension(ext)?;

    let images_dir = ensure_images_dir(&state)?;
    let filename = format!("{}.{}", Uuid::new_v4(), ext);
    let dest = images_dir.join(&filename);

    fs::copy(source, &dest).map_err(|e| format!("Failed to copy image: {}", e))?;

    Ok(format!("images/{}", filename))
}

/// Downloads a remote image into the notes images directory.
///
/// Returns the stored path relative to the notes folder (e.g. `images/<uuid>.png`).
#[tauri::command]
pub async fn add_image_from_url(url: String, state: State<'_, AppState>) -> Result<String, String> {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("Only http:// and https:// URLs are supported".to_string());
    }

    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to download image: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to download image: HTTP {}",
            response.status()
        ));
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    // Prefer the extension from the (post-redirect) URL path, then the
    // content type. This keeps spoofed extensions from slipping through.
    let url_path = response.url().path().to_lowercase();
    let ext_from_url = url_path
        .rsplit('/')
        .next()
        .and_then(|name| name.rsplit_once('.'))
        .map(|(_, ext)| normalize_extension(ext));

    let ext = match ext_from_url {
        Some(Ok(ext)) => ext,
        _ => extension_from_content_type(&content_type)
            .ok_or_else(|| "The URL does not point to an image".to_string())?
            .to_string(),
    };

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to download image: {}", e))?;

    if bytes.len() > MAX_DOWNLOAD_BYTES {
        return Err("Image is too large (maximum 25 MB)".to_string());
    }

    let images_dir = ensure_images_dir(&state)?;
    let filename = format!("{}.{}", Uuid::new_v4(), ext);
    let dest = images_dir.join(&filename);

    fs::write(&dest, &bytes).map_err(|e| format!("Failed to save image: {}", e))?;

    Ok(format!("images/{}", filename))
}

/// Returns the absolute path of the notes images directory.
#[tauri::command]
pub fn get_images_dir(state: State<'_, AppState>) -> Result<String, String> {
    let note_manager = state.note_manager()?;
    let dir = note_manager.notes_folder.join("images");
    Ok(dir.to_string_lossy().to_string())
}

/// Deletes all images in the images directory that are not referenced by any
/// note.
#[tauri::command]
pub fn purge_unused_images(state: State<'_, AppState>) -> Result<ImagePurgeReport, String> {
    let (notes_folder, note_files) = {
        let note_manager = state.note_manager()?;
        (
            note_manager.notes_folder.clone(),
            note_manager.get_sorted_note_files()?,
        )
    };

    // Collect every image filename referenced from note markdown.
    let mut referenced: HashSet<String> = HashSet::new();
    for file_name in &note_files {
        let content = fs::read_to_string(notes_folder.join(file_name)).unwrap_or_default();
        for url in crate::utils::markdown::extract_image_urls(&content) {
            if let Some(filename) = image_filename_from_url(&url) {
                referenced.insert(filename);
            }
        }
    }

    let images_dir = notes_folder.join("images");
    let mut report = ImagePurgeReport {
        removed: 0,
        freed_bytes: 0,
    };

    if images_dir.is_dir() {
        let entries = fs::read_dir(&images_dir)
            .map_err(|e| format!("Failed to read images directory: {}", e))?;
        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            let is_image = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| ALLOWED_EXTENSIONS.contains(&e.to_lowercase().as_str()))
                .unwrap_or(false);
            if !is_image || referenced.contains(&name) {
                continue;
            }
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            if fs::remove_file(&path).is_ok() {
                report.removed += 1;
                report.freed_bytes += size;
            }
        }
    }

    Ok(report)
}

/// Returns the image filename referenced by a note markdown URL, if the URL
/// points into the images directory. Strips the `#<size>-<align>` fragment
/// used for image display options.
pub(crate) fn image_filename_from_url(url: &str) -> Option<String> {
    let url = url.trim().split('#').next().unwrap_or("").trim();
    let path = url.strip_prefix("images/")?;
    let name = path.rsplit('/').next()?;
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

///
/// Unit Tests
///
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_extension_accepts_known_types() {
        assert_eq!(normalize_extension(".PNG").unwrap(), "png");
        assert_eq!(normalize_extension("Jpg").unwrap(), "jpg");
        assert_eq!(normalize_extension("webp").unwrap(), "webp");
    }

    #[test]
    fn test_normalize_extension_rejects_unknown_and_svg() {
        assert!(normalize_extension("txt").is_err());
        assert!(normalize_extension("svg").is_err());
        assert!(normalize_extension("").is_err());
    }

    #[test]
    fn test_extension_from_content_type() {
        assert_eq!(extension_from_content_type("image/png"), Some("png"));
        assert_eq!(
            extension_from_content_type("image/jpeg; charset=utf-8"),
            Some("jpg")
        );
        assert_eq!(extension_from_content_type("text/html"), None);
        assert_eq!(extension_from_content_type(""), None);
    }

    #[test]
    fn test_image_filename_from_url() {
        assert_eq!(
            image_filename_from_url("images/abc.png"),
            Some("abc.png".to_string())
        );
        assert_eq!(
            image_filename_from_url("images/abc.png#small-center"),
            Some("abc.png".to_string())
        );
        assert_eq!(image_filename_from_url("https://x.io/pic.png"), None);
        assert_eq!(image_filename_from_url("images/"), None);
        assert_eq!(image_filename_from_url("notes/abc.png"), None);
    }
}
