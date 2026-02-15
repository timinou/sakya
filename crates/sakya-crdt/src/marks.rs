//! Bidirectional mapping between Markdown formatting and Loro rich text marks.
//!
//! Provides conversion utilities for translating between Markdown syntax
//! and Loro CRDT rich text marks. Used by the Lexical-Loro binding and
//! the file import/export pipeline.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

// ── Mark types ───────────────────────────────────────────────────────────────

/// Expand behavior for a mark type when text is inserted at its boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpandBehavior {
    /// Typing at mark boundary extends the mark (e.g., bold, italic).
    After,
    /// Typing at mark boundary does not extend the mark (e.g., link, code).
    None,
}

/// Registry of known mark types with their expand behaviors.
///
/// The registry defines which mark keys are recognized and how they
/// behave at boundaries. Custom marks (e.g., highlights) can be
/// registered for extensibility.
#[derive(Debug, Clone)]
pub struct MarkRegistry {
    types: HashMap<String, ExpandBehavior>,
}

impl MarkRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
        }
    }

    /// Create a registry with the default built-in marks.
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Inline formatting marks (expand after — typing at boundary extends mark)
        registry.register("bold", ExpandBehavior::After);
        registry.register("italic", ExpandBehavior::After);
        registry.register("strikethrough", ExpandBehavior::After);

        // Non-expanding marks (typing at boundary doesn't extend mark)
        registry.register("code", ExpandBehavior::None);
        registry.register("link", ExpandBehavior::None);
        registry.register("wiki-link", ExpandBehavior::None);

        // Block-level marks
        registry.register("heading", ExpandBehavior::None);
        registry.register("blockquote", ExpandBehavior::None);
        registry.register("list", ExpandBehavior::None);
        registry.register("code-block", ExpandBehavior::None);

        registry
    }

    /// Register a custom mark type.
    pub fn register(&mut self, key: &str, expand: ExpandBehavior) {
        self.types.insert(key.to_string(), expand);
    }

    /// Check if a mark key is registered.
    pub fn is_registered(&self, key: &str) -> bool {
        self.types.contains_key(key)
    }

    /// Get the expand behavior for a mark key.
    pub fn expand_behavior(&self, key: &str) -> Option<ExpandBehavior> {
        self.types.get(key).copied()
    }

    /// List all registered mark keys.
    pub fn keys(&self) -> Vec<&str> {
        self.types.keys().map(|k| k.as_str()).collect()
    }
}

impl Default for MarkRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

// ── MarkedText ───────────────────────────────────────────────────────────────

/// A mark applied to a range of text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkSpan {
    /// Start position (Unicode scalar offset in plain text).
    pub start: usize,
    /// End position (exclusive, Unicode scalar offset).
    pub end: usize,
    /// Mark key (e.g., "bold", "italic", "link").
    pub key: String,
    /// Mark value (e.g., true for bold, URL string for link).
    pub value: JsonValue,
}

/// Plain text with associated mark spans.
///
/// Intermediate representation between Markdown and Loro rich text marks.
/// `plain_text` is visible text (inline syntax stripped, block syntax preserved),
/// and `marks` describes inline formatting applied to ranges.
#[derive(Debug, Clone)]
pub struct MarkedText {
    /// The plain text content (inline formatting syntax removed).
    pub plain_text: String,
    /// Mark spans applied to ranges of the plain text.
    pub marks: Vec<MarkSpan>,
}

// ── Markdown → Marks ─────────────────────────────────────────────────────────

