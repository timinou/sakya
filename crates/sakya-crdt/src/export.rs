//! Export CrdtProject state back to Sakya project files on disk.
//!
//! Generates the YAML + Markdown file structure from CRDT state.
//! Produces output identical to the original files for lossless round-trip.

use crate::error::CrdtError;
use crate::project::CrdtProject;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

// ── Serde types for file serialization ───────────────────────────────────────

#[derive(Serialize)]
struct ManuscriptConfig {
    chapters: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ChapterFrontmatter {
    title: String,
    slug: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pov: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    synopsis: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_words: Option<u32>,
    order: u32,
}

#[derive(Serialize)]
struct NotesConfig {
    notes: Vec<NoteConfigEntry>,
}

#[derive(Serialize)]
struct NoteConfigEntry {
    slug: String,
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    position: Option<CorkboardPosition>,
}

#[derive(Serialize)]
struct CorkboardPosition {
    x: f64,
    y: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EntityFrontmatter {
    title: String,
    slug: String,
    schema_type: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    spider_values: HashMap<String, f64>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    fields: HashMap<String, serde_json::Value>,
}

#[derive(Serialize)]
struct SessionsConfig {
    sessions: Vec<SessionConfigEntry>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionConfigEntry {
    id: String,
    start: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_minutes: Option<f64>,
    words_written: u32,
    chapter_slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sprint_goal: Option<u32>,
}

// ── Frontmatter serialization ────────────────────────────────────────────────

fn serialize_frontmatter<T: Serialize>(frontmatter: &T, body: &str) -> Result<String, CrdtError> {
    let yaml =
        serde_yaml::to_string(frontmatter).map_err(|e| CrdtError::Serialization(e.to_string()))?;
    Ok(format!("---\n{}---\n{}", yaml, body))
}

fn write_yaml<T: Serialize>(path: &Path, data: &T) -> Result<(), CrdtError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let yaml = serde_yaml::to_string(data).map_err(|e| CrdtError::Serialization(e.to_string()))?;
    std::fs::write(path, yaml)?;
    Ok(())
}

// ── Export function ──────────────────────────────────────────────────────────

/// Export a CrdtProject to the Sakya project file structure on disk.
///
/// Writes:
/// - `manuscript/manuscript.yaml` — chapter ordering
/// - `manuscript/{slug}.md` — chapter files with YAML frontmatter + body
/// - `notes/notes.yaml` — note registry
/// - `notes/{slug}.md` — note files
/// - `entities/{schema_type}/{slug}.md` — entity instance files
/// - `.sakya/sessions.yaml` — writing sessions
pub fn export_to_files(project: &CrdtProject, project_path: &Path) -> Result<(), CrdtError> {
    export_chapters(project, project_path)?;
    export_notes(project, project_path)?;
    export_entities(project, project_path)?;
    export_sessions(project, project_path)?;
    Ok(())
}

fn export_chapters(project: &CrdtProject, project_path: &Path) -> Result<(), CrdtError> {
    let chapters = project.list_chapters()?;
    if chapters.is_empty() {
        return Ok(());
    }

    let manuscript_dir = project_path.join("manuscript");
    std::fs::create_dir_all(&manuscript_dir)?;

    // Write manuscript.yaml with chapter ordering
    let config = ManuscriptConfig {
        chapters: chapters.iter().map(|c| c.slug.clone()).collect(),
    };
    write_yaml(&manuscript_dir.join("manuscript.yaml"), &config)?;

    // Write each chapter file
    for (order, summary) in chapters.iter().enumerate() {
        let chapter = project.get_chapter(&summary.slug)?;

        let frontmatter = ChapterFrontmatter {
            title: chapter.title,
            slug: chapter.slug.clone(),
            status: chapter.status,
            pov: chapter.pov,
            synopsis: chapter.synopsis,
            target_words: chapter.target_words,
            order: order as u32,
        };

        let content = serialize_frontmatter(&frontmatter, &chapter.body)?;
        std::fs::write(manuscript_dir.join(format!("{}.md", summary.slug)), content)?;
    }

    Ok(())
}

fn export_notes(project: &CrdtProject, project_path: &Path) -> Result<(), CrdtError> {
    let notes = project.list_notes()?;
    if notes.is_empty() {
        return Ok(());
    }

    let notes_dir = project_path.join("notes");
    std::fs::create_dir_all(&notes_dir)?;

    // Build notes.yaml entries
    let mut note_entries = Vec::new();
    for summary in &notes {
        let (note, pos_x, pos_y) = project.get_note_full(&summary.slug)?;

        let position = match (pos_x, pos_y) {
            (Some(x), Some(y)) => Some(CorkboardPosition { x, y }),
            _ => None,
        };

        note_entries.push(NoteConfigEntry {
            slug: note.slug.clone(),
            title: note.title.clone(),
            color: note.color.clone(),
            label: note.label.clone(),
            position,
        });

        // Write note .md file
        #[derive(Serialize)]
        struct NoteFrontmatter {
            title: String,
            slug: String,
        }

        let fm = NoteFrontmatter {
            title: note.title,
            slug: note.slug.clone(),
        };
        let content = serialize_frontmatter(&fm, &note.body)?;
        std::fs::write(notes_dir.join(format!("{}.md", summary.slug)), content)?;
    }

    write_yaml(
        &notes_dir.join("notes.yaml"),
        &NotesConfig {
            notes: note_entries,
        },
    )?;

    Ok(())
}

fn export_entities(project: &CrdtProject, project_path: &Path) -> Result<(), CrdtError> {
    let schemas = project.list_entity_schemas();
    if schemas.is_empty() {
        return Ok(());
    }

    for schema in &schemas {
        let entities = project.list_entities(schema)?;
        if entities.is_empty() {
            continue;
        }

        let schema_dir = project_path.join(format!("entities/{}", schema));
        std::fs::create_dir_all(&schema_dir)?;

        for entity in &entities {
            let fields_obj = entity.fields.as_object().cloned().unwrap_or_default();

            // Extract special fields
            let tags: Vec<String> = fields_obj
                .get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();

            let spider_values: HashMap<String, f64> = fields_obj
                .get("spiderValues")
                .and_then(|v| v.as_object())
                .map(|obj| {
                    obj.iter()
                        .filter_map(|(k, v)| v.as_f64().map(|f| (k.clone(), f)))
                        .collect()
                })
                .unwrap_or_default();

            let body = fields_obj
                .get("body")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            // Remaining fields (exclude tags, spiderValues, body)
            let custom_fields: HashMap<String, serde_json::Value> = fields_obj
                .into_iter()
                .filter(|(k, _)| k != "tags" && k != "spiderValues" && k != "body")
                .collect();

            let frontmatter = EntityFrontmatter {
                title: entity.title.clone(),
                slug: entity.slug.clone(),
                schema_type: schema.clone(),
                tags,
                spider_values,
                fields: custom_fields,
            };

            let content = serialize_frontmatter(&frontmatter, &body)?;
            std::fs::write(schema_dir.join(format!("{}.md", entity.slug)), content)?;
        }
    }

    Ok(())
}

fn export_sessions(project: &CrdtProject, project_path: &Path) -> Result<(), CrdtError> {
    let session_ids = project.list_session_ids();
    if session_ids.is_empty() {
        return Ok(());
    }

    let mut sessions = Vec::new();
    for id in &session_ids {
        let session = project.get_session(id)?;
        sessions.push(SessionConfigEntry {
            id: session.id,
            start: session.start,
            end: session.end,
            duration_minutes: session.duration_minutes,
            words_written: session.words_written,
            chapter_slug: session.chapter_slug,
            sprint_goal: session.sprint_goal,
        });
    }

    let sakya_dir = project_path.join(".sakya");
    std::fs::create_dir_all(&sakya_dir)?;

    write_yaml(
        &sakya_dir.join("sessions.yaml"),
        &SessionsConfig { sessions },
    )?;

    Ok(())
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::import_from_files;
    use tempfile::TempDir;

    fn create_populated_project() -> CrdtProject {
        let project = CrdtProject::new(uuid::Uuid::new_v4());

        // Add chapters
        project
            .import_chapter(
                "chapter-one",
                "Chapter One",
                "draft",
                Some("Alice"),
                Some("The beginning"),
                Some(5000),
                "The story begins here.\n\nSecond paragraph.",
            )
            .unwrap();

        project
            .import_chapter(
                "chapter-two",
                "Chapter Two",
                "revised",
                None,
                None,
                None,
                "The story continues.",
            )
            .unwrap();

        // Add notes
        project
            .import_note(
                "plot-idea",
                "Plot Idea",
                Some("#ff0000"),
                Some("important"),
                Some(10.0),
                Some(20.0),
                "The hero discovers a hidden city.",
            )
            .unwrap();

        // Add entity
        project
            .create_entity(
                "character",
                "alice",
                "Alice",
                serde_json::json!({
                    "role": "Protagonist",
                    "tags": ["protagonist", "hero"],
                    "body": "Alice is the main character."
                }),
            )
            .unwrap();

        // Add session
        project
            .import_session(
                "2026-02-14T10:30:00Z",
                "2026-02-14T10:30:00Z",
                Some("2026-02-14T11:00:00Z"),
                Some(30.0),
                847,
                "chapter-one",
                Some(500),
            )
            .unwrap();

        project
    }

    #[test]
    fn export_creates_manuscript_yaml() {
        let project = create_populated_project();
        let dir = TempDir::new().unwrap();
        export_to_files(&project, dir.path()).unwrap();

        let content =
            std::fs::read_to_string(dir.path().join("manuscript/manuscript.yaml")).unwrap();
        assert!(content.contains("chapter-one"));
        assert!(content.contains("chapter-two"));
    }

    #[test]
    fn export_creates_chapter_files() {
        let project = create_populated_project();
        let dir = TempDir::new().unwrap();
        export_to_files(&project, dir.path()).unwrap();

        let ch1 = std::fs::read_to_string(dir.path().join("manuscript/chapter-one.md")).unwrap();
        assert!(ch1.contains("title: Chapter One"));
        assert!(ch1.contains("slug: chapter-one"));
        assert!(ch1.contains("status: draft"));
        assert!(ch1.contains("pov: Alice"));
        assert!(ch1.contains("The story begins here."));
    }

    #[test]
    fn export_creates_notes_yaml() {
        let project = create_populated_project();
        let dir = TempDir::new().unwrap();
        export_to_files(&project, dir.path()).unwrap();

        let content = std::fs::read_to_string(dir.path().join("notes/notes.yaml")).unwrap();
        assert!(content.contains("plot-idea"));
        assert!(content.contains("Plot Idea"));
    }

    #[test]
    fn export_creates_note_files() {
        let project = create_populated_project();
        let dir = TempDir::new().unwrap();
        export_to_files(&project, dir.path()).unwrap();

        let note = std::fs::read_to_string(dir.path().join("notes/plot-idea.md")).unwrap();
        assert!(note.contains("title: Plot Idea"));
        assert!(note.contains("The hero discovers a hidden city."));
    }

    #[test]
    fn export_creates_entity_files() {
        let project = create_populated_project();
        let dir = TempDir::new().unwrap();
        export_to_files(&project, dir.path()).unwrap();

        let entity =
            std::fs::read_to_string(dir.path().join("entities/character/alice.md")).unwrap();
        assert!(entity.contains("title: Alice"));
        assert!(entity.contains("slug: alice"));
        assert!(entity.contains("Alice is the main character."));
    }

    #[test]
    fn export_creates_sessions_yaml() {
        let project = create_populated_project();
        let dir = TempDir::new().unwrap();
        export_to_files(&project, dir.path()).unwrap();

        let content = std::fs::read_to_string(dir.path().join(".sakya/sessions.yaml")).unwrap();
        assert!(content.contains("2026-02-14T10:30:00Z"));
        assert!(content.contains("wordsWritten: 847"));
        assert!(content.contains("chapter-one"));
    }

    #[test]
    fn export_empty_project_creates_nothing() {
        let project = CrdtProject::new(uuid::Uuid::new_v4());
        let dir = TempDir::new().unwrap();
        export_to_files(&project, dir.path()).unwrap();

        assert!(!dir.path().join("manuscript").exists());
        assert!(!dir.path().join("notes").exists());
        assert!(!dir.path().join("entities").exists());
        assert!(!dir.path().join(".sakya").exists());
    }

    #[test]
    fn roundtrip_import_export_chapters() {
        // Create source project on disk
        let src_dir = TempDir::new().unwrap();
        std::fs::create_dir_all(src_dir.path().join("manuscript")).unwrap();
        std::fs::write(
            src_dir.path().join("manuscript/manuscript.yaml"),
            "chapters:\n  - ch-one\n",
        )
        .unwrap();
        std::fs::write(
            src_dir.path().join("manuscript/ch-one.md"),
            "---\ntitle: Chapter One\nslug: ch-one\nstatus: draft\norder: 0\n---\nBody text here.",
        )
        .unwrap();

        // Import
        let project = import_from_files(src_dir.path()).unwrap();

        // Export to new location
        let dst_dir = TempDir::new().unwrap();
        export_to_files(&project, dst_dir.path()).unwrap();

        // Verify chapter content round-tripped
        let exported =
            std::fs::read_to_string(dst_dir.path().join("manuscript/ch-one.md")).unwrap();
        assert!(exported.contains("title: Chapter One"));
        assert!(exported.contains("slug: ch-one"));
        assert!(exported.contains("status: draft"));
        assert!(exported.contains("Body text here."));

        // Verify manuscript.yaml
        let manifest =
            std::fs::read_to_string(dst_dir.path().join("manuscript/manuscript.yaml")).unwrap();
        assert!(manifest.contains("ch-one"));
    }

    #[test]
    fn roundtrip_import_export_notes() {
        let src_dir = TempDir::new().unwrap();
        std::fs::create_dir_all(src_dir.path().join("notes")).unwrap();
        std::fs::write(
            src_dir.path().join("notes/notes.yaml"),
            "notes:\n  - slug: my-note\n    title: My Note\n    color: '#00ff00'\n",
        )
        .unwrap();
        std::fs::write(
            src_dir.path().join("notes/my-note.md"),
            "---\ntitle: My Note\nslug: my-note\n---\nNote body.",
        )
        .unwrap();

        let project = import_from_files(src_dir.path()).unwrap();

        let dst_dir = TempDir::new().unwrap();
        export_to_files(&project, dst_dir.path()).unwrap();

        let exported = std::fs::read_to_string(dst_dir.path().join("notes/my-note.md")).unwrap();
        assert!(exported.contains("title: My Note"));
        assert!(exported.contains("Note body."));
    }
}
