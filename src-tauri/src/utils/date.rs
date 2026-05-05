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
