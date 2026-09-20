//! Utilities for processing Markdown text.

use regex::Regex;
use std::sync::LazyLock;

/// Matches Markdown images `![alt](url)` and keeps the alt text.
/// Defined before links so `![..](..)` is consumed before `[..](..)`.
static IMAGE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"!\[([^\]]*)\]\([^)]*\)").unwrap());

/// Matches Markdown links `[text](url)` and keeps the link text.
static LINK_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[([^\]]+)\]\([^)]+\)").unwrap());

/// Strips common markdown characters from a line for cleaner display.
pub fn strip_markdown_line(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // 1. Strip leading markers
    let mut s = trimmed;

    // Thread markers (e.g., !!! Thread Name)
    if s.starts_with("!!! ") {
        s = s[4..].trim_start();
    }

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
    // First extract link/image text: ![alt](url) → alt, [text](url) → text.
    // Images before links so the `!` prefix is consumed first.
    let after_links = IMAGE_RE.replace_all(s, "$1");
    let after_links = LINK_RE.replace_all(&after_links, "$1");

    // We remove longer markers first to avoid leaving orphans
    let mut result = after_links
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

///
/// Unit Tests
///
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_and_whitespace() {
        assert_eq!(strip_markdown_line(""), "");
        assert_eq!(strip_markdown_line("   "), "");
        assert_eq!(strip_markdown_line("\t\n"), "");
    }

    #[test]
    fn test_plain_text_unchanged() {
        assert_eq!(strip_markdown_line("Just some text"), "Just some text");
        assert_eq!(strip_markdown_line("  trim me  "), "trim me");
    }

    #[test]
    fn test_leading_markers_stripped() {
        assert_eq!(strip_markdown_line("!!! Thread Name"), "Thread Name");
        assert_eq!(strip_markdown_line("### Title"), "Title");
        assert_eq!(strip_markdown_line("# Heading"), "Heading");
        assert_eq!(strip_markdown_line("> A quote"), "A quote");
    }

    #[test]
    fn test_lists_and_tasks_stripped() {
        assert_eq!(strip_markdown_line("- [ ] unfinished"), "unfinished");
        assert_eq!(strip_markdown_line("- [x] done"), "done");
        assert_eq!(strip_markdown_line("- item"), "item");
        assert_eq!(strip_markdown_line("* item"), "item");
        assert_eq!(strip_markdown_line("+ item"), "item");
    }

    #[test]
    fn test_inline_formatting_stripped() {
        assert_eq!(strip_markdown_line("**bold**"), "bold");
        assert_eq!(strip_markdown_line("__bold__"), "bold");
        assert_eq!(strip_markdown_line("*italic*"), "italic");
        assert_eq!(strip_markdown_line("_italic_"), "italic");
        assert_eq!(strip_markdown_line("`code`"), "code");
        assert_eq!(strip_markdown_line("***both***"), "both");
    }

    #[test]
    fn test_links_and_images_keep_text() {
        assert_eq!(strip_markdown_line("[text](https://x.io)"), "text");
        assert_eq!(strip_markdown_line("![alt](https://x.io/img.png)"), "alt");
    }

    #[test]
    fn test_combined_markers() {
        assert_eq!(strip_markdown_line("### **bold heading**"), "bold heading");
        assert_eq!(strip_markdown_line("- [x] [done](https://x.io)"), "done");
        assert_eq!(
            strip_markdown_line("> ![logo](pic.png) and *em*"),
            "logo and em"
        );
    }

    #[test]
    fn test_trailing_backslash_stripped() {
        assert_eq!(strip_markdown_line("text\\"), "text");
        // Only a single trailing backslash is popped.
        assert_eq!(strip_markdown_line("text\\\\"), "text\\");
    }
}
