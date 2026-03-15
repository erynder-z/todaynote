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
