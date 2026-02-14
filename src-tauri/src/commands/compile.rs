use crate::error::AppError;
use crate::models::compile::{
    ChapterHeaderStyle, ChapterSeparator, CompileConfig, CompileOutput, OutputFormat,
};
use crate::models::manuscript::ChapterFrontmatter;
use crate::services::frontmatter;

use std::path::PathBuf;

/// Helper: path to manuscript directory.
fn manuscript_dir(project_path: &str) -> PathBuf {
    PathBuf::from(project_path).join("manuscript")
}

/// Helper: path to manuscript config YAML.
fn config_path(project_path: &str) -> PathBuf {
    manuscript_dir(project_path).join("manuscript.yaml")
}

/// Helper: path to a chapter Markdown file.
fn chapter_path(project_path: &str, slug: &str) -> PathBuf {
    manuscript_dir(project_path).join(format!("{}.md", slug))
}

/// Count words by splitting on whitespace and counting non-empty tokens.
fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Embedded CSS stylesheet for HTML export with print-ready formatting.
const HTML_STYLESHEET: &str = r#"
    /* Base typography */
    body {
        font-family: Georgia, 'Times New Roman', 'Noto Serif', serif;
        font-size: 16px;
        line-height: 1.8;
        color: #1a1a1a;
        max-width: 720px;
        margin: 2em auto;
        padding: 0 1em;
    }

    /* Paragraphs */
    p {
        margin: 0 0 1em 0;
        text-indent: 1.5em;
    }

    /* Title page */
    .title-page {
        text-align: center;
        padding: 4em 0 2em 0;
        page-break-after: always;
    }
    .title-page h1 {
        font-size: 2.4em;
        margin-bottom: 0.5em;
        text-indent: 0;
    }
    .title-page .author {
        font-size: 1.3em;
        font-style: italic;
        margin-top: 1em;
    }

    /* Chapter headings */
    .chapter {
        page-break-before: always;
    }
    .chapter:first-of-type {
        page-break-before: auto;
    }
    h2 {
        font-size: 1.6em;
        margin: 2em 0 1em 0;
        text-align: center;
        text-indent: 0;
    }

    /* Separators */
    hr {
        border: none;
        text-align: center;
        margin: 2em 0;
    }
    hr::after {
        content: '* * *';
        font-size: 1em;
        letter-spacing: 0.5em;
    }

    /* Synopsis / emphasis */
    em {
        font-style: italic;
    }

    /* Front matter */
    .front-matter {
        margin-bottom: 2em;
        page-break-after: always;
    }

    /* Block quotes */
    blockquote {
        margin: 1.5em 2em;
        padding-left: 1em;
        border-left: 3px solid #ccc;
        font-style: italic;
    }

    /* Print styles */
    @media print {
        body {
            font-size: 12pt;
            line-height: 1.6;
            max-width: none;
            margin: 0;
            padding: 0;
        }
        .title-page {
            padding: 6em 0 3em 0;
        }
        hr {
            page-break-after: avoid;
        }
    }
"#;

/// Convert a compiled Markdown document to a full HTML document with embedded styles.
///
/// Uses `pulldown-cmark` for Markdown-to-HTML conversion, then wraps the result
/// in a complete HTML document with DOCTYPE, head (including the embedded CSS), and body.
fn render_html(markdown: &str, title: &str) -> String {
    use pulldown_cmark::{html, Options, Parser};

    let options = Options::ENABLE_STRIKETHROUGH | Options::ENABLE_SMART_PUNCTUATION;
    let parser = Parser::new_ext(markdown, options);

    let mut html_body = String::new();
    html::push_html(&mut html_body, parser);

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>{stylesheet}</style>
</head>
<body>
{body}
</body>
</html>"#,
        title = html_escape(title),
        stylesheet = HTML_STYLESHEET,
        body = html_body.trim(),
    )
}

/// Escape special HTML characters in a string for safe embedding in HTML attributes/content.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Convert a compiled Markdown document to plain text.
///
/// Uses `pulldown-cmark` to parse the Markdown AST, then walks the events to produce
/// clean plain text with:
/// - Headers converted to UPPERCASE with underline characters
/// - Bold/italic markers stripped
/// - Links reduced to their display text
/// - Separators rendered according to the configured chapter separator style
/// - Title page text centered within 72 columns
fn render_plain_text(markdown: &str, separator: &ChapterSeparator) -> String {
    use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

    let options = Options::ENABLE_STRIKETHROUGH | Options::ENABLE_SMART_PUNCTUATION;
    let parser = Parser::new_ext(markdown, options);

    let mut output = String::new();
    let mut in_heading = false;
    let mut heading_level: Option<HeadingLevel> = None;
    let mut heading_text = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                in_heading = true;
                heading_level = Some(level);
                heading_text.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                in_heading = false;
                let upper = heading_text.trim().to_uppercase();
                let underline_char = match heading_level {
                    Some(HeadingLevel::H1) => '=',
                    _ => '-',
                };
                let underline =
                    std::iter::repeat_n(underline_char, upper.len()).collect::<String>();
                output.push_str(&upper);
                output.push('\n');
                output.push_str(&underline);
                output.push('\n');
                heading_level = None;
            }
            Event::Start(Tag::Emphasis | Tag::Strong | Tag::Strikethrough) => {
                // Strip formatting markers — just continue collecting text
            }
            Event::End(TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough) => {
                // Nothing to do
            }
            Event::Start(Tag::Link { .. }) => {
                // Just collect the link text, ignore the URL
            }
            Event::End(TagEnd::Link) => {
                // Nothing to do
            }
            Event::Start(Tag::Image { .. }) => {
                // Skip images entirely
            }
            Event::End(TagEnd::Image) => {}
            Event::Start(Tag::Paragraph) => {
                // Nothing special at paragraph start
            }
            Event::End(TagEnd::Paragraph) => {
                output.push_str("\n\n");
            }
            Event::Start(Tag::BlockQuote(_)) => {
                // Blockquotes: we'll just output the text without > markers
            }
            Event::End(TagEnd::BlockQuote(_)) => {}
            Event::Start(Tag::List(_)) => {}
            Event::End(TagEnd::List(_)) => {
                output.push('\n');
            }
            Event::Start(Tag::Item) => {
                output.push_str("- ");
            }
            Event::End(TagEnd::Item) => {
                output.push('\n');
            }
            Event::Start(Tag::CodeBlock(_)) => {}
            Event::End(TagEnd::CodeBlock) => {
                output.push('\n');
            }
            Event::Text(text) => {
                if in_heading {
                    heading_text.push_str(&text);
                } else {
                    output.push_str(&text);
                }
            }
            Event::Code(text) => {
                if in_heading {
                    heading_text.push_str(&text);
                } else {
                    output.push_str(&text);
                }
            }
            Event::SoftBreak => {
                if in_heading {
                    heading_text.push(' ');
                } else {
                    output.push('\n');
                }
            }
            Event::HardBreak => {
                if in_heading {
                    heading_text.push(' ');
                } else {
                    output.push('\n');
                }
            }
            Event::Rule => {
                let rule_text = match separator {
                    ChapterSeparator::ThreeStars => "* * *".to_string(),
                    ChapterSeparator::PageBreak => "=".repeat(40),
                    ChapterSeparator::HorizontalRule => "-".repeat(40),
                    ChapterSeparator::BlankLines => String::new(),
                };
                output.push_str(&rule_text);
                output.push_str("\n\n");
            }
            _ => {}
        }
    }

    output.trim_end().to_string()
}

/// Format the separator string for a given ChapterSeparator variant.
fn separator_string(sep: &ChapterSeparator) -> &'static str {
    match sep {
        ChapterSeparator::PageBreak => "\n\n---\n\n",
        ChapterSeparator::ThreeStars => "\n\n* * *\n\n",
        ChapterSeparator::HorizontalRule => "\n\n---\n\n",
        ChapterSeparator::BlankLines => "\n\n\n\n",
    }
}

/// Generate a chapter header line based on the style, chapter number, and title.
fn chapter_header(style: &ChapterHeaderStyle, number: usize, title: &str) -> Option<String> {
    match style {
        ChapterHeaderStyle::Numbered => Some(format!("## Chapter {}", number)),
        ChapterHeaderStyle::Titled => Some(format!("## {}", title)),
        ChapterHeaderStyle::NumberedAndTitled => Some(format!("## Chapter {}: {}", number, title)),
        ChapterHeaderStyle::None => None,
    }
}

