//! Utilities for text processing and analysis.

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

/// Simple word count for text content.
/// Skips YAML frontmatter and markdown formatting, only counting actual words.
/// Returns (word_count, has_code) tuple where has_code indicates if code blocks are present.
pub fn count_words(content: &str) -> (usize, bool) {
    // Skip YAML frontmatter if present
    let lines: Vec<&str> = content.lines().collect();
    let mut start_idx = 0;

    // Check if content starts with YAML frontmatter
    if lines.first().map(|l| l.trim()) == Some("---") {
        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.trim() == "---" {
                start_idx = i + 1;
                break;
            }
        }
    }

    // Get content after frontmatter
    let content_after_frontmatter = lines[start_idx..].join("\n");

    // Strip basic markdown formatting from each line
    // Track code blocks and HTML content
    let mut in_code_block = false;
    let mut has_code = false;
    let stripped_lines: Vec<String> = content_after_frontmatter
        .lines()
        .map(|line| {
            let mut line = line.to_string();

            // Track code blocks (lines starting with ```)
            if line.trim_start().starts_with("```") {
                in_code_block = !in_code_block;
                if in_code_block {
                    has_code = true;
                }
                return String::new(); // Skip code block markers
            }

            if in_code_block {
                has_code = true;
                return String::new(); // Skip content inside fenced code blocks
            }

            // Detect indented code blocks (4 spaces or 1 tab at start)
            if line.starts_with("    ") || line.starts_with("\t") {
                has_code = true;
                return String::new(); // Skip indented code blocks
            }

            // Skip HTML content (lines starting with <)
            if line.trim_start().starts_with('<') {
                return String::new();
            }

            // Remove thread markers (!!!)
            if line.starts_with("!!! ") {
                line = line[4..].trim_start().to_string();
            }

            // Remove heading markers (#, ##, etc.)
            if line.starts_with('#') {
                line = line.trim_start_matches('#').trim_start().to_string();
            }

            // Remove list markers (-, *, +, etc.)
            if line.starts_with("-") || line.starts_with("*") || line.starts_with("+") {
                line = line[1..].trim_start().to_string();
            }

            // Remove blockquote markers (>>)
            if line.starts_with(">") {
                line = line[1..].trim_start().to_string();
            }

            // Remove inline markdown markers (bold, italic, code, etc.)
            line = line
                .replace("***", "")
                .replace("___", "")
                .replace("**", "")
                .replace("__", "")
                .replace("`", "")
                .replace("*", "")
                .replace("_", "");

            line
        })
        .collect();

    let stripped_content = stripped_lines.join(" ");

    // Count words, filtering out empty strings and markdown-only artifacts
    let word_count = stripped_content
        .split_whitespace()
        .filter(|w| {
            if w.is_empty() {
                return false;
            }

            // Filter out words that are just punctuation or markdown artifacts
            let has_letters = w.chars().any(|c| c.is_alphabetic());
            if !has_letters {
                return false;
            }

            // Filter out very short words that are likely artifacts
            if w.len() == 1 {
                return false;
            }

            true
        })
        .count();

    (word_count, has_code)
}
