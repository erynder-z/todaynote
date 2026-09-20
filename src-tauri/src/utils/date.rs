//! Utilities for date and time formatting.

use chrono::{Days, Local};

/// Returns the current local date formatted as `YYYY-MM-DD`.
pub fn get_current_date() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

/// Returns the date path (YYYY-MM-DD.md) for a date offset from today.
///
/// Positive offset goes forward in time, negative offset goes backward.
/// - offset = 0: today
/// - offset = -1: yesterday
/// - offset = -7: one week ago
pub fn get_date_path_by_offset(offset: i32) -> String {
    let today = Local::now().date_naive();
    // Handle negative offsets by using checked_sub_days
    let target_date = if offset >= 0 {
        today
            .checked_add_days(Days::new(offset as u64))
            .unwrap_or(today)
    } else {
        today
            .checked_sub_days(Days::new((-offset) as u64))
            .unwrap_or(today)
    };
    target_date.format("%Y-%m-%d").to_string()
}

/// Returns the full note path (notes_folder/YYYY-MM-DD.md) for a date offset from today.
pub fn get_note_path_by_offset(
    notes_folder: &std::path::PathBuf,
    offset: i32,
) -> std::path::PathBuf {
    let date_str = get_date_path_by_offset(offset);
    let file_name = format!("{}.md", date_str);
    notes_folder.join(file_name)
}

///
/// Unit Tests
///
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Local};
    use std::path::PathBuf;

    fn fmt(date: chrono::NaiveDate) -> String {
        format!("{:04}-{:02}-{:02}", date.year(), date.month(), date.day())
    }

    #[test]
    fn test_get_current_date_matches_today() {
        let today = Local::now().date_naive();
        assert_eq!(get_current_date(), fmt(today));
    }

    #[test]
    fn test_get_date_path_by_offset_zero_is_today() {
        let today = Local::now().date_naive();
        assert_eq!(get_date_path_by_offset(0), fmt(today));
    }

    #[test]
    fn test_get_date_path_by_offset_negative_and_positive() {
        let today = Local::now().date_naive();
        let yesterday = today.checked_sub_days(Days::new(1)).unwrap();
        let tomorrow = today.checked_add_days(Days::new(1)).unwrap();

        assert_eq!(get_date_path_by_offset(-1), fmt(yesterday));
        assert_eq!(get_date_path_by_offset(1), fmt(tomorrow));
    }

    #[test]
    fn test_get_note_path_by_offset_builds_correct_path() {
        let folder = PathBuf::from("/tmp/notes");
        let today = Local::now().date_naive();
        let expected = folder.join(format!("{}.md", fmt(today)));

        let path = get_note_path_by_offset(&folder, 0);
        assert_eq!(path, expected);
        assert_eq!(path.extension(), Some(std::ffi::OsStr::new("md")));
        assert_eq!(path.parent(), Some(folder.as_path()));
    }
}
