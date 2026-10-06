//! Typed schema for a note's YAML frontmatter header.
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

pub mod keys {
    pub const CREATED: &str = "created";
    pub const LAST_MODIFIED: &str = "last-modified";
    pub const NOTE_TYPE: &str = "note-type";
    pub const TAGS: &str = "tags";
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NoteType {
    Auto,
    Manual,
}

impl fmt::Display for NoteType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NoteType::Auto => write!(f, "auto"),
            NoteType::Manual => write!(f, "manual"),
        }
    }
}

impl NoteType {
    /// Parses the frontmatter value of the `note-type` key.
    pub fn from_value(value: &str) -> Option<Self> {
        match value {
            "auto" => Some(NoteType::Auto),
            "manual" => Some(NoteType::Manual),
            _ => None,
        }
    }
}

/// Errors returned when a frontmatter block does not conform to the schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoteHeaderError {
    /// The content does not start with a `---` delimited frontmatter block.
    MissingFrontmatter,
    /// A required field is absent. Currently only `created` is required.
    MissingField(&'static str),
    /// A date field has a value that is not a valid `YYYY-MM-DD` date.
    InvalidDate { field: &'static str, value: String },
    /// The `note-type` field has a value other than `auto` or `manual`.
    InvalidNoteType(String),
    /// A non-empty frontmatter line is not a `key: value` pair.
    MalformedLine(String),
    /// The same key appears more than once.
    DuplicateField(String),
}

impl fmt::Display for NoteHeaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NoteHeaderError::MissingFrontmatter => {
                write!(f, "note has no frontmatter block")
            }
            NoteHeaderError::MissingField(field) => {
                write!(f, "required field '{}' is missing", field)
            }
            NoteHeaderError::InvalidDate { field, value } => {
                write!(f, "field '{}' is not a valid date: '{}'", field, value)
            }
            NoteHeaderError::InvalidNoteType(value) => {
                write!(
                    f,
                    "field '{}' is not a valid note type: '{}'",
                    keys::NOTE_TYPE,
                    value
                )
            }
            NoteHeaderError::MalformedLine(line) => {
                write!(f, "frontmatter line is not a key/value pair: '{}'", line)
            }
            NoteHeaderError::DuplicateField(key) => {
                write!(f, "field '{}' appears more than once", key)
            }
        }
    }
}

impl std::error::Error for NoteHeaderError {}

