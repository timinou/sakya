use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ChapterHeaderStyle {
    Numbered,
    Titled,
    NumberedAndTitled,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ChapterSeparator {
    PageBreak,
    ThreeStars,
    HorizontalRule,
    BlankLines,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Markdown,
    Html,
    PlainText,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileConfig {
    pub title: String,
    pub author: String,
    pub include_title_page: bool,
    pub chapter_header_style: ChapterHeaderStyle,
    pub chapter_separator: ChapterSeparator,
    pub output_format: OutputFormat,
    pub include_synopsis: bool,
    pub front_matter: String,
}

impl Default for CompileConfig {
    fn default() -> Self {
        Self {
            title: String::new(),
            author: String::new(),
            include_title_page: true,
            chapter_header_style: ChapterHeaderStyle::NumberedAndTitled,
            chapter_separator: ChapterSeparator::PageBreak,
            output_format: OutputFormat::Markdown,
            include_synopsis: false,
            front_matter: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileOutput {
    pub content: String,
    pub format: OutputFormat,
    pub chapter_count: usize,
    pub word_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let config = CompileConfig::default();
        assert_eq!(config.title, "");
        assert_eq!(config.author, "");
        assert!(config.include_title_page);
        assert_eq!(
            config.chapter_header_style,
            ChapterHeaderStyle::NumberedAndTitled
        );
        assert_eq!(config.chapter_separator, ChapterSeparator::PageBreak);
        assert_eq!(config.output_format, OutputFormat::Markdown);
        assert!(!config.include_synopsis);
        assert_eq!(config.front_matter, "");
    }

    #[test]
    fn test_compile_config_serialization_roundtrip() {
        let config = CompileConfig {
            title: "My Novel".to_string(),
            author: "Jane Doe".to_string(),
            include_title_page: false,
            chapter_header_style: ChapterHeaderStyle::Titled,
            chapter_separator: ChapterSeparator::ThreeStars,
            output_format: OutputFormat::Html,
            include_synopsis: true,
            front_matter: "Dedication: To everyone.".to_string(),
        };

        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: CompileConfig = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.title, "My Novel");
        assert_eq!(deserialized.author, "Jane Doe");
        assert!(!deserialized.include_title_page);
        assert_eq!(
            deserialized.chapter_header_style,
            ChapterHeaderStyle::Titled
        );
        assert_eq!(deserialized.chapter_separator, ChapterSeparator::ThreeStars);
        assert_eq!(deserialized.output_format, OutputFormat::Html);
        assert!(deserialized.include_synopsis);
        assert_eq!(deserialized.front_matter, "Dedication: To everyone.");
    }

    #[test]
    fn test_compile_output_serialization_roundtrip() {
        let output = CompileOutput {
            content: "# Chapter 1\n\nSome text.".to_string(),
            format: OutputFormat::Markdown,
            chapter_count: 1,
            word_count: 3,
        };

        let json = serde_json::to_string(&output).expect("serialize");
        let deserialized: CompileOutput = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.content, "# Chapter 1\n\nSome text.");
        assert_eq!(deserialized.format, OutputFormat::Markdown);
        assert_eq!(deserialized.chapter_count, 1);
        assert_eq!(deserialized.word_count, 3);
    }

    #[test]
    fn test_chapter_header_style_enum_serialization() {
        assert_eq!(
            serde_json::to_string(&ChapterHeaderStyle::Numbered).unwrap(),
            "\"numbered\""
        );
        assert_eq!(
            serde_json::to_string(&ChapterHeaderStyle::Titled).unwrap(),
            "\"titled\""
        );
        assert_eq!(
            serde_json::to_string(&ChapterHeaderStyle::NumberedAndTitled).unwrap(),
            "\"numbered_and_titled\""
        );
        assert_eq!(
            serde_json::to_string(&ChapterHeaderStyle::None).unwrap(),
            "\"none\""
        );
    }

    #[test]
    fn test_chapter_separator_enum_serialization() {
        assert_eq!(
            serde_json::to_string(&ChapterSeparator::PageBreak).unwrap(),
            "\"page_break\""
        );
        assert_eq!(
            serde_json::to_string(&ChapterSeparator::ThreeStars).unwrap(),
            "\"three_stars\""
        );
        assert_eq!(
            serde_json::to_string(&ChapterSeparator::HorizontalRule).unwrap(),
            "\"horizontal_rule\""
        );
        assert_eq!(
            serde_json::to_string(&ChapterSeparator::BlankLines).unwrap(),
            "\"blank_lines\""
        );
    }

    #[test]
    fn test_output_format_enum_serialization() {
        assert_eq!(
            serde_json::to_string(&OutputFormat::Markdown).unwrap(),
            "\"markdown\""
        );
        assert_eq!(
            serde_json::to_string(&OutputFormat::Html).unwrap(),
            "\"html\""
        );
        assert_eq!(
            serde_json::to_string(&OutputFormat::PlainText).unwrap(),
            "\"plain_text\""
        );
    }

    #[test]
    fn test_enum_deserialization_from_snake_case() {
        let style: ChapterHeaderStyle = serde_json::from_str("\"numbered_and_titled\"").unwrap();
        assert_eq!(style, ChapterHeaderStyle::NumberedAndTitled);

        let sep: ChapterSeparator = serde_json::from_str("\"three_stars\"").unwrap();
        assert_eq!(sep, ChapterSeparator::ThreeStars);

        let fmt: OutputFormat = serde_json::from_str("\"plain_text\"").unwrap();
        assert_eq!(fmt, OutputFormat::PlainText);
    }

    #[test]
    fn test_compile_config_camel_case_field_names() {
        let config = CompileConfig::default();
        let json = serde_json::to_string(&config).unwrap();

        // Verify camelCase field names in serialized JSON
        assert!(json.contains("\"includeTitlePage\""));
        assert!(json.contains("\"chapterHeaderStyle\""));
        assert!(json.contains("\"chapterSeparator\""));
        assert!(json.contains("\"outputFormat\""));
        assert!(json.contains("\"includeSynopsis\""));
        assert!(json.contains("\"frontMatter\""));

        // Verify snake_case field names are NOT present
        assert!(!json.contains("\"include_title_page\""));
        assert!(!json.contains("\"chapter_header_style\""));
    }

    #[test]
    fn test_compile_output_camel_case_field_names() {
        let output = CompileOutput {
            content: String::new(),
            format: OutputFormat::Markdown,
            chapter_count: 0,
            word_count: 0,
        };
        let json = serde_json::to_string(&output).unwrap();

        assert!(json.contains("\"chapterCount\""));
        assert!(json.contains("\"wordCount\""));
        assert!(!json.contains("\"chapter_count\""));
        assert!(!json.contains("\"word_count\""));
    }
}
