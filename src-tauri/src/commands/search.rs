use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::error::AppError;

// ── Models ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub title: String,
    pub slug: String,
    pub file_type: String,
    pub entity_type: Option<String>,
    pub matching_line: String,
    pub line_number: usize,
    pub context_before: String,
    pub context_after: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WikiLinkTarget {
    pub title: String,
    pub slug: String,
    pub file_type: String,
    pub entity_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BacklinkResult {
    pub title: String,
    pub slug: String,
    pub file_type: String,
    pub entity_type: Option<String>,
    pub matching_line: String,
    pub line_number: usize,
}

// ── Minimal frontmatter for search ────────────────────────────────

/// We only need title + slug from any file's frontmatter.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MinimalFrontmatter {
    title: String,
    slug: String,
}

// ── Helpers ───────────────────────────────────────────────────────

/// Determine file_type and entity_type from a file's path relative to the project root.
fn classify_file(project_path: &Path, file_path: &Path) -> Option<(String, Option<String>)> {
    let rel = file_path.strip_prefix(project_path).ok()?;
    let components: Vec<&str> = rel
        .components()
        .map(|c| c.as_os_str().to_str().unwrap_or(""))
        .collect();

    if components.first() == Some(&"manuscript") {
        Some(("chapter".to_string(), None))
    } else if components.first() == Some(&"entities") && components.len() >= 3 {
        let entity_type = components[1].to_string();
        Some(("entity".to_string(), Some(entity_type)))
    } else if components.first() == Some(&"notes") {
        Some(("note".to_string(), None))
    } else {
        None
    }
}

/// Parse the YAML frontmatter (between --- delimiters) from a markdown string.
/// Returns (MinimalFrontmatter, body_start_line_index) where body_start_line_index
/// is the 0-based index of the first line after the closing ---.
fn parse_frontmatter(content: &str) -> Result<(MinimalFrontmatter, usize), AppError> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return Err(AppError::Validation(
            "Document does not start with frontmatter delimiter".to_string(),
        ));
    }

    let after_first = &trimmed[3..];
    let end_pos = after_first
        .find("\n---")
        .ok_or_else(|| AppError::Validation("Missing closing frontmatter delimiter".to_string()))?;

    let yaml_str = &after_first[..end_pos];
    let fm: MinimalFrontmatter = serde_yaml::from_str(yaml_str)?;

    // Count lines: the opening --- is line 0, then yaml content, then closing ---
    // body_start_line is the line after closing ---
    let prefix = &content[..content.len() - after_first[end_pos..].len()];
    let body_start_line = prefix.lines().count() + 1; // +1 for the closing --- line

    Ok((fm, body_start_line))
}

/// Walk all .md files in the project's manuscript/, entities/, and notes/ directories.
fn walk_md_files(project_path: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let dirs = ["manuscript", "entities", "notes"];

    for dir in &dirs {
        let dir_path = project_path.join(dir);
        if !dir_path.exists() {
            continue;
        }
        for entry in WalkDir::new(&dir_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path().to_path_buf();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
                files.push(path);
            }
        }
    }

    files
}

/// File-type priority for search result sorting (lower = higher priority).
fn file_type_priority(file_type: &str) -> u8 {
    match file_type {
        "chapter" => 0,
        "entity" => 1,
        "note" => 2,
        _ => 3,
    }
}

// ── Commands ──────────────────────────────────────────────────────