/// Parse markdown into plain text with inline mark spans.
///
/// Extracts inline formatting (bold, italic, code, strikethrough, links)
/// as mark spans. Block-level structure (paragraphs, headings, lists) is
/// preserved as text conventions (newlines).
pub fn markdown_to_marks(markdown: &str) -> MarkedText {
    use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(markdown, options);

    let mut plain_text = String::new();
    let mut marks: Vec<MarkSpan> = Vec::new();
    // Stack: (mark_key, mark_value, start_position)
    let mut mark_stack: Vec<(String, JsonValue, usize)> = Vec::new();
    let mut block_count = 0u32;

    for event in parser {
        match event {
            // ── Block-level events ──
            Event::Start(Tag::Paragraph) => {
                if block_count > 0 {
                    plain_text.push_str("\n\n");
                }
                block_count += 1;
            }
            Event::End(TagEnd::Paragraph) => {}

            Event::Start(Tag::Heading { level, .. }) => {
                if block_count > 0 {
                    plain_text.push_str("\n\n");
                }
                block_count += 1;
                // Preserve heading prefix in text
                for _ in 0..level as usize {
                    plain_text.push('#');
                }
                plain_text.push(' ');
            }
            Event::End(TagEnd::Heading(_)) => {}

            Event::Start(Tag::BlockQuote(_)) => {
                if block_count > 0 {
                    plain_text.push_str("\n\n");
                }
                block_count += 1;
                plain_text.push_str("> ");
            }
            Event::End(TagEnd::BlockQuote(_)) => {}

            Event::Start(Tag::List(_)) => {}
            Event::End(TagEnd::List(_)) => {}

            Event::Start(Tag::Item) => {
                if !plain_text.is_empty() && !plain_text.ends_with('\n') {
                    plain_text.push('\n');
                }
                plain_text.push_str("- ");
            }
            Event::End(TagEnd::Item) => {}

            // ── Inline formatting ──
            Event::Start(Tag::Strong) => {
                mark_stack.push(("bold".into(), JsonValue::Bool(true), plain_text.len()));
            }
            Event::End(TagEnd::Strong) => {
                if let Some((key, value, start)) = mark_stack.pop() {
                    if start < plain_text.len() {
                        marks.push(MarkSpan {
                            start,
                            end: plain_text.len(),
                            key,
                            value,
                        });
                    }
                }
            }

            Event::Start(Tag::Emphasis) => {
                mark_stack.push(("italic".into(), JsonValue::Bool(true), plain_text.len()));
            }
            Event::End(TagEnd::Emphasis) => {
                if let Some((key, value, start)) = mark_stack.pop() {
                    if start < plain_text.len() {
                        marks.push(MarkSpan {
                            start,
                            end: plain_text.len(),
                            key,
                            value,
                        });
                    }
                }
            }

            Event::Start(Tag::Strikethrough) => {
                mark_stack.push((
                    "strikethrough".into(),
                    JsonValue::Bool(true),
                    plain_text.len(),
                ));
            }
            Event::End(TagEnd::Strikethrough) => {
                if let Some((key, value, start)) = mark_stack.pop() {
                    if start < plain_text.len() {
                        marks.push(MarkSpan {
                            start,
                            end: plain_text.len(),
                            key,
                            value,
                        });
                    }
                }
            }

            Event::Start(Tag::Link { dest_url, .. }) => {
                mark_stack.push((
                    "link".into(),
                    JsonValue::String(dest_url.to_string()),
                    plain_text.len(),
                ));
            }
            Event::End(TagEnd::Link) => {
                if let Some((key, value, start)) = mark_stack.pop() {
                    if start < plain_text.len() {
                        marks.push(MarkSpan {
                            start,
                            end: plain_text.len(),
                            key,
                            value,
                        });
                    }
                }
            }

            // ── Content events ──
            Event::Text(text) => {
                plain_text.push_str(&text);
            }

            Event::Code(code) => {
                let start = plain_text.len();
                plain_text.push_str(&code);
                marks.push(MarkSpan {
                    start,
                    end: plain_text.len(),
                    key: "code".into(),
                    value: JsonValue::Bool(true),
                });
            }

            Event::SoftBreak => {
                plain_text.push('\n');
            }

            Event::HardBreak => {
                plain_text.push('\n');
            }

            _ => {}
        }
    }

    MarkedText { plain_text, marks }
}

// ── Marks → Markdown ─────────────────────────────────────────────────────────