impl From<NoteHeaderError> for String {
    fn from(error: NoteHeaderError) -> Self {
        error.to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteHeader {
    pub created: NaiveDate,
    pub last_modified: Option<NaiveDate>,
    pub note_type: NoteType,
    pub tags: Vec<String>,
    pub extra: Vec<(String, String)>,
}

impl NoteHeader {
    /// Builds a header with the given dates and type, without tags or extra keys.
    pub fn new(created: NaiveDate, last_modified: NaiveDate, note_type: NoteType) -> Self {
        Self {
            created,
            last_modified: Some(last_modified),
            note_type,
            tags: Vec::new(),
            extra: Vec::new(),
        }
    }

    /// Builds a header for a newly created note, with `created` and
    /// `last-modified` set to today.
    pub fn for_new_note(note_type: NoteType) -> Result<Self, NoteHeaderError> {
        let today = crate::utils::date::get_current_date();
        let date = NaiveDate::parse_from_str(&today, "%Y-%m-%d").map_err(|_| {
            NoteHeaderError::InvalidDate {
                field: keys::CREATED,
                value: today.clone(),
            }
        })?;
        Ok(Self::new(date, date, note_type))
    }

    /// Finds the frontmatter block in the given lines.
    ///
    /// Returns the `(start, end)` indices of the opening and closing `---`
    /// delimiters, or `None` if the content has no frontmatter block.
    pub fn find_frontmatter_range(lines: &[&str]) -> Option<(usize, usize)> {
        if lines.is_empty() || lines[0].trim() != "---" {
            return None;
        }
        lines
            .iter()
            .skip(1)
            .position(|l| l.trim() == "---")
            .map(|idx| (0, idx + 1))
    }

    /// Parses a note's full content into a typed header.
    ///
    /// Only `created` is required; see the module documentation for the
    /// legacy defaults applied to the optional fields.
    pub fn parse(content: &str) -> Result<Self, NoteHeaderError> {
        let lines: Vec<&str> = content.lines().collect();
        let (_, end) =
            Self::find_frontmatter_range(&lines).ok_or(NoteHeaderError::MissingFrontmatter)?;

        let mut created: Option<NaiveDate> = None;
        let mut last_modified: Option<NaiveDate> = None;
        let mut note_type: Option<NoteType> = None;
        let mut tags: Option<Vec<String>> = None;
        let mut extra: Vec<(String, String)> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();

        for line in &lines[1..end] {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let (key, value) = trimmed
                .split_once(':')
                .ok_or_else(|| NoteHeaderError::MalformedLine(line.to_string()))?;
            let key = key.trim();
            let value = value.trim();

            if !seen.insert(key.to_string()) {
                return Err(NoteHeaderError::DuplicateField(key.to_string()));
            }

            match key {
                keys::CREATED => {
                    created = Some(Self::parse_date(keys::CREATED, value)?);
                }
                keys::LAST_MODIFIED => {
                    last_modified = Some(Self::parse_date(keys::LAST_MODIFIED, value)?);
                }
                keys::NOTE_TYPE => {
                    note_type = Some(
                        NoteType::from_value(value)
                            .ok_or_else(|| NoteHeaderError::InvalidNoteType(value.to_string()))?,
                    );
                }
                keys::TAGS => {
                    tags = Some(crate::utils::tag_parser::parse_tags_from_yaml_value(value));
                }
                _ => extra.push((key.to_string(), value.to_string())),
            }
        }

        Ok(Self {
            created: created.ok_or(NoteHeaderError::MissingField(keys::CREATED))?,
            last_modified,
            note_type: note_type.unwrap_or(NoteType::Auto),
            tags: tags.unwrap_or_default(),
            extra,
        })
    }

    /// Renders the header as a canonical frontmatter block, including the
    /// `---` delimiters and a trailing newline.
    pub fn render(&self) -> String {
        let mut lines = vec![String::from("---")];
        lines.push(format!(
            "{}: {}",
            keys::CREATED,
            self.created.format("%Y-%m-%d")
        ));
        if let Some(last_modified) = self.last_modified {
            lines.push(format!(
                "{}: {}",
                keys::LAST_MODIFIED,
                last_modified.format("%Y-%m-%d")
            ));
        }
        lines.push(format!("{}: {}", keys::NOTE_TYPE, self.note_type));
        lines.push(self.render_tags());
        for (key, value) in &self.extra {
            lines.push(format!("{}: {}", key, value));
        }
        lines.push(String::from("---"));

        let mut content = lines.join("\n");
        content.push('\n');
        content
    }

    /// Renders the `tags` line as an inline list.
    fn render_tags(&self) -> String {
        if self.tags.is_empty() {
            return format!("{}: []", keys::TAGS);
        }
        format!("{}: [{}]", keys::TAGS, self.tags.join(", "))
    }

    /// Parses a `YYYY-MM-DD` field value.
    fn parse_date(field: &'static str, value: &str) -> Result<NaiveDate, NoteHeaderError> {
        NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| NoteHeaderError::InvalidDate {
            field,
            value: value.to_string(),
        })
    }
}

///
/// Unit Tests
///
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_header_with_all_fields() {
        let content = "---\ncreated: 2024-01-15\nlast-modified: 2024-01-16\nnote-type: manual\ntags: [work, urgent]\nthreads: abc:0,def:2:pinned\n---\nbody";
        let header = NoteHeader::parse(content).expect("Valid header should parse");

        assert_eq!(
            header.created,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()
        );
        assert_eq!(
            header.last_modified,
            Some(NaiveDate::from_ymd_opt(2024, 1, 16).unwrap())
        );
        assert_eq!(header.note_type, NoteType::Manual);
        assert_eq!(header.tags, vec!["work", "urgent"]);
        assert_eq!(
            header.extra,
            vec![("threads".to_string(), "abc:0,def:2:pinned".to_string())]
        );
    }

