use crate::models::response_types::FolderValidation;
use std::fs;
use std::path::PathBuf;

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
