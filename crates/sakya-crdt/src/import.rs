//! Import Sakya project files into a CrdtProject.
//!
//! Walks an existing Sakya project directory and populates a CrdtProject
//! from the on-disk YAML + Markdown files.

use crate::error::CrdtError;
use crate::project::CrdtProject;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

// ── Serde types for file parsing ─────────────────────────────────────────────

#[derive(Deserialize)]
struct ManuscriptConfig {
    chapters: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChapterFrontmatter {
    title: String,
    slug: String,
    status: ChapterStatus,
    #[serde(default)]
    pov: Option<String>,
    #[serde(default)]
    synopsis: Option<String>,
    #[serde(default)]
    target_words: Option<u32>,
    #[allow(dead_code)]
    #[serde(default)]
    order: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum ChapterStatus {
    Draft,
    Revised,
    Final,
}

impl ChapterStatus {
    fn as_str(&self) -> &str {
        match self {
            Self::Draft => "draft",
            Self::Revised => "revised",
            Self::Final => "final",
        }
    }
}

#[derive(Deserialize)]
struct NotesConfig {
    #[serde(default)]
    notes: Vec<NoteConfigEntry>,
}

#[derive(Deserialize)]
struct NoteConfigEntry {
    slug: String,
    title: String,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    position: Option<CorkboardPosition>,
}

#[derive(Deserialize)]
struct CorkboardPosition {
    x: f64,
    y: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EntityFrontmatter {
    title: String,
    slug: String,
    #[allow(dead_code)]
    schema_type: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    spider_values: HashMap<String, f64>,
    #[serde(default)]
    fields: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize)]
struct SessionsConfig {
    #[serde(default)]
    sessions: Vec<SessionEntry>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionEntry {
    id: String,
    start: String,
    #[serde(default)]
    end: Option<String>,
    #[serde(default)]
    duration_minutes: Option<f64>,
    #[serde(default)]
    words_written: u32,
    chapter_slug: String,
    #[serde(default)]
    sprint_goal: Option<u32>,
}

// ── Frontmatter parsing ──────────────────────────────────────────────────────

const FM_DELIMITER: &str = "---";

struct ParsedDoc<T> {
    frontmatter: T,
    body: String,
}

fn parse_frontmatter<T: DeserializeOwned>(content: &str) -> Result<ParsedDoc<T>, CrdtError> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with(FM_DELIMITER) {
        return Err(CrdtError::Serialization(
            "Document does not start with frontmatter delimiter".to_string(),
        ));
    }

    let after_first = &trimmed[FM_DELIMITER.len()..];
    let end_pos = after_first
        .find(&format!("\n{}", FM_DELIMITER))
        .ok_or_else(|| {
            CrdtError::Serialization("Missing closing frontmatter delimiter".to_string())
        })?;

    let yaml_str = &after_first[..end_pos];
    let body_start = end_pos + 1 + FM_DELIMITER.len();
    let body = after_first[body_start..]
        .trim_start_matches('\n')
        .to_string();

    let frontmatter: T =
        serde_yaml::from_str(yaml_str).map_err(|e| CrdtError::Serialization(e.to_string()))?;

    Ok(ParsedDoc { frontmatter, body })
}

// ── Import function ──────────────────────────────────────────────────────────

/// Import a Sakya project from disk into a new CrdtProject.
///
/// Reads the project directory structure:
/// - `manuscript/manuscript.yaml` → chapter ordering
/// - `manuscript/{slug}.md` → chapter files (YAML frontmatter + Markdown body)
/// - `notes/notes.yaml` → note registry
/// - `notes/{slug}.md` → note files
/// - `entities/{schema_type}/{slug}.md` → entity instance files
/// - `.sakya/sessions.yaml` → writing sessions
pub fn import_from_files(project_path: &Path) -> Result<CrdtProject, CrdtError> {
    let project = CrdtProject::new(Uuid::new_v4());

    import_chapters(&project, project_path)?;
    import_notes(&project, project_path)?;
    import_entities(&project, project_path)?;
    import_sessions(&project, project_path)?;

    Ok(project)
}

fn import_chapters(project: &CrdtProject, project_path: &Path) -> Result<(), CrdtError> {
    let manuscript_yaml = project_path.join("manuscript/manuscript.yaml");
    if !manuscript_yaml.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&manuscript_yaml)?;
    let config: ManuscriptConfig =
        serde_yaml::from_str(&content).map_err(|e| CrdtError::Serialization(e.to_string()))?;

    for slug in &config.chapters {
        let chapter_file = project_path.join(format!("manuscript/{}.md", slug));
        if !chapter_file.exists() {
            continue;
        }

        let file_content = std::fs::read_to_string(&chapter_file)?;
        let doc: ParsedDoc<ChapterFrontmatter> = parse_frontmatter(&file_content)?;
        let fm = doc.frontmatter;

        project.import_chapter(
            &fm.slug,
            &fm.title,
            fm.status.as_str(),
            fm.pov.as_deref(),
            fm.synopsis.as_deref(),
            fm.target_words,
            &doc.body,
        )?;
    }

    Ok(())
}

fn import_notes(project: &CrdtProject, project_path: &Path) -> Result<(), CrdtError> {
    let notes_yaml = project_path.join("notes/notes.yaml");
    if !notes_yaml.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&notes_yaml)?;
    let config: NotesConfig =
        serde_yaml::from_str(&content).map_err(|e| CrdtError::Serialization(e.to_string()))?;