    #[test]
    fn test_parse_legacy_header_defaults() {
        // Pre note-type notes without last-modified still parse
        let content = "---\ncreated: 2024-01-15\ntags: []\n---\nbody";
        let header = NoteHeader::parse(content).expect("Legacy header should parse");

        assert_eq!(header.note_type, NoteType::Auto);
        assert_eq!(header.last_modified, None);
        assert!(header.tags.is_empty());
        assert!(header.extra.is_empty());
    }

    #[test]
    fn test_parse_missing_frontmatter_errors() {
        assert_eq!(
            NoteHeader::parse("just body"),
            Err(NoteHeaderError::MissingFrontmatter)
        );
        assert_eq!(
            NoteHeader::parse("---\ncreated: 2024-01-15\n"),
            Err(NoteHeaderError::MissingFrontmatter)
        );
    }

    #[test]
    fn test_parse_missing_created_errors() {
        let content = "---\ntags: []\n---\nbody";
        assert_eq!(
            NoteHeader::parse(content),
            Err(NoteHeaderError::MissingField(keys::CREATED))
        );
    }

    #[test]
    fn test_parse_invalid_created_errors() {
        let content = "---\ncreated: yesterday\n---\nbody";
        assert_eq!(
            NoteHeader::parse(content),
            Err(NoteHeaderError::InvalidDate {
                field: keys::CREATED,
                value: "yesterday".to_string()
            })
        );
    }

    #[test]
    fn test_parse_invalid_note_type_errors() {
        let content = "---\ncreated: 2024-01-15\nnote-type: bogus\n---\nbody";
        assert_eq!(
            NoteHeader::parse(content),
            Err(NoteHeaderError::InvalidNoteType("bogus".to_string()))
        );
    }

    #[test]
    fn test_parse_malformed_line_errors() {
        let content = "---\ncreated: 2024-01-15\nnot a pair\n---\nbody";
        assert_eq!(
            NoteHeader::parse(content),
            Err(NoteHeaderError::MalformedLine("not a pair".to_string()))
        );
    }

    #[test]
    fn test_parse_duplicate_field_errors() {
        let content = "---\ncreated: 2024-01-15\ncreated: 2024-01-16\n---\nbody";
        assert_eq!(
            NoteHeader::parse(content),
            Err(NoteHeaderError::DuplicateField("created".to_string()))
        );
    }

    #[test]
    fn test_render_canonical_format() {
        let header = NoteHeader::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NoteType::Auto,
        );
        assert_eq!(
            header.render(),
            "---\ncreated: 2024-01-15\nlast-modified: 2024-01-15\nnote-type: auto\ntags: []\n---\n"
        );
    }

    #[test]
    fn test_render_preserves_extra_keys() {
        let mut header = NoteHeader::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NoteType::Manual,
        );
        header
            .extra
            .push(("title".to_string(), "My Note".to_string()));
        assert_eq!(
            header.render(),
            "---\ncreated: 2024-01-15\nlast-modified: 2024-01-15\nnote-type: manual\ntags: []\ntitle: My Note\n---\n"
        );
    }

    #[test]
    fn test_parse_render_round_trip() {
        let content = "---\ncreated: 2024-01-15\nlast-modified: 2024-01-16\nnote-type: manual\ntags: [a, b]\nthreads: id:0\n---\nbody";
        let header = NoteHeader::parse(content).expect("Round-trip parse should succeed");
        assert_eq!(header.render(), "---\ncreated: 2024-01-15\nlast-modified: 2024-01-16\nnote-type: manual\ntags: [a, b]\nthreads: id:0\n---\n");
    }

    #[test]
    fn test_for_new_note_uses_today() {
        let header =
            NoteHeader::for_new_note(NoteType::Manual).expect("Today's date should be valid");
        let today = crate::utils::date::get_current_date();
        let expected = NaiveDate::parse_from_str(&today, "%Y-%m-%d").unwrap();
        assert_eq!(header.created, expected);
        assert_eq!(header.last_modified, Some(expected));
        assert_eq!(header.note_type, NoteType::Manual);
    }

    #[test]
    fn test_find_frontmatter_range() {
        assert_eq!(
            NoteHeader::find_frontmatter_range(&["---", "created: x", "---", "body"]),
            Some((0, 2))
        );
        assert_eq!(NoteHeader::find_frontmatter_range(&["body"]), None);
        assert_eq!(NoteHeader::find_frontmatter_range(&[]), None);
    }
}
