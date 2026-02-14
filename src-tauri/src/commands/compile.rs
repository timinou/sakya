use crate::error::AppError;
use crate::models::compile::{ChapterHeaderStyle, ChapterSeparator, CompileConfig, CompileOutput};
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

    Ok(CompileOutput {
        content,
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
}
