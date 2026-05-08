//! Utilities for processing Markdown text.

/// Strips common markdown characters from a line for cleaner display.
pub fn strip_markdown(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // 1. Strip leading markers
    let mut s = trimmed;

    // Headings (e.g., ### Title)
    if s.starts_with('#') {
        s = s.trim_start_matches('#').trim_start();
    }

    // Blockquotes (e.g., > Quote)
    if s.starts_with("> ") {
        s = &s[2..];
    }

    // Tasks and Lists (e.g., - [ ] Task, * Item)
    if s.starts_with("- [ ] ") || s.starts_with("- [x] ") {
        s = &s[6..];
    } else if s.starts_with("- ") || s.starts_with("* ") || s.starts_with("+ ") {
        s = &s[2..];
    }

    // 2. Strip inline markers (basic)
    // We remove longer markers first to avoid leaving orphans
    let mut result = s
        .replace("***", "")
        .replace("___", "")
        .replace("**", "")
        .replace("__", "")
        .replace("~~", "")
        .replace("`", "")
        .replace("*", "")
        .replace("_", "");

    // 3. Strip trailing line-break backslashes
    if result.ends_with('\\') {
        result.pop();
    }

    result.trim().to_string()
}

/// Generates an excerpt from a text string, centered around the matching indices if provided.
/// Returns (excerpt_string, adjusted_indices).
pub fn generate_excerpt(line: &str, indices: &[u32], max_length: usize) -> (String, Vec<u32>) {
    let chars: Vec<char> = line.chars().collect();
    if chars.len() <= max_length {
        return (line.to_string(), indices.to_vec());
    }

    let first = *indices.first().unwrap_or(&0) as usize;
    let last = *indices.last().unwrap_or(&(chars.len() as u32 - 1)) as usize;
    let match_len = last - first + 1;

    let start = if match_len >= max_length {
        first
    } else {
        let surplus = max_length - match_len;
        first.saturating_sub(surplus / 2)
    };
    let end = (start + max_length).min(chars.len());
    let start = end.saturating_sub(max_length);

    let excerpt: String = chars[start..end].iter().collect();
    let mut adjusted_indices = Vec::new();
    for &idx in indices {
        let idx_usize = idx as usize;
        if idx_usize >= start && idx_usize < end {
            adjusted_indices.push((idx_usize - start) as u32);
        }
    }

    let mut final_excerpt = String::new();
    if start > 0 {
        final_excerpt.push_str("...");
    }
    final_excerpt.push_str(&excerpt);
    if end < chars.len() {
        final_excerpt.push_str("...");
    }

    // Adjust indices for the "..." prefix
    let prefix_offset = if start > 0 { 3 } else { 0 };
    let final_indices = adjusted_indices
        .into_iter()
        .map(|i| i + prefix_offset)
        .collect();

    (final_excerpt, final_indices)
}

/// Simple word count for markdown content.
pub fn count_words(content: &str) -> usize {
    content.split_whitespace().filter(|w| !w.is_empty()).count()
}
