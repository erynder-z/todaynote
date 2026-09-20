//! Utilities for parsing tags from Markdown content and YAML frontmatter.

/// Parses tags from the YAML frontmatter of a Markdown string.
///
/// Expects the note to start with a `---` delimiter.
pub fn parse_tags_from_content(content: &str) -> Vec<String> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() < 2 || lines[0].trim() != "---" {
        return Vec::new();
    }

    let mut end_fm = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            end_fm = Some(i);
            break;
        }
    }

    if let Some(end) = end_fm {
        for i in 1..end {
            let line = lines[i];
            if let Some((key, value)) = line.split_once(':') {
                if key.trim() == "tags" {
                    return parse_tags_from_yaml_value(value.trim());
                }
            }
        }
    }
    Vec::new()
}

/// Parses a YAML tag value string into a vector of strings.
///
/// Handles both inline lists `[tag1, tag2]` and comma-separated strings `tag1, tag2`.
pub fn parse_tags_from_yaml_value(value: &str) -> Vec<String> {
    if value.is_empty() {
        return Vec::new();
    }

    // Remove brackets if present: [tag1, tag2] -> tag1, tag2
    let inner = if value.starts_with('[') && value.ends_with(']') {
        &value[1..value.len() - 1]
    } else {
        value
    };

    inner
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

///
/// Unit Tests
///
#[cfg(test)]
mod tests {
    use super::*;

    ///
    /// parse_tags_from_yaml_value tests
    ///
    #[test]
    fn test_parse_yaml_value_empty() {
        assert!(parse_tags_from_yaml_value("").is_empty());
    }

    #[test]
    fn test_parse_yaml_value_comma_separated() {
        assert_eq!(
            parse_tags_from_yaml_value("tag1, tag2,tag3"),
            vec!["tag1", "tag2", "tag3"]
        );
    }

    #[test]
    fn test_parse_yaml_value_inline_list() {
        assert_eq!(
            parse_tags_from_yaml_value("[tag1, tag2, tag3]"),
            vec!["tag1", "tag2", "tag3"]
        );
    }

    #[test]
    fn test_parse_yaml_value_skips_empty_entries() {
        assert_eq!(
            parse_tags_from_yaml_value("tag1,, tag2, ,"),
            vec!["tag1", "tag2"]
        );
    }
    ///
    /// parse_tags_from_content tests
    ///
    #[test]
    fn test_no_frontmatter_returns_empty() {
        assert!(parse_tags_from_content("just body text").is_empty());
        assert!(parse_tags_from_content("---\ntags: [a]\n").is_empty());
    }

    #[test]
    fn test_frontmatter_tags_inline_list() {
        let content = "---\ntags: [work, urgent]\n---\nbody";
        assert_eq!(parse_tags_from_content(content), vec!["work", "urgent"]);
    }

    #[test]
    fn test_frontmatter_tags_comma_separated() {
        let content = "---\ntags: rust, cli\n---\n# Note";
        assert_eq!(parse_tags_from_content(content), vec!["rust", "cli"]);
    }

    #[test]
    fn test_frontmatter_without_tags_returns_empty() {
        let content = "---\ntitle: My Note\n---\nbody";
        assert!(parse_tags_from_content(content).is_empty());
    }

    #[test]
    fn test_frontmatter_tags_key_must_match() {
        let content = "---\nmytags: [a]\ntags: [b]\n---\n";
        assert_eq!(parse_tags_from_content(content), vec!["b"]);
    }
}
