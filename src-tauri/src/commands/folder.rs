//! Tauri commands for folder operations and validation.

use crate::models::response_types::FolderValidation;
use std::fs;
use std::path::PathBuf;

/// Validates a folder path for use as the notes storage directory.
///
/// This command checks if the path exists, is a directory, is writable,
/// and counts the number of existing Markdown files.
#[tauri::command]
pub async fn validate_folder(path: String) -> Result<FolderValidation, String> {
    let path_buf = PathBuf::from(&path);
    let mut validation = FolderValidation {
        is_valid: true,
        is_writable: false,
        exists: path_buf.exists(),
        note_count: 0,
        error: None,
    };

    if validation.exists {
        if !path_buf.is_dir() {
            validation.is_valid = false;
            validation.error = Some("Path is not a directory".to_string());
            return Ok(validation);
        }

        let temp_file = path_buf.join(".todaynote_write_test");
        match fs::write(&temp_file, "test") {
            Ok(_) => {
                validation.is_writable = true;
                let _ = fs::remove_file(temp_file);
            }
            Err(e) => {
                validation.is_writable = false;
                validation.error = Some(format!("Directory is not writable: {}", e));
            }
        }

        if let Ok(entries) = fs::read_dir(&path_buf) {
            validation.note_count = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
                .count();
        }
    } else {
        match fs::create_dir_all(&path_buf) {
            Ok(_) => {
                validation.is_writable = true;

                let _ = fs::remove_dir(&path_buf);
            }
            Err(e) => {
                validation.is_writable = false;
                validation.is_valid = false;
                validation.error = Some(format!("Cannot create directory at this path: {}", e));
            }
        }
    }

    Ok(validation)
}

///
/// Unit Tests
///
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    // Helper to run async tests
    async fn run_validate_folder(path: String) -> Result<FolderValidation, String> {
        validate_folder(path).await
    }

    #[tokio::test]
    async fn test_validate_existing_writable_folder() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let path = temp_dir.path().to_string_lossy().into_owned();

        let result = run_validate_folder(path)
            .await
            .expect("Validation should succeed");

        assert!(result.is_valid, "Folder should be valid");
        assert!(result.exists, "Folder should exist");
        assert!(result.is_writable, "Folder should be writable");
        assert!(result.error.is_none(), "Should have no error");
    }

    #[tokio::test]
    async fn test_validate_file_instead_of_folder() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test_file.txt");
        fs::write(&file_path, "test content").expect("Failed to create test file");
        let path = file_path.to_string_lossy().into_owned();

        let result = run_validate_folder(path)
            .await
            .expect("Validation should succeed");

        assert!(
            !result.is_valid,
            "File path should not be valid (not a directory)"
        );
        assert!(result.exists, "Path should exist");
        assert!(
            !result.is_writable,
            "File should not be marked as writable directory"
        );
        assert!(result.error.is_some(), "Should have an error");
        assert_eq!(result.error.unwrap(), "Path is not a directory");
    }

    #[tokio::test]
    async fn test_validate_nonexistent_creatable_folder() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let path = temp_dir
            .path()
            .join("new_folder")
            .to_string_lossy()
            .into_owned();

        // Ensure the folder doesn't exist yet
        assert!(
            !std::path::Path::new(&path).exists(),
            "Test folder should not exist yet"
        );

        let result = run_validate_folder(path.clone())
            .await
            .expect("Validation should succeed");

        assert!(result.is_valid, "Folder should be valid (can be created)");
        assert!(!result.exists, "Folder should not exist yet");
        assert!(
            result.is_writable,
            "Folder should be writable (since we can create it)"
        );
        assert!(result.error.is_none(), "Should have no error");

        let _ = fs::remove_dir(&path);
    }

    #[tokio::test]
    async fn test_validate_folder_with_markdown_files() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let path = temp_dir.path().to_string_lossy().into_owned();

        // Create some markdown files
        fs::write(temp_dir.path().join("2024-01-01.md"), "# Note 1")
            .expect("Failed to create md file 1");
        fs::write(temp_dir.path().join("2024-01-02.md"), "# Note 2")
            .expect("Failed to create md file 2");
        // Create a non-markdown file
        fs::write(temp_dir.path().join("readme.txt"), "readme").expect("Failed to create txt file");

        let result = run_validate_folder(path)
            .await
            .expect("Validation should succeed");

        assert!(result.is_valid, "Folder should be valid");
        assert_eq!(result.note_count, 2, "Should count 2 markdown files");
    }

    #[tokio::test]
    async fn test_validate_empty_folder() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let path = temp_dir.path().to_string_lossy().into_owned();

        let result = run_validate_folder(path)
            .await
            .expect("Validation should succeed");

        assert!(result.is_valid, "Empty folder should be valid");
        assert_eq!(result.note_count, 0, "Should have 0 markdown files");
    }

    #[test]
    fn test_folder_validation_serialization() {
        let validation = FolderValidation {
            is_valid: true,
            is_writable: true,
            exists: true,
            note_count: 5,
            error: None,
        };

        let json = serde_json::to_string(&validation).expect("Failed to serialize");
        let deserialized: FolderValidation =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.is_valid, validation.is_valid);
        assert_eq!(deserialized.is_writable, validation.is_writable);
        assert_eq!(deserialized.exists, validation.exists);
        assert_eq!(deserialized.note_count, validation.note_count);
        assert_eq!(deserialized.error, validation.error);
    }

    #[test]
    fn test_folder_validation_with_error() {
        let validation = FolderValidation {
            is_valid: false,
            is_writable: false,
            exists: false,
            note_count: 0,
            error: Some("Cannot create directory at this path: Permission denied".to_string()),
        };

        let json = serde_json::to_string(&validation).expect("Failed to serialize");
        let deserialized: FolderValidation =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(
            deserialized.error,
            Some("Cannot create directory at this path: Permission denied".to_string())
        );
    }
}
