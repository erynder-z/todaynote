//! Utilities for application-specific data and directory paths.

use std::path::PathBuf;

/// Returns the user's home directory path.
/// Defaults to the current directory if it cannot be determined.
pub fn get_home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

/// Returns the absolute path to the application's local data directory.
///
/// This is where the configuration and potentially other persistent
/// app-related data is stored.
pub fn get_app_data_dir() -> PathBuf {
    get_home_dir().join(".todaynote")
}

///
/// Unit Tests
///
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_home_dir_returns_non_empty_path() {
        let home = get_home_dir();

        // In normal environments, home_dir() resolves to a real path.
        // Only the fallback case yields ".".
        if home != PathBuf::from(".") {
            assert!(
                home.is_absolute(),
                "Home dir should be absolute: {:?}",
                home
            );
        }
    }

    #[test]
    fn test_get_app_data_dir_ends_with_todaynote() {
        let app_data = get_app_data_dir();

        assert_eq!(
            app_data.file_name(),
            Some(std::ffi::OsStr::new(".todaynote")),
            "App data dir should end with '.todaynote'"
        );
    }

    #[test]
    fn test_get_app_data_dir_is_under_home_dir() {
        let home = get_home_dir();
        let app_data = get_app_data_dir();

        assert_eq!(
            app_data.parent(),
            Some(home.as_path()),
            "App data dir should be a direct child of the home dir"
        );
    }

    #[test]
    fn test_get_app_data_dir_is_consistent() {
        let first = get_app_data_dir();
        let second = get_app_data_dir();

        assert_eq!(first, second, "Repeated calls should return the same path");
    }
}