/// Convert marked text back to Markdown.
///
/// Translates inline mark spans (bold, italic, code, strikethrough, link)
/// into Markdown syntax. Block-level text (headings, lists) is already
/// present in the plain text.
pub fn marks_to_markdown(marked: &MarkedText) -> String {
    if marked.marks.is_empty() {
        return marked.plain_text.clone();
    }

    // Collect boundary events (open/close) at each character position
    let mut events: Vec<(usize, bool, usize)> = Vec::new(); // (pos, is_open, mark_idx)
    for (i, mark) in marked.marks.iter().enumerate() {
        events.push((mark.start, true, i));
        events.push((mark.end, false, i));
    }

    // Sort: same position → closes before opens; same type → by mark index
    events.sort_by(|a, b| {
        a.0.cmp(&b.0).then_with(|| match (a.1, b.1) {
            (false, true) => std::cmp::Ordering::Less, // close before open
            (true, false) => std::cmp::Ordering::Greater,
            _ => {
                if !a.1 {
                    // Both closes: reverse order (LIFO)
                    b.2.cmp(&a.2)
                } else {
                    // Both opens: forward order
                    a.2.cmp(&b.2)
                }
            }
        })
    });

    let chars: Vec<char> = marked.plain_text.chars().collect();
    let mut result = String::new();
    let mut event_idx = 0;

    for (char_idx, ch) in chars.iter().enumerate() {
        // Process events at this position
        while event_idx < events.len() && events[event_idx].0 == char_idx {
            let (_, is_open, mark_idx) = events[event_idx];
            let mark = &marked.marks[mark_idx];
            if is_open {
                result.push_str(&mark_open_syntax(&mark.key, &mark.value));
            } else {
                result.push_str(&mark_close_syntax(&mark.key, &mark.value));
            }
            event_idx += 1;
        }
        result.push(*ch);
    }

    // Process any remaining close events at the end of text
    while event_idx < events.len() {
        let (_, is_open, mark_idx) = events[event_idx];
        if !is_open {
            let mark = &marked.marks[mark_idx];
            result.push_str(&mark_close_syntax(&mark.key, &mark.value));
        }
        event_idx += 1;
    }

    result
}

fn mark_open_syntax(key: &str, _value: &JsonValue) -> String {
    match key {
        "bold" => "**".to_string(),
        "italic" => "*".to_string(),
        "code" => "`".to_string(),
        "strikethrough" => "~~".to_string(),
        "link" => "[".to_string(),
        _ => String::new(),
    }
}

