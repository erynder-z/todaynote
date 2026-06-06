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
        .replace("`", "")
        .replace("*", "")
        .replace("_", "");

    // 3. Strip trailing line-break backslashes
    if result.ends_with('\\') {
        result.pop();
    }

    result.trim().to_string()
}
