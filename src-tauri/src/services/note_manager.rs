//! Manager to handle note operations and formatting.

use crate::models::response_types::{
    AppStatistics, DailyStat, FormattedNote, InsightResponse, NoteListResponse, TagStat, ThreadStat,
};
use crate::utils;
use chrono::{Datelike, Locale, NaiveDate};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Manager responsible for note-related operations like creation, listing, and formatting.
pub struct NoteManager {
    /// The root directory where notes are stored.
    pub notes_folder: PathBuf,
    /// The currently active locale for date formatting.
    pub locale: String,
}

impl NoteManager {
    /// Creates a new `NoteManager` with the specified configuration.
    pub fn new(notes_folder: PathBuf, locale: String) -> Self {
        Self {
            notes_folder,
            locale,
        }
    }

    /// Updates the manager's configuration when settings change.
    pub fn update_config(&mut self, notes_folder: PathBuf, locale: String) {
        self.notes_folder = notes_folder;
        self.locale = locale;
    }

    /// Ensures that the configured notes folder exists, creating it if necessary.
    pub fn ensure_notes_folder_exists(&self) -> Result<(), String> {
        if !self.notes_folder.exists() {
            fs::create_dir_all(&self.notes_folder)
                .map_err(|e| format!("Failed to create notes folder: {}", e))?;
        }
        Ok(())
    }

    /// Returns the absolute path to today's daily note file.
    pub fn get_today_note_path(&self) -> PathBuf {
        let current_date = utils::date::get_current_date();
        let file_name = format!("{}.md", current_date);
        self.notes_folder.join(&file_name)
    }

    /// Checks if today's daily note file already exists.
    pub fn todays_note_exists(&self) -> bool {
        self.get_today_note_path().exists()
    }

    /// Creates today's daily note if it doesn't already exist.
    ///
    /// The note is initialized with YAML metadata and a default thread header.
    pub fn create_todays_note(&self, note_header: &str) -> Result<PathBuf, String> {
        self.ensure_notes_folder_exists()?;
        let file_path = self.get_today_note_path();

        if self.todays_note_exists() {
            return Ok(file_path);
        }

        let current_date = utils::date::get_current_date();
        let note_content = format!(
            "---\ncreated: {}\ntags: []\n---\n# {}\n",
            current_date, note_header
        );

        fs::write(&file_path, note_content).map_err(|e| format!("Failed to create note: {}", e))?;

        Ok(file_path)
    }

    /// Retrieves all valid Markdown note filenames from the notes folder,
    /// sorted by name descending (most recent first).
    fn get_sorted_note_files(&self) -> Result<Vec<String>, String> {
        if !self.notes_folder.exists() {
            return Ok(vec![]);
        }
        let entries = fs::read_dir(&self.notes_folder)
            .map_err(|e| format!("Failed to read directory: {}", e))?;
        let mut files: Vec<String> = entries
            .filter_map(|e| {
                let e = e.ok()?;
                let name = e.file_name().into_string().ok()?;
                if name.ends_with(".md") && !name.starts_with(".") {
                    Some(name)
                } else {
                    None
                }
            })
            .collect();
        files.sort_by(|a, b| b.cmp(a));
        Ok(files)
    }

    /// Transforms a note filename into a FormattedNote by reading and processing its content.
    fn format_note_file(&self, file_name: &str) -> Option<FormattedNote> {
        let path = self.notes_folder.join(file_name);
        let content = fs::read_to_string(&path).unwrap_or_default();
        Some(FormattedNote {
            filename: file_name.to_string(),
            formatted_name: self.format_note_name(file_name),
            preview: self.extract_preview(&content),
            tags: crate::utils::tag_parser::parse_tags_from_content(&content),
            threads: self.extract_threads(&content, 5),
            word_count: crate::utils::markdown::count_words(&content),
        })
    }