/// Full-text search across all project .md files.
///
/// Returns up to 50 results, sorted by relevance:
/// - Exact title matches first
/// - Then by file_type priority (chapter > entity > note)
///
/// The frontmatter section is skipped for body search, but the title field is searched.
#[tauri::command]
pub fn search_project(project_path: String, query: String) -> Result<Vec<SearchResult>, AppError> {
    if query.is_empty() {
        return Ok(Vec::new());
    }

    let project = PathBuf::from(&project_path);
    let query_lower = query.to_lowercase();
    let mut results: Vec<(bool, SearchResult)> = Vec::new();

    for file_path in walk_md_files(&project) {
        let (file_type, entity_type) = match classify_file(&project, &file_path) {
            Some(c) => c,
            None => continue,
        };

        let content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let (fm, body_start_line) = match parse_frontmatter(&content) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let title_matches = fm.title.to_lowercase().contains(&query_lower);
        let is_exact_title = fm.title.to_lowercase() == query_lower;

        let lines: Vec<&str> = content.lines().collect();

        // Search title first — if it matches, add a result for the title line
        if title_matches {
            // Find the title line in frontmatter
            for (i, line) in lines.iter().enumerate() {
                if i >= body_start_line {
                    break;
                }
                if line.to_lowercase().contains("title:")
                    && line.to_lowercase().contains(&query_lower)
                {
                    let context_before = if i > 0 {
                        lines[i - 1].to_string()
                    } else {
                        String::new()
                    };
                    let context_after = if i + 1 < lines.len() {
                        lines[i + 1].to_string()
                    } else {
                        String::new()
                    };
                    results.push((
                        is_exact_title,
                        SearchResult {
                            title: fm.title.clone(),
                            slug: fm.slug.clone(),
                            file_type: file_type.clone(),
                            entity_type: entity_type.clone(),
                            matching_line: line.to_string(),
                            line_number: i + 1,
                            context_before,
                            context_after,
                        },
                    ));
                    break;
                }
            }
        }

        // Search body lines (after frontmatter)
        for (i, line) in lines.iter().enumerate() {
            if i < body_start_line {
                continue;
            }
            if line.to_lowercase().contains(&query_lower) {
                let context_before = if i > 0 {
                    lines[i - 1].to_string()
                } else {
                    String::new()
                };
                let context_after = if i + 1 < lines.len() {
                    lines[i + 1].to_string()
                } else {
                    String::new()
                };
                results.push((
                    is_exact_title,
                    SearchResult {
                        title: fm.title.clone(),
                        slug: fm.slug.clone(),
                        file_type: file_type.clone(),
                        entity_type: entity_type.clone(),
                        matching_line: line.to_string(),
                        line_number: i + 1,
                        context_before,
                        context_after,
                    },
                ));
            }
        }
    }

    // Sort: exact title matches first, then by file_type priority, then line number
    results.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| {
                file_type_priority(&a.1.file_type).cmp(&file_type_priority(&b.1.file_type))
            })
            .then_with(|| a.1.line_number.cmp(&b.1.line_number))
    });

    let capped: Vec<SearchResult> = results.into_iter().take(50).map(|(_, r)| r).collect();
    Ok(capped)
}

/// Resolve a wiki-link text to its target file.
///
/// Matches case-insensitively against file titles parsed from frontmatter.
/// Returns the first match, or NotFound if no file has a matching title.
#[tauri::command]
pub fn resolve_wiki_link(
    project_path: String,
    link_text: String,
) -> Result<WikiLinkTarget, AppError> {
    let project = PathBuf::from(&project_path);
    let link_lower = link_text.to_lowercase();

    for file_path in walk_md_files(&project) {
        let (file_type, entity_type) = match classify_file(&project, &file_path) {
            Some(c) => c,
            None => continue,
        };

        let content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let (fm, _) = match parse_frontmatter(&content) {
            Ok(r) => r,
            Err(_) => continue,
        };

        if fm.title.to_lowercase() == link_lower {
            return Ok(WikiLinkTarget {
                title: fm.title,
                slug: fm.slug,
                file_type,
                entity_type,
            });
        }
    }

    Err(AppError::NotFound(format!(
        "No file found with title matching: {}",
        link_text
    )))
}

/// Find all files that contain a wiki-link to the given title.
///
/// Searches for the pattern `[[{title}]]` (case-insensitive) in all .md files.
#[tauri::command]
pub fn find_backlinks(
    project_path: String,
    title: String,
) -> Result<Vec<BacklinkResult>, AppError> {
    let project = PathBuf::from(&project_path);
    let pattern = format!("[[{}]]", title.to_lowercase());
    let mut results: Vec<BacklinkResult> = Vec::new();

    for file_path in walk_md_files(&project) {
        let (file_type, entity_type) = match classify_file(&project, &file_path) {
            Some(c) => c,
            None => continue,
        };

        let content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let (fm, _) = match parse_frontmatter(&content) {
            Ok(r) => r,
            Err(_) => continue,
        };

        for (i, line) in content.lines().enumerate() {
            if line.to_lowercase().contains(&pattern) {
                results.push(BacklinkResult {
                    title: fm.title.clone(),
                    slug: fm.slug.clone(),
                    file_type: file_type.clone(),
                    entity_type: entity_type.clone(),
                    matching_line: line.to_string(),
                    line_number: i + 1,
                });
            }
        }
    }

    Ok(results)
}

// ── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::setup_test_dir;

    /// Helper: write a markdown file with frontmatter.
    fn write_md(dir: &Path, rel_path: &str, title: &str, slug: &str, body: &str) {
        let path = dir.join(rel_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let content = format!(
            "---\ntitle: \"{}\"\nslug: \"{}\"\n---\n{}",
            title, slug, body
        );
        std::fs::write(&path, content).unwrap();
    }

    /// Helper: write a markdown file with extra frontmatter fields (for entities).
    fn write_entity_md(dir: &Path, entity_type: &str, slug: &str, title: &str, body: &str) {
        let rel = format!("entities/{}/{}.md", entity_type, slug);
        let path = dir.join(&rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let content = format!(
            "---\ntitle: \"{}\"\nslug: \"{}\"\nschemaType: \"{}\"\n---\n{}",
            title, slug, entity_type, body
        );
        std::fs::write(&path, content).unwrap();
    }

    // ── search_project ────────────────────────────────────────────

    #[test]
    fn search_returns_empty_for_empty_query() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_md(
            dir.path(),
            "notes/test.md",
            "Test Note",
            "test",
            "Some content\n",
        );

        let results = search_project(pp, String::new()).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn search_returns_empty_when_no_matches() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_md(
            dir.path(),
            "notes/test.md",
            "Test Note",
            "test",
            "Hello world\n",
        );

        let results = search_project(pp, "zzzznonexistent".to_string()).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn search_finds_match_in_note_body() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_md(
            dir.path(),
            "notes/quest.md",
            "Quest Ideas",
            "quest",
            "The dragon sleeps in the mountain.\nHeroes must find the sword.\n",
        );

        let results = search_project(pp, "dragon".to_string()).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Quest Ideas");
        assert_eq!(results[0].slug, "quest");
        assert_eq!(results[0].file_type, "note");
        assert!(results[0].entity_type.is_none());
        assert!(results[0].matching_line.contains("dragon"));
    }

    #[test]
    fn search_is_case_insensitive() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_md(
            dir.path(),
            "notes/case.md",
            "Case Test",
            "case",
            "The DRAGON roars loudly.\n",
        );

        let results = search_project(pp, "dragon".to_string()).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].matching_line.contains("DRAGON"));
    }

    #[test]
    fn search_across_file_types() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_md(
            dir.path(),
            "manuscript/ch1.md",
            "Chapter One",
            "ch1",
            "The hero found a magic crystal.\n",
        );
        write_entity_md(
            dir.path(),
            "character",
            "alice",
            "Alice",
            "Alice found a crystal pendant.\n",
        );
        write_md(
            dir.path(),
            "notes/idea.md",
            "Crystal Ideas",
            "idea",
            "Crystals are important plot devices.\n",
        );

        let results = search_project(pp, "crystal".to_string()).unwrap();
        assert!(results.len() >= 3);

        // Chapters should come before entities, entities before notes (by sort)
        let file_types: Vec<&str> = results.iter().map(|r| r.file_type.as_str()).collect();
        // Verify ordering: chapters first, then entities, then notes
        let first_chapter_idx = file_types.iter().position(|t| *t == "chapter");
        let first_entity_idx = file_types.iter().position(|t| *t == "entity");
        let first_note_idx = file_types.iter().position(|t| *t == "note");
        assert!(first_chapter_idx < first_entity_idx);
        assert!(first_entity_idx < first_note_idx);
    }

    #[test]
    fn search_entity_includes_entity_type() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_entity_md(
            dir.path(),
            "character",
            "bob",
            "Bob The Brave",
            "Bob is a warrior.\n",
        );

        let results = search_project(pp, "warrior".to_string()).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].file_type, "entity");
        assert_eq!(results[0].entity_type, Some("character".to_string()));
    }

    #[test]
    fn search_skips_frontmatter_body_but_matches_title() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        // The word "magic" appears only in frontmatter yaml field "slug: magic-sword"
        // but NOT in the body. It should not match on the slug line.
        write_md(
            dir.path(),
            "notes/sword.md",
            "Sword",
            "magic-sword",
            "A powerful weapon.\n",
        );

        let results = search_project(pp, "magic-sword".to_string()).unwrap();
        // Should not find anything — "magic-sword" is only in slug field, not title or body
        assert!(results.is_empty());
    }

    #[test]
    fn search_matches_title_field() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_md(
            dir.path(),
            "notes/dragon.md",
            "Dragon Lore",
            "dragon",
            "Some body text.\n",
        );

        let results = search_project(pp, "Dragon Lore".to_string()).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].title, "Dragon Lore");
    }

    #[test]
    fn search_exact_title_match_sorted_first() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_md(
            dir.path(),
            "notes/note1.md",
            "Magic",
            "magic",
            "No body match here.\n",
        );
        write_md(
            dir.path(),
            "notes/note2.md",
            "Other",
            "other",
            "Magic is everywhere in this story.\n",
        );

        let results = search_project(pp, "Magic".to_string()).unwrap();
        assert!(results.len() >= 2);
        // The exact title match ("Magic") should be first
        assert_eq!(results[0].title, "Magic");
    }

    #[test]
    fn search_context_lines() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_md(
            dir.path(),
            "notes/ctx.md",
            "Context Test",
            "ctx",
            "Line one.\nLine two with match.\nLine three.\n",
        );

        let results = search_project(pp, "match".to_string()).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].context_before, "Line one.");
        assert_eq!(results[0].context_after, "Line three.");
    }

    #[test]
    fn search_caps_at_50_results() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        // Create a note with 60 lines containing the search term
        let mut body = String::new();
        for i in 0..60 {
            body.push_str(&format!("Line {} contains searchterm here.\n", i));
        }
        write_md(dir.path(), "notes/many.md", "Many Matches", "many", &body);

        let results = search_project(pp, "searchterm".to_string()).unwrap();
        assert_eq!(results.len(), 50);
    }

    #[test]
    fn search_empty_project_returns_empty() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let results = search_project(pp, "anything".to_string()).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn search_multiple_matches_same_file() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_md(
            dir.path(),
            "notes/multi.md",
            "Multi Match",
            "multi",
            "First mention of sword.\nSecond mention of sword.\nThird mention of sword.\n",
        );

        let results = search_project(pp, "sword".to_string()).unwrap();
        assert_eq!(results.len(), 3);
        // All from same file
        assert!(results.iter().all(|r| r.slug == "multi"));
    }

    #[test]
    fn search_line_numbers_are_1_based() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        // frontmatter = 4 lines (---, title, slug, ---), then body starts at line 5
        write_md(
            dir.path(),
            "notes/ln.md",
            "LineNum",
            "ln",
            "First body line.\nSecond body line with target.\n",
        );

        let results = search_project(pp, "target".to_string()).unwrap();
        assert_eq!(results.len(), 1);
        // Line 1: ---, Line 2: title, Line 3: slug, Line 4: ---, Line 5: First body, Line 6: Second body
        assert_eq!(results[0].line_number, 6);
    }

    // ── resolve_wiki_link ─────────────────────────────────────────

    #[test]
    fn resolve_wiki_link_exact_match() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_md(dir.path(), "notes/hero.md", "The Hero", "hero", "Body.\n");

        let target = resolve_wiki_link(pp, "The Hero".to_string()).unwrap();
        assert_eq!(target.title, "The Hero");
        assert_eq!(target.slug, "hero");
        assert_eq!(target.file_type, "note");
        assert!(target.entity_type.is_none());
    }

    #[test]
    fn resolve_wiki_link_case_insensitive() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_md(dir.path(), "notes/hero.md", "The Hero", "hero", "Body.\n");

        let target = resolve_wiki_link(pp, "the hero".to_string()).unwrap();
        assert_eq!(target.title, "The Hero");
        assert_eq!(target.slug, "hero");
    }

    #[test]
    fn resolve_wiki_link_not_found() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_md(dir.path(), "notes/hero.md", "The Hero", "hero", "Body.\n");

        let result = resolve_wiki_link(pp, "Nonexistent".to_string());
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Not found") || err_msg.contains("not found"));
    }

    #[test]
    fn resolve_wiki_link_finds_chapter() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_md(
            dir.path(),
            "manuscript/ch1.md",
            "The Beginning",
            "ch1",
            "Body.\n",
        );

        let target = resolve_wiki_link(pp, "The Beginning".to_string()).unwrap();
        assert_eq!(target.file_type, "chapter");
        assert!(target.entity_type.is_none());
    }

    #[test]
    fn resolve_wiki_link_finds_entity() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_entity_md(
            dir.path(),
            "location",
            "castle",
            "The Castle",
            "A grand castle.\n",
        );

        let target = resolve_wiki_link(pp, "The Castle".to_string()).unwrap();
        assert_eq!(target.file_type, "entity");
        assert_eq!(target.entity_type, Some("location".to_string()));
        assert_eq!(target.slug, "castle");
    }

    #[test]
    fn resolve_wiki_link_empty_project() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let result = resolve_wiki_link(pp, "Anything".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn resolve_wiki_link_partial_match_does_not_resolve() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_md(dir.path(), "notes/hero.md", "The Hero", "hero", "Body.\n");

        // Partial match should NOT resolve
        let result = resolve_wiki_link(pp, "Hero".to_string());
        assert!(result.is_err());
    }

    // ── find_backlinks ────────────────────────────────────────────

    #[test]
    fn find_backlinks_single_source() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_md(dir.path(), "notes/hero.md", "The Hero", "hero", "Body.\n");
        write_md(
            dir.path(),
            "manuscript/ch1.md",
            "Chapter One",
            "ch1",
            "The story mentions [[The Hero]] in passing.\n",
        );

        let backlinks = find_backlinks(pp, "The Hero".to_string()).unwrap();
        assert_eq!(backlinks.len(), 1);
        assert_eq!(backlinks[0].title, "Chapter One");
        assert_eq!(backlinks[0].slug, "ch1");
        assert_eq!(backlinks[0].file_type, "chapter");
        assert!(backlinks[0].matching_line.contains("[[The Hero]]"));
    }

    #[test]
    fn find_backlinks_multiple_sources() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_md(dir.path(), "notes/target.md", "Target", "target", "Body.\n");
        write_md(
            dir.path(),
            "manuscript/ch1.md",
            "Chapter One",
            "ch1",
            "See [[Target]] for details.\n",
        );
        write_md(
            dir.path(),
            "notes/idea.md",
            "Idea",
            "idea",
            "Related to [[Target]] somehow.\n",
        );
        write_entity_md(
            dir.path(),
            "character",
            "bob",
            "Bob",
            "Bob links to [[Target]] as well.\n",
        );

        let backlinks = find_backlinks(pp, "Target".to_string()).unwrap();
        assert_eq!(backlinks.len(), 3);
    }

    #[test]
    fn find_backlinks_case_insensitive() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_md(dir.path(), "notes/target.md", "Target", "target", "Body.\n");
        write_md(
            dir.path(),
            "notes/ref.md",
            "Reference",
            "ref",
            "Links to [[target]] with lowercase.\n",
        );

        let backlinks = find_backlinks(pp, "Target".to_string()).unwrap();
        assert_eq!(backlinks.len(), 1);
        assert_eq!(backlinks[0].slug, "ref");
    }

    #[test]
    fn find_backlinks_no_backlinks() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_md(
            dir.path(),
            "notes/alone.md",
            "Alone",
            "alone",
            "No links here.\n",
        );

        let backlinks = find_backlinks(pp, "Alone".to_string()).unwrap();
        assert!(backlinks.is_empty());
    }

    #[test]
    fn find_backlinks_empty_project() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let backlinks = find_backlinks(pp, "Anything".to_string()).unwrap();
        assert!(backlinks.is_empty());
    }

    #[test]
    fn find_backlinks_multiple_links_in_same_file() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_md(dir.path(), "notes/target.md", "Target", "target", "Body.\n");
        write_md(
            dir.path(),
            "manuscript/ch1.md",
            "Chapter One",
            "ch1",
            "First [[Target]] mention.\nSecond [[Target]] mention.\n",
        );

        let backlinks = find_backlinks(pp, "Target".to_string()).unwrap();
        assert_eq!(backlinks.len(), 2);
        assert!(backlinks.iter().all(|b| b.slug == "ch1"));
    }

    #[test]
    fn find_backlinks_line_number_is_correct() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_md(dir.path(), "notes/target.md", "Target", "target", "Body.\n");
        write_md(
            dir.path(),
            "notes/source.md",
            "Source",
            "source",
            "Line one.\nLine two.\nLine three with [[Target]] link.\n",
        );

        let backlinks = find_backlinks(pp, "Target".to_string()).unwrap();
        assert_eq!(backlinks.len(), 1);
        // Lines: 1=---, 2=title, 3=slug, 4=---, 5=Line one, 6=Line two, 7=Line three
        assert_eq!(backlinks[0].line_number, 7);
    }

    #[test]
    fn find_backlinks_does_not_match_partial_bracket_syntax() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_md(dir.path(), "notes/target.md", "Target", "target", "Body.\n");
        write_md(
            dir.path(),
            "notes/source.md",
            "Source",
            "source",
            "This has [Target] single brackets.\nThis has Target without brackets.\n",
        );

        let backlinks = find_backlinks(pp, "Target".to_string()).unwrap();
        assert!(backlinks.is_empty());
    }

    #[test]
    fn find_backlinks_entity_includes_entity_type() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        write_md(dir.path(), "notes/target.md", "Target", "target", "Body.\n");
        write_entity_md(
            dir.path(),
            "character",
            "alice",
            "Alice",
            "Alice mentions [[Target]] here.\n",
        );

        let backlinks = find_backlinks(pp, "Target".to_string()).unwrap();
        assert_eq!(backlinks.len(), 1);
        assert_eq!(backlinks[0].file_type, "entity");
        assert_eq!(backlinks[0].entity_type, Some("character".to_string()));
    }

    // ── classify_file ─────────────────────────────────────────────

    #[test]
    fn classify_manuscript_file() {
        let project = Path::new("/project");
        let file = Path::new("/project/manuscript/ch1.md");
        let result = classify_file(project, file);
        assert_eq!(result, Some(("chapter".to_string(), None)));
    }

    #[test]
    fn classify_entity_file() {
        let project = Path::new("/project");
        let file = Path::new("/project/entities/character/alice.md");
        let result = classify_file(project, file);
        assert_eq!(
            result,
            Some(("entity".to_string(), Some("character".to_string())))
        );
    }

    #[test]
    fn classify_note_file() {
        let project = Path::new("/project");
        let file = Path::new("/project/notes/idea.md");
        let result = classify_file(project, file);
        assert_eq!(result, Some(("note".to_string(), None)));
    }

    #[test]
    fn classify_unknown_directory_returns_none() {
        let project = Path::new("/project");
        let file = Path::new("/project/random/file.md");
        let result = classify_file(project, file);
        assert!(result.is_none());
    }

    // ── parse_frontmatter ─────────────────────────────────────────

    #[test]
    fn parse_frontmatter_valid() {
        let content = "---\ntitle: \"Test\"\nslug: \"test\"\n---\nBody here.\n";
        let (fm, body_start) = parse_frontmatter(content).unwrap();
        assert_eq!(fm.title, "Test");
        assert_eq!(fm.slug, "test");
        assert_eq!(body_start, 4);
    }

    #[test]
    fn parse_frontmatter_no_delimiter() {
        let content = "No frontmatter here.\n";
        let result = parse_frontmatter(content);
        assert!(result.is_err());
    }

    #[test]
    fn parse_frontmatter_missing_closing() {
        let content = "---\ntitle: \"Test\"\nslug: \"test\"\nBody here.\n";
        let result = parse_frontmatter(content);
        assert!(result.is_err());
    }
}