/// Compile the full manuscript into a single document string.
///
/// Pipeline:
/// 1. Read ManuscriptConfig to get ordered chapter slugs
/// 2. Load each chapter (frontmatter + body)
/// 3. Build the compiled document with front matter, title page, chapter headers,
///    synopses, bodies, and separators
/// 4. Count words and chapters
/// 5. Return CompileOutput
#[tauri::command]
pub fn compile_manuscript(
    project_path: String,
    config: CompileConfig,
) -> Result<CompileOutput, AppError> {
    use crate::models::manuscript::ManuscriptConfig;
    use crate::services::yaml_service::read_yaml;

    // 1. Read manuscript config
    let manuscript_config: ManuscriptConfig = {
        let path = config_path(&project_path);
        if !path.exists() {
            ManuscriptConfig { chapters: vec![] }
        } else {
            read_yaml(&path)?
        }
    };

    let slugs = &manuscript_config.chapters;

    // Early return for empty manuscript
    if slugs.is_empty() {
        return Ok(CompileOutput {
            content: String::new(),
            format: config.output_format,
            chapter_count: 0,
            word_count: 0,
        });
    }

    let mut output = String::new();

    // 3a. Front matter
    if !config.front_matter.is_empty() {
        output.push_str(&config.front_matter);
        output.push_str(separator_string(&config.chapter_separator));
    }

    // 3b. Title page
    if config.include_title_page {
        output.push_str(&format!("# {}\n\n", config.title));
        output.push_str(&format!("**{}**", config.author));
        output.push_str(separator_string(&config.chapter_separator));
    }

    // 2. Load each chapter, skip missing ones gracefully
    let mut chapter_count: usize = 0;
    let mut chapter_number: usize = 0;

    for (i, slug) in slugs.iter().enumerate() {
        let path = chapter_path(&project_path, slug);

        if !path.exists() {
            eprintln!("Warning: chapter file not found, skipping: {}", slug);
            continue;
        }

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: failed to read chapter {}: {}", slug, e);
                continue;
            }
        };

        let doc: frontmatter::ParsedDocument<ChapterFrontmatter> =
            match frontmatter::parse(&content) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Warning: failed to parse chapter {}: {}", slug, e);
                    continue;
                }
            };

        // Insert separator BETWEEN chapters (not before the first one)
        if chapter_count > 0 {
            output.push_str(separator_string(&config.chapter_separator));
        }

        chapter_number += 1;
        chapter_count += 1;

        // Chapter header
        if let Some(header) = chapter_header(
            &config.chapter_header_style,
            chapter_number,
            &doc.frontmatter.title,
        ) {
            output.push_str(&header);
            output.push('\n');
            // Check if there is a synopsis or body to add after the header
            let has_synopsis = config.include_synopsis
                && doc
                    .frontmatter
                    .synopsis
                    .as_ref()
                    .is_some_and(|s| !s.is_empty());
            let has_body = !doc.body.is_empty();
            if has_synopsis || has_body {
                output.push('\n');
            }
        }

        // Synopsis
        if config.include_synopsis {
            if let Some(ref synopsis) = doc.frontmatter.synopsis {
                if !synopsis.is_empty() {
                    output.push_str(&format!("*{}*", synopsis));
                    output.push('\n');
                    if !doc.body.is_empty() {
                        output.push('\n');
                    }
                }
            }
        }

        // Body
        if !doc.body.is_empty() {
            output.push_str(&doc.body);
            // Ensure no trailing newline duplication - body may already end with newline
            if !doc.body.ends_with('\n') {
                // don't add; the body as-is is fine
            }
        }

        // Remove trailing whitespace from the last chapter's contribution
        // We'll trim the whole output at the end
        let _ = i; // suppress unused variable warning
    }

    // Trim trailing whitespace from the entire output
    let content = output.trim_end().to_string();

    let word_count = count_words(&content);

    // Post-process: convert Markdown to the requested output format
    let final_content = match config.output_format {
        OutputFormat::Html => render_html(&content, &config.title),
        OutputFormat::PlainText => render_plain_text(&content, &config.chapter_separator),
        OutputFormat::Markdown => content,
    };

    Ok(CompileOutput {
        content: final_content,
        format: config.output_format,
        chapter_count,
        word_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::compile::{
        ChapterHeaderStyle, ChapterSeparator, CompileConfig, OutputFormat,
    };
    use crate::models::manuscript::{ChapterStatus, ManuscriptConfig};
    use crate::services::yaml_service::write_yaml;
    use crate::test_helpers::setup_test_dir;

    /// Helper: create a chapter file with frontmatter and body.
    fn write_chapter(
        project_path: &str,
        slug: &str,
        title: &str,
        synopsis: Option<&str>,
        body: &str,
    ) {
        use crate::models::manuscript::ChapterFrontmatter;
        use crate::services::frontmatter::serialize;

        let dir = manuscript_dir(project_path);
        std::fs::create_dir_all(&dir).unwrap();

        let fm = ChapterFrontmatter {
            title: title.to_string(),
            slug: slug.to_string(),
            status: ChapterStatus::Draft,
            pov: None,
            synopsis: synopsis.map(|s| s.to_string()),
            target_words: None,
            order: 0,
        };

        let content = serialize(&fm, body).unwrap();
        let path = chapter_path(project_path, slug);
        std::fs::write(&path, content).unwrap();
    }

    /// Helper: write manuscript config with ordered slugs.
    fn write_config(project_path: &str, slugs: &[&str]) {
        let config = ManuscriptConfig {
            chapters: slugs.iter().map(|s| s.to_string()).collect(),
        };
        let path = config_path(project_path);
        write_yaml(&path, &config).unwrap();
    }

    /// Helper: create a default compile config for tests.
    fn default_config() -> CompileConfig {
        CompileConfig {
            title: "My Novel".to_string(),
            author: "Jane Author".to_string(),
            include_title_page: false,
            chapter_header_style: ChapterHeaderStyle::Titled,
            chapter_separator: ChapterSeparator::ThreeStars,
            output_format: OutputFormat::Markdown,
            include_synopsis: false,
            front_matter: String::new(),
        }
    }

    // ── Empty manuscript ────────────────────────────────────────────

    #[test]
    fn empty_manuscript_no_config_file() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let result = compile_manuscript(pp, default_config()).unwrap();
        assert_eq!(result.content, "");
        assert_eq!(result.chapter_count, 0);
        assert_eq!(result.word_count, 0);
    }

    #[test]
    fn empty_manuscript_empty_chapters_list() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();
        write_config(&pp, &[]);

        let result = compile_manuscript(pp, default_config()).unwrap();
        assert_eq!(result.content, "");
        assert_eq!(result.chapter_count, 0);
        assert_eq!(result.word_count, 0);
    }

    // ── Single chapter ──────────────────────────────────────────────

    #[test]
    fn single_chapter_titled_header() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["chapter-one"]);
        write_chapter(
            &pp,
            "chapter-one",
            "The Beginning",
            None,
            "Once upon a time.",
        );

        let result = compile_manuscript(pp, default_config()).unwrap();
        assert!(result.content.contains("## The Beginning"));
        assert!(result.content.contains("Once upon a time."));
        assert_eq!(result.chapter_count, 1);
        assert!(result.word_count > 0);
    }

    #[test]
    fn single_chapter_numbered_header() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["chapter-one"]);
        write_chapter(&pp, "chapter-one", "The Beginning", None, "Hello world.");

        let mut config = default_config();
        config.chapter_header_style = ChapterHeaderStyle::Numbered;

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result.content.contains("## Chapter 1"));
        assert!(!result.content.contains("The Beginning"));
    }

    #[test]
    fn single_chapter_numbered_and_titled_header() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["chapter-one"]);
        write_chapter(&pp, "chapter-one", "The Beginning", None, "Hello world.");

        let mut config = default_config();
        config.chapter_header_style = ChapterHeaderStyle::NumberedAndTitled;

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result.content.contains("## Chapter 1: The Beginning"));
    }

    #[test]
    fn single_chapter_no_header() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["chapter-one"]);
        write_chapter(&pp, "chapter-one", "The Beginning", None, "Hello world.");

        let mut config = default_config();
        config.chapter_header_style = ChapterHeaderStyle::None;

        let result = compile_manuscript(pp, config).unwrap();
        assert!(!result.content.contains("##"));
        assert!(result.content.contains("Hello world."));
    }

    // ── Multiple chapters with separators ──────────────────────────

    #[test]
    fn two_chapters_three_stars_separator() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1", "ch-2"]);
        write_chapter(&pp, "ch-1", "One", None, "First chapter body.");
        write_chapter(&pp, "ch-2", "Two", None, "Second chapter body.");

        let mut config = default_config();
        config.chapter_separator = ChapterSeparator::ThreeStars;

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result.content.contains("* * *"));
        assert_eq!(result.chapter_count, 2);
    }

    #[test]
    fn two_chapters_page_break_separator() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1", "ch-2"]);
        write_chapter(&pp, "ch-1", "One", None, "First.");
        write_chapter(&pp, "ch-2", "Two", None, "Second.");

        let mut config = default_config();
        config.chapter_separator = ChapterSeparator::PageBreak;

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result.content.contains("---"));
        assert_eq!(result.chapter_count, 2);
    }

    #[test]
    fn two_chapters_horizontal_rule_separator() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1", "ch-2"]);
        write_chapter(&pp, "ch-1", "One", None, "First.");
        write_chapter(&pp, "ch-2", "Two", None, "Second.");

        let mut config = default_config();
        config.chapter_separator = ChapterSeparator::HorizontalRule;

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result.content.contains("---"));
    }

    #[test]
    fn two_chapters_blank_lines_separator() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1", "ch-2"]);
        write_chapter(&pp, "ch-1", "One", None, "First.");
        write_chapter(&pp, "ch-2", "Two", None, "Second.");

        let mut config = default_config();
        config.chapter_separator = ChapterSeparator::BlankLines;

        let result = compile_manuscript(pp, config).unwrap();
        // BlankLines separator is \n\n\n\n — four newlines
        // The content between the two chapters should contain at least 3 consecutive newlines
        assert!(result.content.contains("\n\n\n"));
    }

    #[test]
    fn no_separator_after_last_chapter() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1", "ch-2"]);
        write_chapter(&pp, "ch-1", "One", None, "First.");
        write_chapter(&pp, "ch-2", "Two", None, "Second.");

        let result = compile_manuscript(pp, default_config()).unwrap();
        // Content should end with chapter body, not a separator
        assert!(result.content.trim_end().ends_with("Second."));
    }

    // ── Title page ─────────────────────────────────────────────────

    #[test]
    fn title_page_included() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "One", None, "Body.");

        let mut config = default_config();
        config.include_title_page = true;
        config.title = "My Great Novel".to_string();
        config.author = "John Smith".to_string();

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result.content.contains("# My Great Novel"));
        assert!(result.content.contains("**John Smith**"));
        // Title page should come before chapter content
        let title_pos = result.content.find("# My Great Novel").unwrap();
        let body_pos = result.content.find("Body.").unwrap();
        assert!(title_pos < body_pos);
    }

    #[test]
    fn title_page_not_included() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "One", None, "Body.");

        let config = default_config(); // include_title_page is false by default

        let result = compile_manuscript(pp, config).unwrap();
        assert!(!result.content.contains("# My Novel"));
        assert!(!result.content.contains("**Jane Author**"));
    }

    // ── Front matter ───────────────────────────────────────────────

    #[test]
    fn front_matter_prepended() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "One", None, "Body.");

        let mut config = default_config();
        config.front_matter = "This is the dedication.\n\nFor my family.".to_string();

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result.content.starts_with("This is the dedication."));
        let fm_pos = result.content.find("This is the dedication.").unwrap();
        let body_pos = result.content.find("Body.").unwrap();
        assert!(fm_pos < body_pos);
    }

    #[test]
    fn empty_front_matter_not_prepended() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "One", None, "Body.");

        let config = default_config(); // front_matter is empty by default

        let result = compile_manuscript(pp, config).unwrap();
        // Should start with the chapter header, not a separator
        assert!(result.content.starts_with("## One"));
    }

    // ── Synopsis ───────────────────────────────────────────────────

    #[test]
    fn synopsis_included_when_enabled() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(
            &pp,
            "ch-1",
            "One",
            Some("A chapter about beginnings"),
            "Body text.",
        );

        let mut config = default_config();
        config.include_synopsis = true;

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result.content.contains("*A chapter about beginnings*"));
        // Synopsis should come after header but before body
        let synopsis_pos = result.content.find("*A chapter about beginnings*").unwrap();
        let body_pos = result.content.find("Body text.").unwrap();
        assert!(synopsis_pos < body_pos);
    }

    #[test]
    fn synopsis_not_included_when_disabled() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "One", Some("Secret synopsis"), "Body text.");

        let config = default_config(); // include_synopsis is false by default

        let result = compile_manuscript(pp, config).unwrap();
        assert!(!result.content.contains("Secret synopsis"));
    }

    #[test]
    fn synopsis_not_included_when_chapter_has_none() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "One", None, "Body text.");

        let mut config = default_config();
        config.include_synopsis = true;

        let result = compile_manuscript(pp, config).unwrap();
        // No italicized synopsis marker
        assert!(!result.content.contains("*\n"));
    }

    // ── Missing chapters (graceful skip) ───────────────────────────

    #[test]
    fn missing_chapter_file_skipped() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1", "missing-chapter", "ch-3"]);
        write_chapter(&pp, "ch-1", "One", None, "First.");
        // Deliberately not creating "missing-chapter"
        write_chapter(&pp, "ch-3", "Three", None, "Third.");

        let result = compile_manuscript(pp, default_config()).unwrap();
        assert!(result.content.contains("First."));
        assert!(result.content.contains("Third."));
        assert_eq!(result.chapter_count, 2); // Only the two that exist
    }

    // ── Chapter with empty body ────────────────────────────────────

    #[test]
    fn chapter_with_empty_body() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "Empty Chapter", None, "");

        let result = compile_manuscript(pp, default_config()).unwrap();
        assert!(result.content.contains("## Empty Chapter"));
        assert_eq!(result.chapter_count, 1);
    }

    // ── Word count ─────────────────────────────────────────────────

    #[test]
    fn word_count_accurate() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "One", None, "One two three four five.");

        let mut config = default_config();
        config.chapter_header_style = ChapterHeaderStyle::None;

        let result = compile_manuscript(pp, config).unwrap();
        // Body has 5 words: "One two three four five."
        assert_eq!(result.word_count, 5);
    }

    #[test]
    fn word_count_includes_header_and_title_page() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "Intro", None, "Hello world.");

        let mut config = default_config();
        config.include_title_page = true;
        config.chapter_header_style = ChapterHeaderStyle::Titled;

        let result = compile_manuscript(pp, config).unwrap();
        // Title page: "# My Novel" (2) + "**Jane Author**" (2, asterisks around count as word chars)
        // Chapter header: "## Intro" (1)
        // Body: "Hello world." (2)
        // Separator between title page and chapter: "* * *" (1 each = 3)
        // Total varies by exact formatting, but should be > 2
        assert!(result.word_count > 2);
    }

    // ── Output format passthrough ──────────────────────────────────

    #[test]
    fn output_format_markdown() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();
        write_config(&pp, &[]);

        let mut config = default_config();
        config.output_format = OutputFormat::Markdown;

        let result = compile_manuscript(pp, config).unwrap();
        assert_eq!(result.format, OutputFormat::Markdown);
    }

    #[test]
    fn output_format_html() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();
        write_config(&pp, &[]);

        let mut config = default_config();
        config.output_format = OutputFormat::Html;

        let result = compile_manuscript(pp, config).unwrap();
        assert_eq!(result.format, OutputFormat::Html);
    }

    #[test]
    fn output_format_plaintext() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();
        write_config(&pp, &[]);

        let mut config = default_config();
        config.output_format = OutputFormat::PlainText;

        let result = compile_manuscript(pp, config).unwrap();
        assert_eq!(result.format, OutputFormat::PlainText);
    }

    // ── Full integration: front matter + title page + multiple chapters + synopses ──

    #[test]
    fn full_compilation_with_all_features() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["prologue", "ch-1", "ch-2"]);
        write_chapter(
            &pp,
            "prologue",
            "Prologue",
            Some("The world before"),
            "In the beginning...",
        );
        write_chapter(
            &pp,
            "ch-1",
            "The Journey",
            Some("Our hero departs"),
            "The hero set out at dawn.",
        );
        write_chapter(&pp, "ch-2", "The Return", None, "And so it ended.");

        let config = CompileConfig {
            title: "Epic Tale".to_string(),
            author: "A. Writer".to_string(),
            include_title_page: true,
            chapter_header_style: ChapterHeaderStyle::NumberedAndTitled,
            chapter_separator: ChapterSeparator::ThreeStars,
            output_format: OutputFormat::Markdown,
            include_synopsis: true,
            front_matter: "For those who dream.".to_string(),
        };

        let result = compile_manuscript(pp, config).unwrap();

        // Front matter first
        assert!(result.content.starts_with("For those who dream."));
        // Then title page
        assert!(result.content.contains("# Epic Tale"));
        assert!(result.content.contains("**A. Writer**"));
        // Chapter headers
        assert!(result.content.contains("## Chapter 1: Prologue"));
        assert!(result.content.contains("## Chapter 2: The Journey"));
        assert!(result.content.contains("## Chapter 3: The Return"));
        // Synopses (only for chapters that have them)
        assert!(result.content.contains("*The world before*"));
        assert!(result.content.contains("*Our hero departs*"));
        // Bodies
        assert!(result.content.contains("In the beginning..."));
        assert!(result.content.contains("The hero set out at dawn."));
        assert!(result.content.contains("And so it ended."));
        // Separators between chapters
        assert_eq!(result.content.matches("* * *").count(), 4); // fm->title, title->ch1, ch1->ch2, ch2->ch3
                                                                // Metadata
        assert_eq!(result.chapter_count, 3);
        assert_eq!(result.format, OutputFormat::Markdown);
        assert!(result.word_count > 0);
    }

    // ── Ordering ───────────────────────────────────────────────────

    #[test]
    fn chapters_compiled_in_config_order() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        // Config order: ch-2 first, then ch-1
        write_config(&pp, &["ch-2", "ch-1"]);
        write_chapter(&pp, "ch-1", "Alpha", None, "I am alpha.");
        write_chapter(&pp, "ch-2", "Beta", None, "I am beta.");

        let result = compile_manuscript(pp, default_config()).unwrap();
        let beta_pos = result.content.find("I am beta.").unwrap();
        let alpha_pos = result.content.find("I am alpha.").unwrap();
        assert!(
            beta_pos < alpha_pos,
            "ch-2 should come before ch-1 per config order"
        );
    }

    // ── Unit tests for helper functions ────────────────────────────

    #[test]
    fn test_count_words_basic() {
        assert_eq!(count_words("hello world"), 2);
    }

    #[test]
    fn test_count_words_empty() {
        assert_eq!(count_words(""), 0);
    }

    #[test]
    fn test_count_words_whitespace_only() {
        assert_eq!(count_words("   \n\t  "), 0);
    }

    #[test]
    fn test_count_words_multiple_spaces() {
        assert_eq!(count_words("one   two   three"), 3);
    }

    #[test]
    fn test_chapter_header_numbered() {
        assert_eq!(
            chapter_header(&ChapterHeaderStyle::Numbered, 5, "Ignored"),
            Some("## Chapter 5".to_string())
        );
    }

    #[test]
    fn test_chapter_header_titled() {
        assert_eq!(
            chapter_header(&ChapterHeaderStyle::Titled, 5, "My Title"),
            Some("## My Title".to_string())
        );
    }

    #[test]
    fn test_chapter_header_numbered_and_titled() {
        assert_eq!(
            chapter_header(&ChapterHeaderStyle::NumberedAndTitled, 3, "Dawn"),
            Some("## Chapter 3: Dawn".to_string())
        );
    }

    #[test]
    fn test_chapter_header_none() {
        assert_eq!(chapter_header(&ChapterHeaderStyle::None, 1, "Title"), None);
    }

    #[test]
    fn test_separator_string_values() {
        assert_eq!(
            separator_string(&ChapterSeparator::PageBreak),
            "\n\n---\n\n"
        );
        assert_eq!(
            separator_string(&ChapterSeparator::ThreeStars),
            "\n\n* * *\n\n"
        );
        assert_eq!(
            separator_string(&ChapterSeparator::HorizontalRule),
            "\n\n---\n\n"
        );
        assert_eq!(separator_string(&ChapterSeparator::BlankLines), "\n\n\n\n");
    }

    // ══════════════════════════════════════════════════════════════
    // ITEM-102: Comprehensive compilation tests
    // ══════════════════════════════════════════════════════════════

    // ── Default config compilation ────────────────────────────────

    #[test]
    fn compile_with_default_config_produces_expected_output() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1", "ch-2"]);
        write_chapter(&pp, "ch-1", "The Dawn", None, "Morning light.");
        write_chapter(&pp, "ch-2", "The Dusk", None, "Evening shadows.");

        let config = CompileConfig::default();
        let result = compile_manuscript(pp, config).unwrap();

        // Default config: include_title_page=true, NumberedAndTitled, PageBreak, Markdown
        // Title page has empty title/author by default
        assert!(result.content.contains("## Chapter 1: The Dawn"));
        assert!(result.content.contains("## Chapter 2: The Dusk"));
        assert!(result.content.contains("Morning light."));
        assert!(result.content.contains("Evening shadows."));
        assert_eq!(result.chapter_count, 2);
        assert_eq!(result.format, OutputFormat::Markdown);
        assert!(result.word_count > 0);
    }

    // ── Multi-chapter header style matrix ─────────────────────────

    #[test]
    fn multi_chapter_numbered_headers() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-a", "ch-b", "ch-c"]);
        write_chapter(&pp, "ch-a", "Alpha", None, "Body A.");
        write_chapter(&pp, "ch-b", "Beta", None, "Body B.");
        write_chapter(&pp, "ch-c", "Gamma", None, "Body C.");

        let mut config = default_config();
        config.chapter_header_style = ChapterHeaderStyle::Numbered;

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result.content.contains("## Chapter 1"));
        assert!(result.content.contains("## Chapter 2"));
        assert!(result.content.contains("## Chapter 3"));
        // Should NOT contain chapter titles in headers
        assert!(!result.content.contains("## Alpha"));
        assert!(!result.content.contains("## Beta"));
        assert!(!result.content.contains("## Gamma"));
        assert_eq!(result.chapter_count, 3);
    }

    #[test]
    fn multi_chapter_titled_headers() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-a", "ch-b", "ch-c"]);
        write_chapter(&pp, "ch-a", "Alpha", None, "Body A.");
        write_chapter(&pp, "ch-b", "Beta", None, "Body B.");
        write_chapter(&pp, "ch-c", "Gamma", None, "Body C.");

        let mut config = default_config();
        config.chapter_header_style = ChapterHeaderStyle::Titled;

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result.content.contains("## Alpha"));
        assert!(result.content.contains("## Beta"));
        assert!(result.content.contains("## Gamma"));
        // Should NOT contain numbered chapter markers
        assert!(!result.content.contains("Chapter 1"));
        assert!(!result.content.contains("Chapter 2"));
        assert!(!result.content.contains("Chapter 3"));
        assert_eq!(result.chapter_count, 3);
    }

    #[test]
    fn multi_chapter_numbered_and_titled_headers() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-a", "ch-b", "ch-c"]);
        write_chapter(&pp, "ch-a", "Alpha", None, "Body A.");
        write_chapter(&pp, "ch-b", "Beta", None, "Body B.");
        write_chapter(&pp, "ch-c", "Gamma", None, "Body C.");

        let mut config = default_config();
        config.chapter_header_style = ChapterHeaderStyle::NumberedAndTitled;

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result.content.contains("## Chapter 1: Alpha"));
        assert!(result.content.contains("## Chapter 2: Beta"));
        assert!(result.content.contains("## Chapter 3: Gamma"));
        assert_eq!(result.chapter_count, 3);
    }

    #[test]
    fn multi_chapter_no_headers() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-a", "ch-b", "ch-c"]);
        write_chapter(&pp, "ch-a", "Alpha", None, "Body A.");
        write_chapter(&pp, "ch-b", "Beta", None, "Body B.");
        write_chapter(&pp, "ch-c", "Gamma", None, "Body C.");

        let mut config = default_config();
        config.chapter_header_style = ChapterHeaderStyle::None;

        let result = compile_manuscript(pp, config).unwrap();
        // No ## markers at all
        assert!(!result.content.contains("##"));
        // But body content should still be present
        assert!(result.content.contains("Body A."));
        assert!(result.content.contains("Body B."));
        assert!(result.content.contains("Body C."));
        assert_eq!(result.chapter_count, 3);
    }

    // ── Multi-chapter separator matrix ────────────────────────────

    #[test]
    fn multi_chapter_three_stars_separator_count() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-a", "ch-b", "ch-c"]);
        write_chapter(&pp, "ch-a", "A", None, "Body A.");
        write_chapter(&pp, "ch-b", "B", None, "Body B.");
        write_chapter(&pp, "ch-c", "C", None, "Body C.");

        let mut config = default_config();
        config.chapter_separator = ChapterSeparator::ThreeStars;

        let result = compile_manuscript(pp, config).unwrap();
        // 3 chapters -> 2 separators between them
        assert_eq!(result.content.matches("* * *").count(), 2);
    }

    #[test]
    fn multi_chapter_page_break_separator_count() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-a", "ch-b", "ch-c"]);
        write_chapter(&pp, "ch-a", "A", None, "Body A.");
        write_chapter(&pp, "ch-b", "B", None, "Body B.");
        write_chapter(&pp, "ch-c", "C", None, "Body C.");

        let mut config = default_config();
        config.chapter_separator = ChapterSeparator::PageBreak;

        let result = compile_manuscript(pp, config).unwrap();
        // PageBreak uses "---", 3 chapters -> 2 separators
        assert_eq!(result.content.matches("---").count(), 2);
    }

    #[test]
    fn multi_chapter_horizontal_rule_separator_count() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-a", "ch-b", "ch-c"]);
        write_chapter(&pp, "ch-a", "A", None, "Body A.");
        write_chapter(&pp, "ch-b", "B", None, "Body B.");
        write_chapter(&pp, "ch-c", "C", None, "Body C.");

        let mut config = default_config();
        config.chapter_separator = ChapterSeparator::HorizontalRule;

        let result = compile_manuscript(pp, config).unwrap();
        // HorizontalRule also uses "---", 3 chapters -> 2 separators
        assert_eq!(result.content.matches("---").count(), 2);
    }

    #[test]
    fn multi_chapter_blank_lines_separator_placement() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-a", "ch-b", "ch-c"]);
        write_chapter(&pp, "ch-a", "A", None, "Body A.");
        write_chapter(&pp, "ch-b", "B", None, "Body B.");
        write_chapter(&pp, "ch-c", "C", None, "Body C.");

        let mut config = default_config();
        config.chapter_separator = ChapterSeparator::BlankLines;

        let result = compile_manuscript(pp, config).unwrap();
        // All body content is present
        assert!(result.content.contains("Body A."));
        assert!(result.content.contains("Body B."));
        assert!(result.content.contains("Body C."));
        assert_eq!(result.chapter_count, 3);
    }

    // ── Configuration matrix: title_page × synopsis ───────────────

    #[test]
    fn config_matrix_title_page_on_synopsis_on() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(
            &pp,
            "ch-1",
            "One",
            Some("First chapter synopsis"),
            "Body text.",
        );

        let mut config = default_config();
        config.include_title_page = true;
        config.include_synopsis = true;

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result.content.contains("# My Novel"));
        assert!(result.content.contains("**Jane Author**"));
        assert!(result.content.contains("*First chapter synopsis*"));
        assert!(result.content.contains("Body text."));
    }

    #[test]
    fn config_matrix_title_page_on_synopsis_off() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "One", Some("Hidden synopsis"), "Body text.");

        let mut config = default_config();
        config.include_title_page = true;
        config.include_synopsis = false;

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result.content.contains("# My Novel"));
        assert!(result.content.contains("**Jane Author**"));
        assert!(!result.content.contains("Hidden synopsis"));
        assert!(result.content.contains("Body text."));
    }

    #[test]
    fn config_matrix_title_page_off_synopsis_on() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "One", Some("Visible synopsis"), "Body text.");

        let mut config = default_config();
        config.include_title_page = false;
        config.include_synopsis = true;

        let result = compile_manuscript(pp, config).unwrap();
        assert!(!result.content.contains("# My Novel"));
        assert!(!result.content.contains("**Jane Author**"));
        assert!(result.content.contains("*Visible synopsis*"));
        assert!(result.content.contains("Body text."));
    }

    #[test]
    fn config_matrix_title_page_off_synopsis_off() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "One", Some("Invisible synopsis"), "Body text.");

        let config = default_config(); // both off by default in test helper

        let result = compile_manuscript(pp, config).unwrap();
        assert!(!result.content.contains("# My Novel"));
        assert!(!result.content.contains("**Jane Author**"));
        assert!(!result.content.contains("Invisible synopsis"));
        assert!(result.content.contains("Body text."));
    }

    // ── Special characters in chapter titles ──────────────────────

    #[test]
    fn chapter_title_with_double_quotes() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "The \"Great\" Escape", None, "Content here.");

        let result = compile_manuscript(pp, default_config()).unwrap();
        assert!(result.content.contains("The \"Great\" Escape"));
        assert_eq!(result.chapter_count, 1);
    }

    #[test]
    fn chapter_title_with_single_quotes() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "It's a New Day", None, "Content here.");

        let result = compile_manuscript(pp, default_config()).unwrap();
        assert!(result.content.contains("It's a New Day"));
        assert_eq!(result.chapter_count, 1);
    }

    #[test]
    fn chapter_title_with_ampersand() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "War & Peace", None, "Content here.");

        let result = compile_manuscript(pp, default_config()).unwrap();
        assert!(result.content.contains("War & Peace"));
        assert_eq!(result.chapter_count, 1);
    }

    #[test]
    fn chapter_title_with_angle_brackets() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "A <Bold> Move", None, "Content here.");

        let result = compile_manuscript(pp, default_config()).unwrap();
        assert!(result.content.contains("A <Bold> Move"));
        assert_eq!(result.chapter_count, 1);
    }

    #[test]
    fn chapter_title_with_unicode_japanese() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "第一章：始まり", None, "日本語のテキスト。");

        let result = compile_manuscript(pp, default_config()).unwrap();
        assert!(result.content.contains("第一章：始まり"));
        assert!(result.content.contains("日本語のテキスト。"));
        assert_eq!(result.chapter_count, 1);
    }

    #[test]
    fn chapter_title_with_unicode_emoji_and_accents() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "Café Résumé", None, "Après-midi content.");

        let result = compile_manuscript(pp, default_config()).unwrap();
        assert!(result.content.contains("Café Résumé"));
        assert!(result.content.contains("Après-midi content."));
        assert_eq!(result.chapter_count, 1);
    }

    #[test]
    fn chapter_title_with_mixed_special_characters() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(
            &pp,
            "ch-1",
            "\"Hello\" & <World> — Über Cool™",
            None,
            "Complex title test.",
        );

        let mut config = default_config();
        config.chapter_header_style = ChapterHeaderStyle::NumberedAndTitled;

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result
            .content
            .contains("## Chapter 1: \"Hello\" & <World> — Über Cool™"));
        assert_eq!(result.chapter_count, 1);
    }

    // ── Front matter with Markdown formatting ─────────────────────

    #[test]
    fn front_matter_with_markdown_formatting_preserved() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "One", None, "Body.");

        let mut config = default_config();
        config.front_matter =
            "## Dedication\n\n**For my family.**\n\n*With love and gratitude.*".to_string();

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result.content.contains("## Dedication"));
        assert!(result.content.contains("**For my family.**"));
        assert!(result.content.contains("*With love and gratitude.*"));
        // Front matter should come before chapter content
        let fm_pos = result.content.find("## Dedication").unwrap();
        let body_pos = result.content.find("Body.").unwrap();
        assert!(fm_pos < body_pos);
    }

    #[test]
    fn front_matter_with_list_formatting() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "One", None, "Body.");

        let mut config = default_config();
        config.front_matter =
            "## Acknowledgments\n\n- Editor: John\n- Agent: Jane\n- Family: Always".to_string();

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result.content.contains("- Editor: John"));
        assert!(result.content.contains("- Agent: Jane"));
        assert!(result.content.contains("- Family: Always"));
    }

    // ── Very long chapter body (performance) ──────────────────────

    #[test]
    fn very_long_chapter_body_does_not_panic() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        // Generate a large body: ~100,000 words
        let long_body: String = (0..100_000)
            .map(|i| format!("word{}", i))
            .collect::<Vec<_>>()
            .join(" ");

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "The Long Chapter", None, &long_body);

        let mut config = default_config();
        config.chapter_header_style = ChapterHeaderStyle::None;

        let result = compile_manuscript(pp, config).unwrap();
        assert_eq!(result.chapter_count, 1);
        assert_eq!(result.word_count, 100_000);
        assert!(result.content.contains("word0"));
        assert!(result.content.contains("word99999"));
    }

    // ================================================================
    // Plain text rendering tests
    // ================================================================

    #[test]
    fn test_render_plain_text_strips_bold() {
        let md = "This is **bold** text.";
        let result = render_plain_text(md, &ChapterSeparator::ThreeStars);
        assert_eq!(result, "This is bold text.");
    }

    #[test]
    fn test_render_plain_text_strips_italic() {
        let md = "This is *italic* text.";
        let result = render_plain_text(md, &ChapterSeparator::ThreeStars);
        assert_eq!(result, "This is italic text.");
    }

    #[test]
    fn test_render_plain_text_strips_bold_and_italic() {
        let md = "Mix of **bold** and *italic* and ***both***.";
        let result = render_plain_text(md, &ChapterSeparator::ThreeStars);
        assert_eq!(result, "Mix of bold and italic and both.");
    }

    #[test]
    fn test_render_plain_text_h1_uppercase_with_equals() {
        let md = "# My Great Novel";
        let result = render_plain_text(md, &ChapterSeparator::ThreeStars);
        assert_eq!(result, "MY GREAT NOVEL\n==============");
    }

    #[test]
    fn test_render_plain_text_h2_uppercase_with_dashes() {
        let md = "## Chapter 1: The Beginning";
        let result = render_plain_text(md, &ChapterSeparator::ThreeStars);
        assert_eq!(result, "CHAPTER 1: THE BEGINNING\n------------------------");
    }

    #[test]
    fn test_render_plain_text_h3_uppercase_with_dashes() {
        let md = "### Subsection";
        let result = render_plain_text(md, &ChapterSeparator::ThreeStars);
        assert_eq!(result, "SUBSECTION\n----------");
    }

    #[test]
    fn test_render_plain_text_separator_three_stars() {
        let md = "Before\n\n* * *\n\nAfter";
        let result = render_plain_text(md, &ChapterSeparator::ThreeStars);
        assert!(result.contains("* * *"));
        assert!(result.starts_with("Before"));
        assert!(result.ends_with("After"));
    }

    #[test]
    fn test_render_plain_text_separator_page_break() {
        let md = "Before\n\n---\n\nAfter";
        let result = render_plain_text(md, &ChapterSeparator::PageBreak);
        assert!(result.contains(&"=".repeat(40)));
        assert!(!result.contains("---"));
    }

    #[test]
    fn test_render_plain_text_separator_horizontal_rule() {
        let md = "Before\n\n---\n\nAfter";
        let result = render_plain_text(md, &ChapterSeparator::HorizontalRule);
        assert!(result.contains(&"-".repeat(40)));
    }

    #[test]
    fn test_render_plain_text_separator_blank_lines() {
        let md = "Before\n\n---\n\nAfter";
        let result = render_plain_text(md, &ChapterSeparator::BlankLines);
        // Should not have dashes or equals, just whitespace between
        assert!(!result.contains(&"-".repeat(40)));
        assert!(!result.contains(&"=".repeat(40)));
        assert!(result.contains("Before"));
        assert!(result.contains("After"));
    }

    #[test]
    fn test_render_plain_text_strips_links() {
        let md = "Click [here](https://example.com) for more.";
        let result = render_plain_text(md, &ChapterSeparator::ThreeStars);
        assert_eq!(result, "Click here for more.");
    }

    #[test]
    fn test_render_plain_text_strips_strikethrough() {
        let md = "This is ~~deleted~~ text.";
        let result = render_plain_text(md, &ChapterSeparator::ThreeStars);
        assert_eq!(result, "This is deleted text.");
    }

    #[test]
    fn test_render_plain_text_preserves_list_items() {
        let md = "Shopping list:\n\n- Apples\n- Bananas\n- Cherries";
        let result = render_plain_text(md, &ChapterSeparator::ThreeStars);
        assert!(result.contains("- Apples"));
        assert!(result.contains("- Bananas"));
        assert!(result.contains("- Cherries"));
    }

    #[test]
    fn test_render_plain_text_preserves_code() {
        let md = "Use the `println!` macro.";
        let result = render_plain_text(md, &ChapterSeparator::ThreeStars);
        assert!(result.contains("println!"));
    }

    #[test]
    fn test_render_plain_text_preserves_paragraphs() {
        let md = "First paragraph.\n\nSecond paragraph.";
        let result = render_plain_text(md, &ChapterSeparator::ThreeStars);
        assert!(result.contains("First paragraph."));
        assert!(result.contains("Second paragraph."));
        // Should have blank line between paragraphs
        assert!(result.contains("First paragraph.\n\nSecond paragraph."));
    }

    #[test]
    fn test_render_plain_text_empty_input() {
        let result = render_plain_text("", &ChapterSeparator::ThreeStars);
        assert_eq!(result, "");
    }

    #[test]
    fn test_render_plain_text_plain_text_passthrough() {
        let md = "Just plain text with no formatting.";
        let result = render_plain_text(md, &ChapterSeparator::ThreeStars);
        assert_eq!(result, "Just plain text with no formatting.");
    }

    #[test]
    fn test_render_plain_text_blockquote_stripped() {
        let md = "> This is a quote.";
        let result = render_plain_text(md, &ChapterSeparator::ThreeStars);
        assert!(result.contains("This is a quote."));
        assert!(!result.contains(">"));
    }

    // ================================================================
    // Plain text full compilation integration tests
    // ================================================================

    #[test]
    fn plaintext_single_chapter_with_header() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(
            &pp,
            "ch-1",
            "The Beginning",
            None,
            "It was a dark and stormy night.",
        );

        let config = CompileConfig {
            title: "My Novel".to_string(),
            author: "Jane Author".to_string(),
            include_title_page: false,
            chapter_header_style: ChapterHeaderStyle::Titled,
            chapter_separator: ChapterSeparator::ThreeStars,
            output_format: OutputFormat::PlainText,
            include_synopsis: false,
            front_matter: String::new(),
        };

        let result = compile_manuscript(pp, config).unwrap();
        assert_eq!(result.format, OutputFormat::PlainText);
        assert!(result.content.contains("THE BEGINNING"));
        assert!(result.content.contains("-------------"));
        assert!(result.content.contains("It was a dark and stormy night."));
        // Should NOT contain markdown syntax
        assert!(!result.content.contains("## "));
    }

    #[test]
    fn plaintext_title_page_h1_with_equals_underline() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "One", None, "Body text.");

        let config = CompileConfig {
            title: "Epic Tale".to_string(),
            author: "A. Writer".to_string(),
            include_title_page: true,
            chapter_header_style: ChapterHeaderStyle::None,
            chapter_separator: ChapterSeparator::ThreeStars,
            output_format: OutputFormat::PlainText,
            include_synopsis: false,
            front_matter: String::new(),
        };

        let result = compile_manuscript(pp, config).unwrap();
        // Title should be uppercase H1 with = underline
        assert!(result.content.contains("EPIC TALE"));
        assert!(result.content.contains("========="));
        // Author should be present (bold stripped)
        assert!(result.content.contains("A. Writer"));
        // Should NOT contain markdown bold markers
        assert!(!result.content.contains("**"));
    }

    #[test]
    fn plaintext_chapter_header_numbered() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "The Journey", None, "Off we go.");

        let config = CompileConfig {
            title: "Novel".to_string(),
            author: "Author".to_string(),
            include_title_page: false,
            chapter_header_style: ChapterHeaderStyle::Numbered,
            chapter_separator: ChapterSeparator::ThreeStars,
            output_format: OutputFormat::PlainText,
            include_synopsis: false,
            front_matter: String::new(),
        };

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result.content.contains("CHAPTER 1"));
        assert!(result.content.contains("---------"));
    }

    #[test]
    fn plaintext_chapter_header_numbered_and_titled() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "The Beginning", None, "Content here.");

        let config = CompileConfig {
            title: "Novel".to_string(),
            author: "Author".to_string(),
            include_title_page: false,
            chapter_header_style: ChapterHeaderStyle::NumberedAndTitled,
            chapter_separator: ChapterSeparator::ThreeStars,
            output_format: OutputFormat::PlainText,
            include_synopsis: false,
            front_matter: String::new(),
        };

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result.content.contains("CHAPTER 1: THE BEGINNING"));
        assert!(result.content.contains("------------------------"));
    }

    #[test]
    fn plaintext_two_chapters_three_stars_separator() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1", "ch-2"]);
        write_chapter(&pp, "ch-1", "One", None, "First chapter.");
        write_chapter(&pp, "ch-2", "Two", None, "Second chapter.");

        let config = CompileConfig {
            title: "Novel".to_string(),
            author: "Author".to_string(),
            include_title_page: false,
            chapter_header_style: ChapterHeaderStyle::Titled,
            chapter_separator: ChapterSeparator::ThreeStars,
            output_format: OutputFormat::PlainText,
            include_synopsis: false,
            front_matter: String::new(),
        };

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result.content.contains("* * *"));
        assert!(result.content.contains("ONE"));
        assert!(result.content.contains("TWO"));
    }

    #[test]
    fn plaintext_two_chapters_page_break_separator() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1", "ch-2"]);
        write_chapter(&pp, "ch-1", "One", None, "First.");
        write_chapter(&pp, "ch-2", "Two", None, "Second.");

        let config = CompileConfig {
            title: "Novel".to_string(),
            author: "Author".to_string(),
            include_title_page: false,
            chapter_header_style: ChapterHeaderStyle::Titled,
            chapter_separator: ChapterSeparator::PageBreak,
            output_format: OutputFormat::PlainText,
            include_synopsis: false,
            front_matter: String::new(),
        };

        let result = compile_manuscript(pp, config).unwrap();
        // Page break should be rendered as equals signs
        assert!(result.content.contains(&"=".repeat(40)));
    }

    #[test]
    fn plaintext_two_chapters_horizontal_rule_separator() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1", "ch-2"]);
        write_chapter(&pp, "ch-1", "One", None, "First.");
        write_chapter(&pp, "ch-2", "Two", None, "Second.");

        let config = CompileConfig {
            title: "Novel".to_string(),
            author: "Author".to_string(),
            include_title_page: false,
            chapter_header_style: ChapterHeaderStyle::Titled,
            chapter_separator: ChapterSeparator::HorizontalRule,
            output_format: OutputFormat::PlainText,
            include_synopsis: false,
            front_matter: String::new(),
        };

        let result = compile_manuscript(pp, config).unwrap();
        // Horizontal rule should be rendered as dashes
        assert!(result.content.contains(&"-".repeat(40)));
    }

    #[test]
    fn plaintext_synopsis_stripped_of_italic_markers() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(
            &pp,
            "ch-1",
            "One",
            Some("The hero begins the journey"),
            "Content.",
        );

        let config = CompileConfig {
            title: "Novel".to_string(),
            author: "Author".to_string(),
            include_title_page: false,
            chapter_header_style: ChapterHeaderStyle::Titled,
            chapter_separator: ChapterSeparator::ThreeStars,
            output_format: OutputFormat::PlainText,
            include_synopsis: true,
            front_matter: String::new(),
        };

        let result = compile_manuscript(pp, config).unwrap();
        // Synopsis text present but without italic markers
        assert!(result.content.contains("The hero begins the journey"));
        assert!(!result.content.contains("*The hero begins the journey*"));
    }

    #[test]
    fn plaintext_front_matter_included() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "One", None, "Body.");

        let config = CompileConfig {
            title: "Novel".to_string(),
            author: "Author".to_string(),
            include_title_page: false,
            chapter_header_style: ChapterHeaderStyle::Titled,
            chapter_separator: ChapterSeparator::ThreeStars,
            output_format: OutputFormat::PlainText,
            include_synopsis: false,
            front_matter: "For those who dream.".to_string(),
        };

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result.content.starts_with("For those who dream."));
    }

    #[test]
    fn plaintext_word_count_computed_from_markdown_before_conversion() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "The Title", None, "One two three four five.");

        // Compare word count between Markdown and PlainText output
        let md_config = CompileConfig {
            title: "Novel".to_string(),
            author: "Author".to_string(),
            include_title_page: false,
            chapter_header_style: ChapterHeaderStyle::Titled,
            chapter_separator: ChapterSeparator::ThreeStars,
            output_format: OutputFormat::Markdown,
            include_synopsis: false,
            front_matter: String::new(),
        };

        let pt_config = CompileConfig {
            output_format: OutputFormat::PlainText,
            ..md_config.clone()
        };

        let md_result = compile_manuscript(pp.clone(), md_config).unwrap();
        let pt_result = compile_manuscript(pp, pt_config).unwrap();

        // Word counts should be identical since both are computed from markdown
        assert_eq!(md_result.word_count, pt_result.word_count);
    }

    #[test]
    fn plaintext_no_markdown_hash_in_headers() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "Hello World", None, "Body.");

        let config = CompileConfig {
            title: "My Title".to_string(),
            author: "Author".to_string(),
            include_title_page: true,
            chapter_header_style: ChapterHeaderStyle::NumberedAndTitled,
            chapter_separator: ChapterSeparator::ThreeStars,
            output_format: OutputFormat::PlainText,
            include_synopsis: false,
            front_matter: String::new(),
        };

        let result = compile_manuscript(pp, config).unwrap();
        // No markdown header syntax should remain
        assert!(!result.content.contains("# "));
        assert!(!result.content.contains("## "));
    }

    #[test]
    fn plaintext_no_bold_markers_anywhere() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "One", None, "This has **bold** text.");

        let config = CompileConfig {
            title: "Title".to_string(),
            author: "Author Name".to_string(),
            include_title_page: true,
            chapter_header_style: ChapterHeaderStyle::Titled,
            chapter_separator: ChapterSeparator::ThreeStars,
            output_format: OutputFormat::PlainText,
            include_synopsis: false,
            front_matter: String::new(),
        };

        let result = compile_manuscript(pp, config).unwrap();
        assert!(!result.content.contains("**"));
    }

    #[test]
    fn plaintext_full_compilation_with_all_features() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["prologue", "ch-1", "ch-2"]);
        write_chapter(
            &pp,
            "prologue",
            "Prologue",
            Some("The world before"),
            "In the beginning...",
        );
        write_chapter(
            &pp,
            "ch-1",
            "The Journey",
            Some("Our hero departs"),
            "The hero set out at dawn.",
        );
        write_chapter(&pp, "ch-2", "The Return", None, "And so it ended.");

        let config = CompileConfig {
            title: "Epic Tale".to_string(),
            author: "A. Writer".to_string(),
            include_title_page: true,
            chapter_header_style: ChapterHeaderStyle::NumberedAndTitled,
            chapter_separator: ChapterSeparator::ThreeStars,
            output_format: OutputFormat::PlainText,
            include_synopsis: true,
            front_matter: "For those who dream.".to_string(),
        };

        let result = compile_manuscript(pp, config).unwrap();

        // Front matter
        assert!(result.content.contains("For those who dream."));
        // Title page (H1 with = underline, no markdown syntax)
        assert!(result.content.contains("EPIC TALE"));
        assert!(result.content.contains("========="));
        assert!(result.content.contains("A. Writer"));
        assert!(!result.content.contains("**"));
        assert!(!result.content.contains("# "));
        // Chapter headers (H2 with - underline)
        assert!(result.content.contains("CHAPTER 1: PROLOGUE"));
        assert!(result.content.contains("CHAPTER 2: THE JOURNEY"));
        assert!(result.content.contains("CHAPTER 3: THE RETURN"));
        // Synopses (no italic markers)
        assert!(result.content.contains("The world before"));
        assert!(result.content.contains("Our hero departs"));
        assert!(!result.content.contains("*The world before*"));
        // Bodies (smart punctuation converts ... to ellipsis character)
        assert!(result.content.contains("In the beginning\u{2026}"));
        assert!(result.content.contains("The hero set out at dawn."));
        assert!(result.content.contains("And so it ended."));
        // Separators (three stars)
        assert!(result.content.contains("* * *"));
        // Metadata
        assert_eq!(result.chapter_count, 3);
        assert_eq!(result.format, OutputFormat::PlainText);
        assert!(result.word_count > 0);
    }

    #[test]
    fn plaintext_empty_manuscript() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();
        write_config(&pp, &[]);

        let config = CompileConfig {
            title: "Empty".to_string(),
            author: "Author".to_string(),
            include_title_page: false,
            chapter_header_style: ChapterHeaderStyle::Titled,
            chapter_separator: ChapterSeparator::ThreeStars,
            output_format: OutputFormat::PlainText,
            include_synopsis: false,
            front_matter: String::new(),
        };

        let result = compile_manuscript(pp, config).unwrap();
        assert_eq!(result.content, "");
        assert_eq!(result.chapter_count, 0);
        assert_eq!(result.word_count, 0);
    }

    #[test]
    fn plaintext_underline_width_matches_header_text() {
        let md = "## Short";
        let result = render_plain_text(md, &ChapterSeparator::ThreeStars);
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "SHORT");
        assert_eq!(lines[1], "-----");
        assert_eq!(lines[0].len(), lines[1].len());
    }

    #[test]
    fn plaintext_h1_underline_width_matches_header_text() {
        let md = "# A Longer Title Here";
        let result = render_plain_text(md, &ChapterSeparator::ThreeStars);
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines[0], "A LONGER TITLE HERE");
        assert_eq!(lines[1], "===================");
        assert_eq!(lines[0].len(), lines[1].len());
    }

    // ── Word count accuracy across multiple chapters ──────────────

    #[test]
    fn word_count_accurate_across_multiple_chapters() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1", "ch-2", "ch-3"]);
        write_chapter(&pp, "ch-1", "A", None, "one two three"); // 3 words
        write_chapter(&pp, "ch-2", "B", None, "four five"); // 2 words
        write_chapter(&pp, "ch-3", "C", None, "six"); // 1 word

        let mut config = default_config();
        config.chapter_header_style = ChapterHeaderStyle::None;

        let result = compile_manuscript(pp, config).unwrap();
        // Body words: 3 + 2 + 1 = 6
        // The separator "* * *" adds words too (3 per separator, 2 separators = 6)
        // Total depends on separator choice
        // With ThreeStars (default in test helper): "* * *" = 3 words, 2 seps = 6
        // Total = 6 body + 6 separator = 12
        assert_eq!(result.word_count, 12);
    }

    #[test]
    fn word_count_accurate_no_separators_noise() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1", "ch-2"]);
        write_chapter(&pp, "ch-1", "A", None, "alpha beta gamma"); // 3 words
        write_chapter(&pp, "ch-2", "B", None, "delta epsilon"); // 2 words

        let mut config = default_config();
        config.chapter_header_style = ChapterHeaderStyle::None;
        config.chapter_separator = ChapterSeparator::BlankLines;

        let result = compile_manuscript(pp, config).unwrap();
        // BlankLines separator is "\n\n\n\n" — zero words
        // Total = 3 + 2 = 5
        assert_eq!(result.word_count, 5);
    }

    #[test]
    fn word_count_includes_markdown_syntax_tokens() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "Title", None, "**bold** and *italic* text");

        let mut config = default_config();
        config.chapter_header_style = ChapterHeaderStyle::None;

        let result = compile_manuscript(pp, config).unwrap();
        // count_words splits on whitespace: "**bold**" "and" "*italic*" "text" = 4
        assert_eq!(result.word_count, 4);
    }

    #[test]
    fn word_count_zero_for_empty_document() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &[]);

        let config = default_config();
        let result = compile_manuscript(pp, config).unwrap();
        assert_eq!(result.word_count, 0);
        assert_eq!(result.chapter_count, 0);
    }

    // ── Chapter count matches actual compiled chapters ─────────────

    #[test]
    fn chapter_count_with_some_missing_files() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(
            &pp,
            &["real-1", "missing-1", "real-2", "missing-2", "real-3"],
        );
        write_chapter(&pp, "real-1", "Real One", None, "Content 1.");
        write_chapter(&pp, "real-2", "Real Two", None, "Content 2.");
        write_chapter(&pp, "real-3", "Real Three", None, "Content 3.");
        // missing-1 and missing-2 not created

        let result = compile_manuscript(pp, default_config()).unwrap();
        assert_eq!(result.chapter_count, 3);
        assert!(result.content.contains("Content 1."));
        assert!(result.content.contains("Content 2."));
        assert!(result.content.contains("Content 3."));
    }

    #[test]
    fn chapter_count_all_missing() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["missing-1", "missing-2"]);
        // No chapter files created

        let result = compile_manuscript(pp, default_config()).unwrap();
        assert_eq!(result.chapter_count, 0);
        assert_eq!(result.word_count, 0);
        // Content may have title page etc, but no chapters
    }

    // ── Separator placement ───────────────────────────────────────

    #[test]
    fn no_separator_before_first_chapter() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1", "ch-2"]);
        write_chapter(&pp, "ch-1", "First", None, "First body.");
        write_chapter(&pp, "ch-2", "Second", None, "Second body.");

        let mut config = default_config();
        config.chapter_separator = ChapterSeparator::ThreeStars;

        let result = compile_manuscript(pp, config).unwrap();
        // Content should start with the chapter header, not a separator
        assert!(result.content.starts_with("## First"));
        // There should be exactly 1 separator (between ch-1 and ch-2)
        assert_eq!(result.content.matches("* * *").count(), 1);
    }

    #[test]
    fn no_separator_after_last_chapter_trimmed() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1", "ch-2", "ch-3"]);
        write_chapter(&pp, "ch-1", "A", None, "Body A.");
        write_chapter(&pp, "ch-2", "B", None, "Body B.");
        write_chapter(&pp, "ch-3", "C", None, "Body C final content.");

        let mut config = default_config();
        config.chapter_separator = ChapterSeparator::ThreeStars;

        let result = compile_manuscript(pp, config).unwrap();
        // Content should end with the last chapter's body, not a separator
        assert!(result.content.trim_end().ends_with("Body C final content."));
        // Exactly 2 separators for 3 chapters
        assert_eq!(result.content.matches("* * *").count(), 2);
    }

    #[test]
    fn single_chapter_has_no_separator() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "Solo", None, "All alone.");

        let mut config = default_config();
        config.chapter_separator = ChapterSeparator::ThreeStars;

        let result = compile_manuscript(pp, config).unwrap();
        assert!(!result.content.contains("* * *"));
        assert!(result.content.contains("All alone."));
        assert_eq!(result.chapter_count, 1);
    }

    // ── Chapter ordering (additional) ─────────────────────────────

    #[test]
    fn chapters_in_reverse_alphabetical_order_follow_config() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        // Config order is reverse alphabetical
        write_config(&pp, &["zebra", "mango", "apple"]);
        write_chapter(&pp, "zebra", "Zebra", None, "I am zebra.");
        write_chapter(&pp, "mango", "Mango", None, "I am mango.");
        write_chapter(&pp, "apple", "Apple", None, "I am apple.");

        let result = compile_manuscript(pp, default_config()).unwrap();
        let zebra_pos = result.content.find("I am zebra.").unwrap();
        let mango_pos = result.content.find("I am mango.").unwrap();
        let apple_pos = result.content.find("I am apple.").unwrap();
        assert!(
            zebra_pos < mango_pos,
            "zebra should come before mango per config"
        );
        assert!(
            mango_pos < apple_pos,
            "mango should come before apple per config"
        );
        assert_eq!(result.chapter_count, 3);
    }

    #[test]
    fn chapter_numbering_skips_missing_chapters_in_sequence() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1", "ch-missing", "ch-3"]);
        write_chapter(&pp, "ch-1", "First", None, "Body 1.");
        // ch-missing not created
        write_chapter(&pp, "ch-3", "Third", None, "Body 3.");

        let mut config = default_config();
        config.chapter_header_style = ChapterHeaderStyle::Numbered;

        let result = compile_manuscript(pp, config).unwrap();
        // First existing chapter is Chapter 1, second existing is Chapter 2
        // (numbering is sequential for compiled chapters, not config indices)
        assert!(result.content.contains("## Chapter 1"));
        assert!(result.content.contains("## Chapter 2"));
        assert!(!result.content.contains("## Chapter 3"));
        assert_eq!(result.chapter_count, 2);
    }

    // ── Synopsis edge cases ───────────────────────────────────────

    #[test]
    fn synopsis_with_empty_string_not_rendered() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "One", Some(""), "Body text.");

        let mut config = default_config();
        config.include_synopsis = true;

        let result = compile_manuscript(pp, config).unwrap();
        // Empty synopsis should not produce italic markers
        assert!(!result.content.contains("**"));
        assert!(result.content.contains("Body text."));
    }

    #[test]
    fn synopsis_on_multiple_chapters_mixed() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1", "ch-2", "ch-3"]);
        write_chapter(&pp, "ch-1", "One", Some("Synopsis for one"), "Body 1.");
        write_chapter(&pp, "ch-2", "Two", None, "Body 2.");
        write_chapter(&pp, "ch-3", "Three", Some("Synopsis for three"), "Body 3.");

        let mut config = default_config();
        config.include_synopsis = true;

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result.content.contains("*Synopsis for one*"));
        assert!(!result.content.contains("*Synopsis for two*"));
        assert!(result.content.contains("*Synopsis for three*"));
    }

    // ── Edge case: empty body chapters ────────────────────────────

    #[test]
    fn multiple_chapters_some_with_empty_bodies() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1", "ch-2", "ch-3"]);
        write_chapter(&pp, "ch-1", "Full", None, "Has content.");
        write_chapter(&pp, "ch-2", "Empty", None, "");
        write_chapter(&pp, "ch-3", "Also Full", None, "Also has content.");

        let result = compile_manuscript(pp, default_config()).unwrap();
        assert!(result.content.contains("## Full"));
        assert!(result.content.contains("## Empty"));
        assert!(result.content.contains("## Also Full"));
        assert!(result.content.contains("Has content."));
        assert!(result.content.contains("Also has content."));
        assert_eq!(result.chapter_count, 3);
    }

    // ── Front matter + title page ordering ────────────────────────

    #[test]
    fn front_matter_before_title_page_before_chapters() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "One", None, "Chapter body.");

        let mut config = default_config();
        config.front_matter = "FRONT MATTER TEXT".to_string();
        config.include_title_page = true;
        config.title = "TITLE".to_string();
        config.author = "AUTHOR".to_string();

        let result = compile_manuscript(pp, config).unwrap();

        let fm_pos = result.content.find("FRONT MATTER TEXT").unwrap();
        let title_pos = result.content.find("# TITLE").unwrap();
        let body_pos = result.content.find("Chapter body.").unwrap();
        assert!(
            fm_pos < title_pos,
            "Front matter should come before title page"
        );
        assert!(
            title_pos < body_pos,
            "Title page should come before chapter body"
        );
    }

    // ── count_words edge cases ────────────────────────────────────

    #[test]
    fn test_count_words_with_newlines() {
        assert_eq!(count_words("hello\nworld\nfoo"), 3);
    }

    #[test]
    fn test_count_words_with_tabs() {
        assert_eq!(count_words("hello\tworld"), 2);
    }

    #[test]
    fn test_count_words_with_mixed_whitespace() {
        assert_eq!(count_words("  hello  \n\n  world  \t  foo  "), 3);
    }

    #[test]
    fn test_count_words_with_punctuation() {
        // Punctuation attached to words counts as part of the word
        assert_eq!(count_words("hello, world! foo."), 3);
    }

    #[test]
    fn test_count_words_markdown_bold() {
        assert_eq!(count_words("**bold** text"), 2);
    }

    #[test]
    fn test_count_words_markdown_header() {
        assert_eq!(count_words("## Chapter 1: Title"), 4);
    }

    // ── chapter_header edge cases ─────────────────────────────────

    #[test]
    fn test_chapter_header_with_special_chars() {
        assert_eq!(
            chapter_header(&ChapterHeaderStyle::Titled, 1, "A \"Brave\" & <Bold> Move"),
            Some("## A \"Brave\" & <Bold> Move".to_string())
        );
    }

    #[test]
    fn test_chapter_header_with_unicode() {
        assert_eq!(
            chapter_header(&ChapterHeaderStyle::NumberedAndTitled, 7, "第七章"),
            Some("## Chapter 7: 第七章".to_string())
        );
    }

    #[test]
    fn test_chapter_header_with_empty_title() {
        // Even an empty title produces a header for Titled style
        assert_eq!(
            chapter_header(&ChapterHeaderStyle::Titled, 1, ""),
            Some("## ".to_string())
        );
        // For NumberedAndTitled, it shows "## Chapter 1: "
        assert_eq!(
            chapter_header(&ChapterHeaderStyle::NumberedAndTitled, 1, ""),
            Some("## Chapter 1: ".to_string())
        );
    }

    // ── Output format is correctly passed through ─────────────────

    #[test]
    fn output_format_preserved_with_content() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "One", None, "Body.");

        for format in [
            OutputFormat::Markdown,
            OutputFormat::Html,
            OutputFormat::PlainText,
        ] {
            let mut config = default_config();
            config.output_format = format.clone();

            let result = compile_manuscript(pp.clone(), config).unwrap();
            assert_eq!(result.format, format);
            assert!(result.content.contains("Body."));
        }
    }

    // ========================================================================
    // HTML export tests (ITEM-101)
    // ========================================================================

    #[test]
    fn html_output_has_valid_document_structure() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();
        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "First", None, "Hello world.");

        let mut config = default_config();
        config.output_format = OutputFormat::Html;

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result.content.starts_with("<!DOCTYPE html>"));
        assert!(result.content.contains("<html lang=\"en\">"));
        assert!(result.content.contains("<head>"));
        assert!(result.content.contains("<meta charset=\"UTF-8\">"));
        assert!(result.content.contains("</head>"));
        assert!(result.content.contains("<body>"));
        assert!(result.content.contains("</body>"));
        assert!(result.content.contains("</html>"));
    }

    #[test]
    fn html_output_has_embedded_css() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();
        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "First", None, "Hello world.");

        let mut config = default_config();
        config.output_format = OutputFormat::Html;

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result.content.contains("<style>"));
        assert!(result.content.contains("</style>"));
        // Check key CSS rules are present
        assert!(result.content.contains("font-family: Georgia"));
        assert!(result.content.contains("page-break-before: always"));
        assert!(result.content.contains("max-width: 720px"));
        assert!(result.content.contains("@media print"));
    }

    #[test]
    fn html_output_converts_markdown_formatting() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();
        write_config(&pp, &["ch-1"]);
        write_chapter(
            &pp,
            "ch-1",
            "Test Chapter",
            None,
            "This is **bold** and *italic* text.",
        );

        let mut config = default_config();
        config.output_format = OutputFormat::Html;

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result.content.contains("<strong>bold</strong>"));
        assert!(result.content.contains("<em>italic</em>"));
    }

    #[test]
    fn html_output_converts_chapter_headers() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();
        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "The Journey", None, "Content here.");

        let mut config = default_config();
        config.output_format = OutputFormat::Html;
        config.chapter_header_style = ChapterHeaderStyle::Titled;

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result.content.contains("<h2>The Journey</h2>"));
    }

    #[test]
    fn html_output_includes_title_in_head() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();
        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "First", None, "Hello.");

        let mut config = default_config();
        config.output_format = OutputFormat::Html;
        config.title = "My Great Novel".to_string();

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result.content.contains("<title>My Great Novel</title>"));
    }

    #[test]
    fn html_output_escapes_title_in_head() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();
        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "First", None, "Hello.");

        let mut config = default_config();
        config.output_format = OutputFormat::Html;
        config.title = "Tom & Jerry <script>".to_string();

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result
            .content
            .contains("<title>Tom &amp; Jerry &lt;script&gt;</title>"));
        // Ensure the raw dangerous characters are NOT in the title tag
        assert!(!result.content.contains("<title>Tom & Jerry"));
    }

    #[test]
    fn html_output_has_title_page() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();
        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "First", None, "Hello.");

        let mut config = default_config();
        config.output_format = OutputFormat::Html;
        config.include_title_page = true;
        config.title = "Epic Tale".to_string();
        config.author = "A. Writer".to_string();

        let result = compile_manuscript(pp, config).unwrap();
        // Title page content should be converted from markdown (# Title -> <h1>)
        assert!(result.content.contains("<h1>Epic Tale</h1>"));
        // Author should be bold
        assert!(result.content.contains("<strong>A. Writer</strong>"));
    }

    #[test]
    fn html_output_renders_separators_as_hr() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();
        write_config(&pp, &["ch-1", "ch-2"]);
        write_chapter(&pp, "ch-1", "One", None, "First chapter.");
        write_chapter(&pp, "ch-2", "Two", None, "Second chapter.");

        let mut config = default_config();
        config.output_format = OutputFormat::Html;
        config.chapter_separator = ChapterSeparator::ThreeStars;

        let result = compile_manuscript(pp, config).unwrap();
        // pulldown-cmark converts "* * *" to <hr />
        assert!(result.content.contains("<hr />"));
    }

    #[test]
    fn html_output_word_count_from_markdown() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();
        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "First", None, "One two three four five.");

        let mut config_md = default_config();
        config_md.output_format = OutputFormat::Markdown;

        let mut config_html = default_config();
        config_html.output_format = OutputFormat::Html;

        let result_md = compile_manuscript(pp.clone(), config_md).unwrap();
        let result_html = compile_manuscript(pp, config_html).unwrap();

        // Word count should be the same regardless of output format
        // because it's calculated from the markdown before conversion
        assert_eq!(result_md.word_count, result_html.word_count);
    }

    #[test]
    fn html_output_preserves_format_field() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();
        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "First", None, "Hello.");

        let mut config = default_config();
        config.output_format = OutputFormat::Html;

        let result = compile_manuscript(pp, config).unwrap();
        assert_eq!(result.format, OutputFormat::Html);
    }

    #[test]
    fn html_output_renders_synopsis() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();
        write_config(&pp, &["ch-1"]);
        write_chapter(
            &pp,
            "ch-1",
            "The Journey",
            Some("Our hero departs"),
            "The story begins.",
        );

        let mut config = default_config();
        config.output_format = OutputFormat::Html;
        config.include_synopsis = true;

        let result = compile_manuscript(pp, config).unwrap();
        // Synopsis is *italic* in markdown, converted to <em> in HTML
        assert!(result.content.contains("<em>Our hero departs</em>"));
    }

    #[test]
    fn html_output_empty_manuscript() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();
        write_config(&pp, &[]);

        let mut config = default_config();
        config.output_format = OutputFormat::Html;

        let result = compile_manuscript(pp, config).unwrap();
        // Empty manuscript should still return empty content, not an HTML wrapper
        assert_eq!(result.content, "");
        assert_eq!(result.format, OutputFormat::Html);
        assert_eq!(result.chapter_count, 0);
        assert_eq!(result.word_count, 0);
    }

    #[test]
    fn html_output_front_matter() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();
        write_config(&pp, &["ch-1"]);
        write_chapter(&pp, "ch-1", "First", None, "Hello.");

        let mut config = default_config();
        config.output_format = OutputFormat::Html;
        config.front_matter = "For those who dream.".to_string();

        let result = compile_manuscript(pp, config).unwrap();
        assert!(result.content.contains("For those who dream."));
    }

    #[test]
    fn html_full_compilation_with_all_features() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_config(&pp, &["prologue", "ch-1", "ch-2"]);
        write_chapter(
            &pp,
            "prologue",
            "Prologue",
            Some("The world before"),
            "In the beginning...",
        );
        write_chapter(
            &pp,
            "ch-1",
            "The Journey",
            Some("Our hero departs"),
            "The hero set out at dawn.",
        );
        write_chapter(&pp, "ch-2", "The Return", None, "And so it ended.");

        let config = CompileConfig {
            title: "Epic Tale".to_string(),
            author: "A. Writer".to_string(),
            include_title_page: true,
            chapter_header_style: ChapterHeaderStyle::NumberedAndTitled,
            chapter_separator: ChapterSeparator::ThreeStars,
            output_format: OutputFormat::Html,
            include_synopsis: true,
            front_matter: "For those who dream.".to_string(),
        };

        let result = compile_manuscript(pp, config).unwrap();

        // Valid HTML structure
        assert!(result.content.starts_with("<!DOCTYPE html>"));
        assert!(result.content.contains("<html lang=\"en\">"));
        assert!(result.content.contains("</html>"));

        // Title in head
        assert!(result.content.contains("<title>Epic Tale</title>"));

        // Embedded CSS
        assert!(result.content.contains("<style>"));

        // Front matter
        assert!(result.content.contains("For those who dream."));

        // Title page (converted from Markdown)
        assert!(result.content.contains("<h1>Epic Tale</h1>"));
        assert!(result.content.contains("<strong>A. Writer</strong>"));

        // Chapter headers (## -> h2)
        assert!(result.content.contains("<h2>Chapter 1: Prologue</h2>"));
        assert!(result.content.contains("<h2>Chapter 2: The Journey</h2>"));
        assert!(result.content.contains("<h2>Chapter 3: The Return</h2>"));

        // Synopses
        assert!(result.content.contains("<em>The world before</em>"));
        assert!(result.content.contains("<em>Our hero departs</em>"));

        // Bodies
        assert!(result.content.contains("In the beginning"));
        assert!(result.content.contains("The hero set out at dawn."));
        assert!(result.content.contains("And so it ended."));

        // Separators
        assert!(result.content.contains("<hr />"));

        // Metadata
        assert_eq!(result.chapter_count, 3);
        assert_eq!(result.format, OutputFormat::Html);
        assert!(result.word_count > 0);
    }

    #[test]
    fn test_render_html_basic() {
        let html = render_html("# Hello\n\nWorld", "Test Title");
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<title>Test Title</title>"));
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("<p>World</p>"));
        assert!(html.contains("<style>"));
    }

    #[test]
    fn test_render_html_preserves_markdown_features() {
        let md = "**bold** *italic* [link](http://example.com)\n\n- item 1\n- item 2";
        let html = render_html(md, "Features");
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<em>italic</em>"));
        assert!(html.contains("<a href=\"http://example.com\">link</a>"));
        assert!(html.contains("<li>item 1</li>"));
        assert!(html.contains("<li>item 2</li>"));
    }

    #[test]
    fn test_html_escape_function() {
        assert_eq!(html_escape("Hello"), "Hello");
        assert_eq!(html_escape("a & b"), "a &amp; b");
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("say \"hi\""), "say &quot;hi&quot;");
        assert_eq!(
            html_escape("Tom & Jerry <\"hi\">"),
            "Tom &amp; Jerry &lt;&quot;hi&quot;&gt;"
        );
    }
}