    /// Lists all valid Markdown notes in the configured notes folder.
    ///
    /// Notes are returned as `FormattedNote` objects with localized display names.
    /// If a limit is provided, only the most recent N notes are fully processed.
    pub fn list_notes(&self, limit: Option<usize>) -> Result<NoteListResponse, String> {
        let all_files = self.get_sorted_note_files()?;
        let total_count = all_files.len();
        let files_to_process: Vec<String> = match limit {
            Some(l) => all_files[..l.min(all_files.len())].to_vec(),
            None => all_files,
        };
        let notes = files_to_process
            .iter()
            .filter_map(|f| self.format_note_file(f))
            .collect();
        Ok(NoteListResponse { notes, total_count })
    }

    /// Extracts the first N thread names (headings) from the content.
    fn extract_threads(&self, content: &str, limit: usize) -> Vec<String> {
        content
            .lines()
            .filter(|l| l.starts_with("# "))
            .take(limit)
            .map(|l| l[2..].trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Extracts a short preview from the note content, skipping frontmatter and headings.
    fn extract_preview(&self, content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut start_idx = 0;

        // Skip frontmatter
        if lines.first().map(|l| l.trim()) == Some("---") {
            for (i, line) in lines.iter().enumerate().skip(1) {
                if line.trim() == "---" {
                    start_idx = i + 1;
                    break;
                }
            }
        }

        let mut preview_text = Vec::new();
        for line in lines.iter().skip(start_idx) {
            let stripped = crate::utils::markdown::strip_markdown(line);
            // Skip headings (already handled by strip_markdown but we also skip empty results)
            if stripped.is_empty() {
                continue;
            }
            preview_text.push(stripped);
            if preview_text.len() > 3 {
                break;
            }
        }

        let joined = preview_text.join(" ");
        let (preview, _) = crate::utils::markdown::generate_excerpt(&joined, &[], 150);
        preview
    }

    /// Formats a note's filename into a human-readable, localized string.
    ///
    /// If the filename follows the `YYYY-MM-DD.md` pattern, it is transformed
    /// into a localized date string.
    pub fn format_note_name(&self, note_name: &str) -> String {
        let without_ext = note_name.replace(".md", "");

        if let Ok(date) = NaiveDate::parse_from_str(&without_ext, "%Y-%m-%d") {
            let locale = match self.locale.as_str() {
                "de" => Locale::de_DE,
                "ja" => Locale::ja_JP,
                _ => Locale::en_US,
            };

            match self.locale.as_str() {
                "de" => format!("{}", date.format_localized("%A, %e. %B %Y", locale)),
                "ja" => format!("{}", date.format_localized("%Y年%m月%d日 (%A)", locale)),
                _ => format!("{}", date.format_localized("%A, %B %e, %Y", locale)),
            }
        } else {
            without_ext
        }
    }

    /// Reads the content of a note file from the specified path.
    pub fn read_note_content(&self, path: &PathBuf) -> Result<String, String> {
        fs::read_to_string(path).map_err(|e| format!("Failed to read note content: {}", e))
    }

    /// Deletes all notes that have no content (only frontmatter and headings).
    pub fn purge_empty_notes(&self) -> Result<usize, String> {
        let all_files = self.get_sorted_note_files()?;
        let mut purged_count = 0;

        for file_name in all_files {
            let path = self.notes_folder.join(&file_name);
            if let Ok(content) = fs::read_to_string(&path) {
                if self.is_note_empty(&content) {
                    fs::remove_file(&path)
                        .map_err(|e| format!("Failed to delete empty note {}: {}", file_name, e))?;
                    purged_count += 1;
                }
            }
        }
        Ok(purged_count)
    }

    /// Determines if a note is considered "empty" (no tags and no content beyond headings).
    fn is_note_empty(&self, content: &str) -> bool {
        // If it has tags, it's not empty
        let tags = crate::utils::tag_parser::parse_tags_from_content(content);
        if !tags.is_empty() {
            return false;
        }

        let lines: Vec<&str> = content.lines().collect();
        let mut start_idx = 0;

        // Skip frontmatter
        if lines.first().map(|l| l.trim()) == Some("---") {
            for (i, line) in lines.iter().enumerate().skip(1) {
                if line.trim() == "---" {
                    start_idx = i + 1;
                    break;
                }
            }
        }

        // Check if there is any content other than headings and whitespace
        for line in lines.iter().skip(start_idx) {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("# ") {
                continue;
            }
            return false;
        }

        true
    }

    /// Gathers comprehensive statistics across all notes in the configured folder.
    pub fn get_statistics(&self) -> Result<AppStatistics, String> {
        let all_files = self.get_sorted_note_files()?;

        let mut total_notes = 0;
        let mut total_threads = 0;
        let mut total_characters = 0;
        let mut total_words = 0;
        let mut tag_counts: HashMap<String, usize> = HashMap::new();
        let mut thread_counts: HashMap<String, usize> = HashMap::new();
        let mut tag_last_used: HashMap<String, NaiveDate> = HashMap::new();
        let mut thread_last_used: HashMap<String, NaiveDate> = HashMap::new();
        let mut daily_stats = Vec::new();
        let mut note_dates: Vec<NaiveDate> = Vec::new();
        let mut weekday_distribution = vec![0; 7];

        let today = utils::date::get_current_date();
        let today_date = NaiveDate::parse_from_str(&today, "%Y-%m-%d").unwrap();

        for file_name in all_files {
            let path = self.notes_folder.join(&file_name);
            if let Ok(content) = fs::read_to_string(&path) {
                total_notes += 1;

                let char_count = content.chars().count();
                let word_count = crate::utils::markdown::count_words(&content);

                total_characters += char_count;
                total_words += word_count;

                // Extract date for streaks and distribution
                let date_str = file_name.replace(".md", "");
                if let Ok(date) = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
                    note_dates.push(date);
                    // chrono weekday 0=Mon, 6=Sun
                    let day_idx = date.weekday().num_days_from_monday() as usize;
                    weekday_distribution[day_idx] += word_count;

                    // Count threads
                    let threads = self.extract_threads(&content, 100);
                    total_threads += threads.len();
                    for thread in threads {
                        *thread_counts.entry(thread.clone()).or_insert(0) += 1;
                        let last = thread_last_used.entry(thread).or_insert(date);
                        if date > *last {
                            *last = date;
                        }
                    }

                    // Count tags
                    let tags = crate::utils::tag_parser::parse_tags_from_content(&content);
                    for tag in tags {
                        *tag_counts.entry(tag.clone()).or_insert(0) += 1;
                        let last = tag_last_used.entry(tag).or_insert(date);
                        if date > *last {
                            *last = date;
                        }
                    }
                }

                daily_stats.push(DailyStat {
                    date: date_str,
                    character_count: char_count,
                    word_count,
                });
            }
        }

        // Calculate streaks (note_dates is sorted desc)
        note_dates.sort_by(|a, b| b.cmp(a));
        let mut current_streak = 0;
        let mut best_streak = 0;

        if !note_dates.is_empty() {
            // Check if current streak is still active
            let mut last_date = note_dates[0];
            let diff_to_today = (today_date - last_date).num_days();

            if diff_to_today <= 1 {
                let mut temp_current = 1;
                for i in 1..note_dates.len() {
                    let diff = (last_date - note_dates[i]).num_days();
                    if diff == 1 {
                        temp_current += 1;
                        last_date = note_dates[i];
                    } else if diff > 1 {
                        break;
                    }
                }
                current_streak = temp_current;
            }

            // Calculate best streak
            let mut temp_best = 1;
            last_date = note_dates[0];
            best_streak = 1;
            for i in 1..note_dates.len() {
                let diff = (last_date - note_dates[i]).num_days();
                if diff == 1 {
                    temp_best += 1;
                } else if diff > 1 {
                    if temp_best > best_streak {
                        best_streak = temp_best;
                    }
                    temp_best = 1;
                }
                last_date = note_dates[i];
            }
            if temp_best > best_streak {
                best_streak = temp_best;
            }
        }

        let mut top_tags: Vec<TagStat> = tag_counts
            .iter()
            .map(|(name, count)| TagStat {
                name: name.clone(),
                count: *count,
            })
            .collect();

        let mut top_threads: Vec<ThreadStat> = thread_counts
            .iter()
            .map(|(name, count)| ThreadStat {
                name: name.clone(),
                count: *count,
            })
            .collect();

        top_tags.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));
        top_threads.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));

        let total_tags = top_tags.len();
        top_tags.truncate(10);
        top_threads.truncate(10);

        daily_stats.sort_by(|a, b| a.date.cmp(&b.date));

        // Generate procedural insights
        let mut insights = Vec::new();

        // 1. Streak insight
        if current_streak > 1 {
            let mut params = HashMap::new();
            params.insert("count".to_string(), current_streak.to_string());
            insights.push(InsightResponse {
                key: "statistics.insight.streak.current".to_string(),
                params,
            });
        } else if best_streak > 3 {
            let mut params = HashMap::new();
            params.insert("count".to_string(), best_streak.to_string());
            insights.push(InsightResponse {
                key: "statistics.insight.streak.best".to_string(),
                params,
            });
        }

        // 2. Weekday insight
        let mut max_day_idx = 0;
        let mut max_day_val = 0;
        for (i, &val) in weekday_distribution.iter().enumerate() {
            if val > max_day_val {
                max_day_val = val;
                max_day_idx = i;
            }
        }
        if max_day_val > 0 {
            let mut params = HashMap::new();
            params.insert("day_index".to_string(), max_day_idx.to_string());
            insights.push(InsightResponse {
                key: "statistics.insight.weekday.max".to_string(),
                params,
            });
        }

        // 3. Trend Analysis (Last 7 days vs previous 7 days)
        let last_7_days: Vec<&DailyStat> = daily_stats.iter().rev().take(7).collect();
        let prev_7_days: Vec<&DailyStat> = daily_stats.iter().rev().skip(7).take(7).collect();

        if last_7_days.len() >= 3 && prev_7_days.len() >= 3 {
            let last_avg = last_7_days.iter().map(|s| s.word_count).sum::<usize>() as f32
                / last_7_days.len() as f32;
            let prev_avg = prev_7_days.iter().map(|s| s.word_count).sum::<usize>() as f32
                / prev_7_days.len() as f32;

            if last_avg > prev_avg * 1.1 {
                let diff = ((last_avg / prev_avg - 1.0) * 100.0) as i32;
                let mut params = HashMap::new();
                params.insert("percent".to_string(), diff.to_string());
                insights.push(InsightResponse {
                    key: "statistics.insight.trend.longer".to_string(),
                    params,
                });
            } else if last_avg < prev_avg * 0.9 {
                insights.push(InsightResponse {
                    key: "statistics.insight.trend.shorter".to_string(),
                    params: HashMap::new(),
                });
            }
        }

        // 4. "Languishing" Tags (Frequent tags not used recently)
        for (tag, &count) in tag_counts.iter() {
            if count >= 3 {
                if let Some(&last_date) = tag_last_used.get(tag) {
                    let days_since = (today_date - last_date).num_days();
                    if days_since >= 7 && days_since <= 21 {
                        let mut params = HashMap::new();
                        params.insert("tag".to_string(), tag.clone());
                        params.insert("days".to_string(), days_since.to_string());
                        insights.push(InsightResponse {
                            key: "statistics.insight.languishing.tag".to_string(),
                            params,
                        });
                        break;
                    }
                }
            }
        }

        // 5. Thread consistency
        if let Some(thread) = top_threads.first() {
            let percentage = (thread.count as f32 / total_notes as f32 * 100.0) as i32;
            if percentage > 50 {
                let mut params = HashMap::new();
                params.insert("name".to_string(), thread.name.clone());
                params.insert("percent".to_string(), percentage.to_string());
                insights.push(InsightResponse {
                    key: "statistics.insight.consistency.thread".to_string(),
                    params,
                });
            }
        }

        // 6. Word count milestone
        if total_words > 5000 {
            let mut params = HashMap::new();
            params.insert(
                "count".to_string(),
                ((total_words / 1000) * 1000).to_string(),
            );
            insights.push(InsightResponse {
                key: "statistics.insight.milestone.words".to_string(),
                params,
            });
        }

        Ok(AppStatistics {
            total_notes,
            total_tags,
            total_threads,
            total_characters,
            total_words,
            current_streak,
            best_streak,
            top_tags,
            top_threads,
            daily_stats,
            weekday_distribution,
            insights,
        })
    }
}
