//! Utilities for date and time formatting.

use chrono::{DateTime, Days, Local, NaiveDate};
use std::sync::Mutex;
use std::time::SystemTime;

/// Per-day cache for the current local date.
///
/// `Local::now()` performs a timezone-database lookup on every call, and hot
/// commands call `get_current_date()` several times each. The cache is
/// validated against a cheap UTC clock (`SystemTime::now()`, no timezone
/// lookup) plus the cached UTC offset, so the real local timezone is only
/// consulted when the local day actually changes (or on the first call).
struct DateCache {
    /// The local date the cached value was computed for.
    local_date: NaiveDate,
    /// UTC offset in seconds that was in effect when the cache was filled.
    utc_offset_secs: i32,
    /// Cached `YYYY-MM-DD` string.
    value: String,
}

static CURRENT_DATE_CACHE: Mutex<Option<DateCache>> = Mutex::new(None);

/// Returns the current local date and its `YYYY-MM-DD` string, using the
/// per-day cache.
fn current_local_date() -> (NaiveDate, String) {
    let (secs, nanos) = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| (d.as_secs() as i64, d.subsec_nanos()))
        .unwrap_or((0, 0));

    let mut guard = CURRENT_DATE_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(cache) = guard.as_ref() {
        // Fast path: derive the local date from the UTC clock + cached offset.
        if let Some(utc) = DateTime::from_timestamp(secs, nanos) {
            let local = utc.naive_utc() + chrono::Duration::seconds(cache.utc_offset_secs as i64);
            if local.date() == cache.local_date {
                return (cache.local_date, cache.value.clone());
            }
        }
    }

    // Slow path (first call or local day changed): real timezone lookup.
    let local_now = Local::now();
    let date = local_now.date_naive();
    let value = local_now.format("%Y-%m-%d").to_string();
    *guard = Some(DateCache {
        local_date: date,
        utc_offset_secs: local_now.offset().local_minus_utc(),
        value: value.clone(),
    });
    (date, value)
}

/// Returns the current local date formatted as `YYYY-MM-DD` (cached per day).
pub fn get_current_date() -> String {
    current_local_date().1
}

/// Returns the date path (YYYY-MM-DD.md) for a date offset from today.
///
/// Positive offset goes forward in time, negative offset goes backward.
/// - offset = 0: today
/// - offset = -1: yesterday
/// - offset = -7: one week ago
pub fn get_date_path_by_offset(offset: i32) -> String {
    let today = current_local_date().0;
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
    fn test_get_current_date_is_stable_across_calls() {
        // Repeated calls (cache fast path) must return the same value.
        let first = get_current_date();
        let second = get_current_date();
        assert_eq!(first, second);
        assert_eq!(first, fmt(Local::now().date_naive()));
        // Offset path shares the same cache.
        assert_eq!(get_date_path_by_offset(0), first);
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