fn mark_close_syntax(key: &str, value: &JsonValue) -> String {
    match key {
        "bold" => "**".to_string(),
        "italic" => "*".to_string(),
        "code" => "`".to_string(),
        "strikethrough" => "~~".to_string(),
        "link" => {
            if let JsonValue::String(url) = value {
                format!("]({})", url)
            } else {
                "]()".to_string()
            }
        }
        _ => String::new(),
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── MarkRegistry tests ───────────────────────────────────────────────

    #[test]
    fn registry_defaults_include_built_in_marks() {
        let registry = MarkRegistry::with_defaults();
        assert!(registry.is_registered("bold"));
        assert!(registry.is_registered("italic"));
        assert!(registry.is_registered("code"));
        assert!(registry.is_registered("strikethrough"));
        assert!(registry.is_registered("link"));
        assert!(registry.is_registered("wiki-link"));
        assert!(registry.is_registered("heading"));
        assert!(registry.is_registered("blockquote"));
    }

    #[test]
    fn registry_expand_behaviors() {
        let registry = MarkRegistry::with_defaults();
        assert_eq!(
            registry.expand_behavior("bold"),
            Some(ExpandBehavior::After)
        );
        assert_eq!(
            registry.expand_behavior("italic"),
            Some(ExpandBehavior::After)
        );
        assert_eq!(registry.expand_behavior("code"), Some(ExpandBehavior::None));
        assert_eq!(registry.expand_behavior("link"), Some(ExpandBehavior::None));
    }

    #[test]
    fn registry_custom_mark() {
        let mut registry = MarkRegistry::with_defaults();
        registry.register("highlight", ExpandBehavior::None);
        assert!(registry.is_registered("highlight"));
        assert_eq!(
            registry.expand_behavior("highlight"),
            Some(ExpandBehavior::None)
        );
    }

    #[test]
    fn registry_unregistered_returns_none() {
        let registry = MarkRegistry::with_defaults();
        assert!(!registry.is_registered("nonexistent"));
        assert_eq!(registry.expand_behavior("nonexistent"), None);
    }

    // ── markdown_to_marks tests ──────────────────────────────────────────

    #[test]
    fn plain_text_no_marks() {
        let result = markdown_to_marks("Hello, world!");
        assert_eq!(result.plain_text, "Hello, world!");
        assert!(result.marks.is_empty());
    }

    #[test]
    fn bold_text() {
        let result = markdown_to_marks("Hello **bold** world");
        assert_eq!(result.plain_text, "Hello bold world");
        assert_eq!(result.marks.len(), 1);
        assert_eq!(result.marks[0].key, "bold");
        assert_eq!(result.marks[0].start, 6);
        assert_eq!(result.marks[0].end, 10);
        assert_eq!(result.marks[0].value, JsonValue::Bool(true));
    }

    #[test]
    fn italic_text() {
        let result = markdown_to_marks("Hello *italic* world");
        assert_eq!(result.plain_text, "Hello italic world");
        assert_eq!(result.marks.len(), 1);
        assert_eq!(result.marks[0].key, "italic");
        assert_eq!(result.marks[0].start, 6);
        assert_eq!(result.marks[0].end, 12);
    }

    #[test]
    fn inline_code() {
        let result = markdown_to_marks("Use `code` here");
        assert_eq!(result.plain_text, "Use code here");
        assert_eq!(result.marks.len(), 1);
        assert_eq!(result.marks[0].key, "code");
        assert_eq!(result.marks[0].start, 4);
        assert_eq!(result.marks[0].end, 8);
    }

    #[test]
    fn strikethrough_text() {
        let result = markdown_to_marks("Hello ~~deleted~~ world");
        assert_eq!(result.plain_text, "Hello deleted world");
        assert_eq!(result.marks.len(), 1);
        assert_eq!(result.marks[0].key, "strikethrough");
        assert_eq!(result.marks[0].start, 6);
        assert_eq!(result.marks[0].end, 13);
    }

    #[test]
    fn link_text() {
        let result = markdown_to_marks("Click [here](https://example.com) now");
        assert_eq!(result.plain_text, "Click here now");
        assert_eq!(result.marks.len(), 1);
        assert_eq!(result.marks[0].key, "link");
        assert_eq!(result.marks[0].start, 6);
        assert_eq!(result.marks[0].end, 10);
        assert_eq!(
            result.marks[0].value,
            JsonValue::String("https://example.com".to_string())
        );
    }

    #[test]
    fn nested_bold_italic() {
        let result = markdown_to_marks("Hello ***bold italic*** world");
        assert_eq!(result.plain_text, "Hello bold italic world");
        assert_eq!(result.marks.len(), 2);
        // Should have both bold and italic marks on the same range
        let keys: Vec<&str> = result.marks.iter().map(|m| m.key.as_str()).collect();
        assert!(keys.contains(&"bold"));
        assert!(keys.contains(&"italic"));
    }

    #[test]
    fn bold_inside_text() {
        let result = markdown_to_marks("The hero walked **boldly** into the sunset.");
        assert_eq!(result.plain_text, "The hero walked boldly into the sunset.");
        assert_eq!(result.marks.len(), 1);
        assert_eq!(result.marks[0].key, "bold");
        assert_eq!(result.marks[0].start, 16);
        assert_eq!(result.marks[0].end, 22);
    }

    #[test]
    fn multiple_inline_marks() {
        let result = markdown_to_marks("**bold** and *italic* text");
        assert_eq!(result.plain_text, "bold and italic text");
        assert_eq!(result.marks.len(), 2);

        let bold = result.marks.iter().find(|m| m.key == "bold").unwrap();
        assert_eq!(bold.start, 0);
        assert_eq!(bold.end, 4);

        let italic = result.marks.iter().find(|m| m.key == "italic").unwrap();
        assert_eq!(italic.start, 9);
        assert_eq!(italic.end, 15);
    }

    #[test]
    fn paragraphs_separated_by_newlines() {
        let result = markdown_to_marks("First paragraph.\n\nSecond paragraph.");
        assert_eq!(result.plain_text, "First paragraph.\n\nSecond paragraph.");
        assert!(result.marks.is_empty());
    }

    #[test]
    fn paragraph_with_formatting() {
        let result = markdown_to_marks("First **bold** paragraph.\n\nSecond *italic* paragraph.");
        assert!(result.plain_text.contains("First bold paragraph."));
        assert!(result.plain_text.contains("Second italic paragraph."));
        assert_eq!(result.marks.len(), 2);
    }

    // ── marks_to_markdown tests ──────────────────────────────────────────

    #[test]
    fn no_marks_returns_plain_text() {
        let marked = MarkedText {
            plain_text: "Hello world".to_string(),
            marks: vec![],
        };
        assert_eq!(marks_to_markdown(&marked), "Hello world");
    }

    #[test]
    fn bold_mark_to_markdown() {
        let marked = MarkedText {
            plain_text: "Hello bold world".to_string(),
            marks: vec![MarkSpan {
                start: 6,
                end: 10,
                key: "bold".into(),
                value: JsonValue::Bool(true),
            }],
        };
        assert_eq!(marks_to_markdown(&marked), "Hello **bold** world");
    }

    #[test]
    fn italic_mark_to_markdown() {
        let marked = MarkedText {
            plain_text: "Hello italic world".to_string(),
            marks: vec![MarkSpan {
                start: 6,
                end: 12,
                key: "italic".into(),
                value: JsonValue::Bool(true),
            }],
        };
        assert_eq!(marks_to_markdown(&marked), "Hello *italic* world");
    }

    #[test]
    fn code_mark_to_markdown() {
        let marked = MarkedText {
            plain_text: "Use code here".to_string(),
            marks: vec![MarkSpan {
                start: 4,
                end: 8,
                key: "code".into(),
                value: JsonValue::Bool(true),
            }],
        };
        assert_eq!(marks_to_markdown(&marked), "Use `code` here");
    }

    #[test]
    fn link_mark_to_markdown() {
        let marked = MarkedText {
            plain_text: "Click here now".to_string(),
            marks: vec![MarkSpan {
                start: 6,
                end: 10,
                key: "link".into(),
                value: JsonValue::String("https://example.com".to_string()),
            }],
        };
        assert_eq!(
            marks_to_markdown(&marked),
            "Click [here](https://example.com) now"
        );
    }

    #[test]
    fn strikethrough_mark_to_markdown() {
        let marked = MarkedText {
            plain_text: "Hello deleted world".to_string(),
            marks: vec![MarkSpan {
                start: 6,
                end: 13,
                key: "strikethrough".into(),
                value: JsonValue::Bool(true),
            }],
        };
        assert_eq!(marks_to_markdown(&marked), "Hello ~~deleted~~ world");
    }

    #[test]
    fn multiple_marks_to_markdown() {
        let marked = MarkedText {
            plain_text: "bold and italic text".to_string(),
            marks: vec![
                MarkSpan {
                    start: 0,
                    end: 4,
                    key: "bold".into(),
                    value: JsonValue::Bool(true),
                },
                MarkSpan {
                    start: 9,
                    end: 15,
                    key: "italic".into(),
                    value: JsonValue::Bool(true),
                },
            ],
        };
        assert_eq!(marks_to_markdown(&marked), "**bold** and *italic* text");
    }

    // ── Round-trip tests ─────────────────────────────────────────────────

    #[test]
    fn roundtrip_bold() {
        let original = "Hello **bold** world";
        let marked = markdown_to_marks(original);
        let result = marks_to_markdown(&marked);
        assert_eq!(result, original);
    }

    #[test]
    fn roundtrip_italic() {
        let original = "Hello *italic* world";
        let marked = markdown_to_marks(original);
        let result = marks_to_markdown(&marked);
        assert_eq!(result, original);
    }

    #[test]
    fn roundtrip_code() {
        let original = "Use `code` here";
        let marked = markdown_to_marks(original);
        let result = marks_to_markdown(&marked);
        assert_eq!(result, original);
    }

    #[test]
    fn roundtrip_strikethrough() {
        let original = "Hello ~~deleted~~ world";
        let marked = markdown_to_marks(original);
        let result = marks_to_markdown(&marked);
        assert_eq!(result, original);
    }

    #[test]
    fn roundtrip_link() {
        let original = "Click [here](https://example.com) now";
        let marked = markdown_to_marks(original);
        let result = marks_to_markdown(&marked);
        assert_eq!(result, original);
    }

    #[test]
    fn roundtrip_multiple_marks() {
        let original = "**bold** and *italic* and `code` text";
        let marked = markdown_to_marks(original);
        let result = marks_to_markdown(&marked);
        assert_eq!(result, original);
    }

    #[test]
    fn roundtrip_plain_text() {
        let original = "Just plain text without any formatting.";
        let marked = markdown_to_marks(original);
        let result = marks_to_markdown(&marked);
        assert_eq!(result, original);
    }
}