    for entry in &config.notes {
        // Read note body from .md file
        let note_file = project_path.join(format!("notes/{}.md", entry.slug));
        let body = if note_file.exists() {
            let file_content = std::fs::read_to_string(&note_file)?;
            // Notes may or may not have frontmatter
            if file_content.trim_start().starts_with(FM_DELIMITER) {
                match parse_frontmatter::<serde_yaml::Value>(&file_content) {
                    Ok(doc) => doc.body,
                    Err(_) => file_content,
                }
            } else {
                file_content
            }
        } else {
            String::new()
        };

        let (pos_x, pos_y) = entry
            .position
            .as_ref()
            .map(|p| (Some(p.x), Some(p.y)))
            .unwrap_or((None, None));

        project.import_note(
            &entry.slug,
            &entry.title,
            entry.color.as_deref(),
            entry.label.as_deref(),
            pos_x,
            pos_y,
            &body,
        )?;
    }

    Ok(())
}

fn import_entities(project: &CrdtProject, project_path: &Path) -> Result<(), CrdtError> {
    let entities_dir = project_path.join("entities");
    if !entities_dir.exists() {
        return Ok(());
    }

    // Walk schema directories
    let entries = std::fs::read_dir(&entities_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let schema_type = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Walk entity files in this schema directory
        let entity_entries = std::fs::read_dir(&path)?;
        for entity_entry in entity_entries {
            let entity_entry = entity_entry?;
            let entity_path = entity_entry.path();
            if entity_path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }

            let file_content = std::fs::read_to_string(&entity_path)?;
            let doc: ParsedDoc<EntityFrontmatter> = parse_frontmatter(&file_content)?;
            let fm = doc.frontmatter;

            // Build fields JSON: merge tags, spiderValues, fields, and body
            let mut fields_obj = serde_json::Map::new();

            // Add custom fields
            for (k, v) in &fm.fields {
                fields_obj.insert(k.clone(), v.clone());
            }

            // Add tags as a field
            if !fm.tags.is_empty() {
                fields_obj.insert(
                    "tags".to_string(),
                    serde_json::Value::Array(
                        fm.tags
                            .iter()
                            .map(|t| serde_json::Value::String(t.clone()))
                            .collect(),
                    ),
                );
            }

            // Add spider values as a field
            if !fm.spider_values.is_empty() {
                let sv: serde_json::Map<String, serde_json::Value> = fm
                    .spider_values
                    .iter()
                    .map(|(k, v)| {
                        (
                            k.clone(),
                            serde_json::Number::from_f64(*v)
                                .map(serde_json::Value::Number)
                                .unwrap_or(serde_json::Value::Null),
                        )
                    })
                    .collect();
                fields_obj.insert("spiderValues".to_string(), serde_json::Value::Object(sv));
            }

            // Add body as a field
            if !doc.body.is_empty() {
                fields_obj.insert("body".to_string(), serde_json::Value::String(doc.body));
            }

            project.create_entity(
                &schema_type,
                &fm.slug,
                &fm.title,
                serde_json::Value::Object(fields_obj),
            )?;
        }
    }

    Ok(())
}

