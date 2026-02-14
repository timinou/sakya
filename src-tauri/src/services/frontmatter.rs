use crate::error::AppError;
use serde::de::DeserializeOwned;
use serde::Serialize;

const FRONTMATTER_DELIMITER: &str = "---";

/// Parsed document with YAML frontmatter and Markdown body.
#[derive(Debug, Clone)]
pub struct ParsedDocument<T> {
    pub frontmatter: T,
    pub body: String,
}

/// Parse a Markdown string that may have YAML frontmatter delimited by `---`.
pub fn parse<T: DeserializeOwned>(content: &str) -> Result<ParsedDocument<T>, AppError> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with(FRONTMATTER_DELIMITER) {
        return Err(AppError::Validation(
            "Document does not start with frontmatter delimiter".to_string(),
        ));
    }

    let after_first = &trimmed[FRONTMATTER_DELIMITER.len()..];
    let end_pos = after_first
        .find(&format!("\n{}", FRONTMATTER_DELIMITER))
        .ok_or_else(|| AppError::Validation("Missing closing frontmatter delimiter".to_string()))?;

    let yaml_str = &after_first[..end_pos];
    let body_start = end_pos + 1 + FRONTMATTER_DELIMITER.len();
    let body = after_first[body_start..]
        .trim_start_matches('\n')
        .to_string();

    let frontmatter: T = serde_yaml::from_str(yaml_str)?;
    Ok(ParsedDocument { frontmatter, body })
}

/// Serialize a document with YAML frontmatter and Markdown body.
pub fn serialize<T: Serialize>(frontmatter: &T, body: &str) -> Result<String, AppError> {
    let yaml = serde_yaml::to_string(frontmatter)?;
    Ok(format!("---\n{}---\n{}", yaml, body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestFrontmatter {
        title: String,
        #[serde(default)]
        tags: Vec<String>,
    }

    #[test]
    fn parse_with_frontmatter() {
        let content = "---\ntitle: Hello World\ntags:\n  - test\n  - demo\n---\nThis is the body.\n\nSecond paragraph.";
        let doc: ParsedDocument<TestFrontmatter> = parse(content).unwrap();
        assert_eq!(doc.frontmatter.title, "Hello World");
        assert_eq!(doc.frontmatter.tags, vec!["test", "demo"]);
        assert_eq!(doc.body, "This is the body.\n\nSecond paragraph.");
    }

    #[test]
    fn parse_without_frontmatter_errors() {
        let content = "Just a regular document.";
        let result: Result<ParsedDocument<TestFrontmatter>, _> = parse(content);
        assert!(result.is_err());
    }

    #[test]
    fn parse_empty_body() {
        let content = "---\ntitle: Empty\ntags: []\n---\n";
        let doc: ParsedDocument<TestFrontmatter> = parse(content).unwrap();
        assert_eq!(doc.frontmatter.title, "Empty");
        assert_eq!(doc.body, "");
    }

    #[test]
    fn round_trip() {
        let fm = TestFrontmatter {
            title: "Round Trip".to_string(),
            tags: vec!["a".to_string(), "b".to_string()],
        };
        let body = "Body content here.\n";
        let serialized = serialize(&fm, body).unwrap();
        let parsed: ParsedDocument<TestFrontmatter> = parse(&serialized).unwrap();
        assert_eq!(parsed.frontmatter.title, "Round Trip");
        assert_eq!(parsed.frontmatter.tags, vec!["a", "b"]);
        assert_eq!(parsed.body, body);
    }

    #[test]
    fn parse_with_body_containing_dashes() {
        let content = "---\ntitle: Dashes\ntags: []\n---\nSome text with --- dashes in it.\n\nAnother --- line.";
        let doc: ParsedDocument<TestFrontmatter> = parse(content).unwrap();
        assert_eq!(doc.frontmatter.title, "Dashes");
        assert!(doc.body.contains("--- dashes"));
    }
}
