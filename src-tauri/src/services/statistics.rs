//! Statistics calculation utilities for note analysis.

use crate::models::response_types::{
    AppStatistics, DailyStat, InsightResponse, TagStat, ThreadStat,
};
use chrono::{Datelike, NaiveDate};
use std::collections::HashMap;

/// Calculates writing streaks from a list of dates.
/// Returns (current_streak, best_streak)
pub fn calculate_streaks(note_dates: &[NaiveDate], today_date: NaiveDate) -> (usize, usize) {
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

    (current_streak, best_streak)
}

/// Generates insights based on statistics data.
pub fn generate_insights(
    current_streak: usize,
    best_streak: usize,
    weekday_distribution: &[usize],
    daily_stats: &[DailyStat],
    total_notes: usize,
    total_words: usize,
    tag_counts: &HashMap<String, usize>,
    tag_last_used: &HashMap<String, NaiveDate>,
    top_threads: &[ThreadStat],
    today_date: NaiveDate,
) -> Vec<InsightResponse> {
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

    insights
}

/// Collects statistics data from note files.
pub fn collect_note_statistics(
    note_manager: &crate::services::note_manager::NoteManager,
    all_files: &[String],
) -> (
    usize,
    usize,
    usize,
    usize,
    HashMap<String, usize>,
    HashMap<String, usize>,
    HashMap<String, NaiveDate>,
    HashMap<String, NaiveDate>,
    Vec<DailyStat>,
    Vec<NaiveDate>,
    Vec<usize>,
) {
    use std::collections::HashMap;
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

    for file_name in all_files {
        let path = note_manager.notes_folder.join(file_name);
        if let Ok(content) = std::fs::read_to_string(&path) {
            total_notes += 1;

            let char_count = content.chars().count();
            let (word_count, _) = crate::utils::markdown::count_words(&content);

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
                let threads = note_manager.extract_threads(&content, 100);
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

    (
        total_notes,
        total_threads,
        total_characters,
        total_words,
        tag_counts,
        thread_counts,
        tag_last_used,
        thread_last_used,
        daily_stats,
        note_dates,
        weekday_distribution,
    )
}

/// Processes tag and thread statistics from collected data.
pub fn process_tag_and_thread_stats(
    tag_counts: &HashMap<String, usize>,
    thread_counts: &HashMap<String, usize>,
) -> (Vec<TagStat>, Vec<ThreadStat>) {
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

    let _total_tags = top_tags.len();
    top_tags.truncate(10);
    top_threads.truncate(10);

    (top_tags, top_threads)
}

/// Gathers comprehensive statistics across all notes in the configured folder.
pub fn get_statistics(
    note_manager: &crate::services::note_manager::NoteManager,
) -> Result<AppStatistics, String> {
    let all_files = note_manager.get_sorted_note_files()?;
    let today = crate::utils::date::get_current_date();
    let today_date = NaiveDate::parse_from_str(&today, "%Y-%m-%d").unwrap();

    // Collect data from all notes
    let (
        total_notes,
        total_threads,
        total_characters,
        total_words,
        tag_counts,
        thread_counts,
        tag_last_used,
        _thread_last_used,
        mut daily_stats,
        mut note_dates,
        weekday_distribution,
    ) = collect_note_statistics(note_manager, &all_files);

    // Calculate streaks
    note_dates.sort_by(|a, b| b.cmp(a));
    let (current_streak, best_streak) = calculate_streaks(&note_dates, today_date);

    // Process tag and thread statistics
    let (top_tags, top_threads) = process_tag_and_thread_stats(&tag_counts, &thread_counts);
    let total_tags = top_tags.len();

    // Sort daily stats
    daily_stats.sort_by(|a, b| a.date.cmp(&b.date));

    // Generate insights
    let insights = generate_insights(
        current_streak,
        best_streak,
        &weekday_distribution,
        &daily_stats,
        total_notes,
        total_words,
        &tag_counts,
        &tag_last_used,
        &top_threads,
        today_date,
    );

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