fn import_sessions(project: &CrdtProject, project_path: &Path) -> Result<(), CrdtError> {
    let sessions_yaml = project_path.join(".sakya/sessions.yaml");
    if !sessions_yaml.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&sessions_yaml)?;
    let config: SessionsConfig =
        serde_yaml::from_str(&content).map_err(|e| CrdtError::Serialization(e.to_string()))?;

    for session in &config.sessions {
        project.import_session(
            &session.id,
            &session.start,
            session.end.as_deref(),
            session.duration_minutes,
            session.words_written,
            &session.chapter_slug,
            session.sprint_goal,
        )?;
    }

    Ok(())
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_project() -> TempDir {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        // Create manuscript
        std::fs::create_dir_all(root.join("manuscript")).unwrap();
        std::fs::write(
            root.join("manuscript/manuscript.yaml"),
            "chapters:\n  - chapter-one\n  - chapter-two\n",
        )
        .unwrap();

        std::fs::write(
            root.join("manuscript/chapter-one.md"),
            "---\ntitle: Chapter One\nslug: chapter-one\nstatus: draft\npov: Alice\nsynopsis: The beginning\ntargetWords: 5000\norder: 0\n---\nThe story begins here.\n\nSecond paragraph.",
        )
        .unwrap();

        std::fs::write(
            root.join("manuscript/chapter-two.md"),
            "---\ntitle: Chapter Two\nslug: chapter-two\nstatus: revised\norder: 1\n---\nThe story continues.",
        )
        .unwrap();

        // Create notes
        std::fs::create_dir_all(root.join("notes")).unwrap();
        std::fs::write(
            root.join("notes/notes.yaml"),
            "notes:\n  - slug: plot-idea\n    title: Plot Idea\n    color: '#ff0000'\n    label: important\n    position:\n      x: 10.0\n      y: 20.0\n  - slug: character-sketch\n    title: Character Sketch\n",
        )
        .unwrap();

        std::fs::write(
            root.join("notes/plot-idea.md"),
            "---\ntitle: Plot Idea\nslug: plot-idea\n---\nThe hero discovers a hidden city.",
        )
        .unwrap();

        std::fs::write(
            root.join("notes/character-sketch.md"),
            "---\ntitle: Character Sketch\nslug: character-sketch\n---\nAlice is brave and kind.",
        )
        .unwrap();

        // Create entities
        std::fs::create_dir_all(root.join("entities/character")).unwrap();
        std::fs::write(
            root.join("entities/character/alice.md"),
            "---\ntitle: Alice\nslug: alice\nschemaType: character\ntags:\n  - protagonist\n  - hero\nspiderValues:\n  Courage: 8.5\n  Resilience: 7.0\nfields:\n  role: Protagonist\n  age: 25\n---\nAlice is the main character.",
        )
        .unwrap();

        // Create sessions
        std::fs::create_dir_all(root.join(".sakya")).unwrap();
        std::fs::write(
            root.join(".sakya/sessions.yaml"),
            "sessions:\n  - id: '2026-02-14T10:30:00Z'\n    start: '2026-02-14T10:30:00Z'\n    end: '2026-02-14T11:00:00Z'\n    durationMinutes: 30.0\n    wordsWritten: 847\n    chapterSlug: chapter-one\n    sprintGoal: 500\n",
        )
        .unwrap();

        dir
    }

    #[test]
    fn import_empty_project() {
        let dir = TempDir::new().unwrap();
        let project = import_from_files(dir.path()).unwrap();
        assert!(project.list_chapters().unwrap().is_empty());
        assert!(project.list_notes().unwrap().is_empty());
    }

    #[test]
    fn import_chapters() {
        let dir = create_test_project();
        let project = import_from_files(dir.path()).unwrap();

        let chapters = project.list_chapters().unwrap();
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].slug, "chapter-one");
        assert_eq!(chapters[0].title, "Chapter One");
        assert_eq!(chapters[0].status, "draft");
        assert_eq!(chapters[1].slug, "chapter-two");
        assert_eq!(chapters[1].title, "Chapter Two");
        assert_eq!(chapters[1].status, "revised");
    }

    #[test]
    fn import_chapter_full_data() {
        let dir = create_test_project();
        let project = import_from_files(dir.path()).unwrap();

        let ch1 = project.get_chapter("chapter-one").unwrap();
        assert_eq!(ch1.title, "Chapter One");
        assert_eq!(ch1.status, "draft");
        assert_eq!(ch1.pov, Some("Alice".to_string()));
        assert_eq!(ch1.synopsis, Some("The beginning".to_string()));
        assert_eq!(ch1.target_words, Some(5000));
        assert_eq!(ch1.body, "The story begins here.\n\nSecond paragraph.");
    }

    #[test]
    fn import_chapter_minimal_data() {
        let dir = create_test_project();
        let project = import_from_files(dir.path()).unwrap();

        let ch2 = project.get_chapter("chapter-two").unwrap();
        assert_eq!(ch2.title, "Chapter Two");
        assert_eq!(ch2.status, "revised");
        assert_eq!(ch2.pov, None);
        assert_eq!(ch2.synopsis, None);
        assert_eq!(ch2.target_words, None);
        assert_eq!(ch2.body, "The story continues.");
    }

    #[test]
    fn import_notes() {
        let dir = create_test_project();
        let project = import_from_files(dir.path()).unwrap();

        let notes = project.list_notes().unwrap();
        assert_eq!(notes.len(), 2);

        // Check note with full metadata
        let note1 = project.get_note("plot-idea").unwrap();
        assert_eq!(note1.title, "Plot Idea");
        assert_eq!(note1.color, Some("#ff0000".to_string()));
        assert_eq!(note1.label, Some("important".to_string()));
        assert_eq!(note1.body, "The hero discovers a hidden city.");
    }

    #[test]
    fn import_note_position() {
        let dir = create_test_project();
        let project = import_from_files(dir.path()).unwrap();

        let (_, pos_x, pos_y) = project.get_note_full("plot-idea").unwrap();
        assert_eq!(pos_x, Some(10.0));
        assert_eq!(pos_y, Some(20.0));
    }

    #[test]
    fn import_entities() {
        let dir = create_test_project();
        let project = import_from_files(dir.path()).unwrap();

        let entities = project.list_entities("character").unwrap();
        assert_eq!(entities.len(), 1);

        let alice = project.get_entity("character", "alice").unwrap();
        assert_eq!(alice.title, "Alice");
        assert_eq!(alice.schema_type, "character");

        // Check custom fields
        let fields = alice.fields.as_object().unwrap();
        assert_eq!(fields.get("role").unwrap(), "Protagonist");
        assert_eq!(fields.get("body").unwrap(), "Alice is the main character.");

        // Check tags
        let tags = fields.get("tags").unwrap().as_array().unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0], "protagonist");
        assert_eq!(tags[1], "hero");
    }

    #[test]
    fn import_sessions() {
        let dir = create_test_project();
        let project = import_from_files(dir.path()).unwrap();

        let session_ids = project.list_session_ids();
        assert_eq!(session_ids.len(), 1);

        let session = project.get_session("2026-02-14T10:30:00Z").unwrap();
        assert_eq!(session.start, "2026-02-14T10:30:00Z");
        assert_eq!(session.end, Some("2026-02-14T11:00:00Z".to_string()));
        assert_eq!(session.duration_minutes, Some(30.0));
        assert_eq!(session.words_written, 847);
        assert_eq!(session.chapter_slug, "chapter-one");
        assert_eq!(session.sprint_goal, Some(500));
    }

    #[test]
    fn import_missing_manuscript_yaml() {
        let dir = TempDir::new().unwrap();
        // Create notes but no manuscript
        std::fs::create_dir_all(dir.path().join("notes")).unwrap();
        std::fs::write(
            dir.path().join("notes/notes.yaml"),
            "notes:\n  - slug: test\n    title: Test\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("notes/test.md"),
            "---\ntitle: Test\nslug: test\n---\nTest body.",
        )
        .unwrap();

        let project = import_from_files(dir.path()).unwrap();
        assert!(project.list_chapters().unwrap().is_empty());
        assert_eq!(project.list_notes().unwrap().len(), 1);
    }

    #[test]
    fn import_preserves_chapter_order() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        std::fs::create_dir_all(root.join("manuscript")).unwrap();
        std::fs::write(
            root.join("manuscript/manuscript.yaml"),
            "chapters:\n  - third\n  - first\n  - second\n",
        )
        .unwrap();

        for (slug, title) in &[("first", "First"), ("second", "Second"), ("third", "Third")] {
            std::fs::write(
                root.join(format!("manuscript/{}.md", slug)),
                format!(
                    "---\ntitle: {}\nslug: {}\nstatus: draft\norder: 0\n---\nBody.",
                    title, slug
                ),
            )
            .unwrap();
        }

        let project = import_from_files(dir.path()).unwrap();
        let chapters = project.list_chapters().unwrap();
        assert_eq!(chapters[0].slug, "third");
        assert_eq!(chapters[1].slug, "first");
        assert_eq!(chapters[2].slug, "second");
    }
}
