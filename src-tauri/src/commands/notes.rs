use std::path::PathBuf;

use crate::error::AppError;
use crate::models::notes::{NoteContent, NoteEntry, NoteFrontmatter, NotesConfig};
use crate::services::frontmatter;
use crate::services::slug_service::slugify;
use crate::services::yaml_service::{read_yaml, write_yaml};

/// Helper: path to notes directory.
fn notes_dir(project_path: &str) -> PathBuf {
    PathBuf::from(project_path).join("notes")
}

/// Helper: path to notes config YAML.
fn config_path(project_path: &str) -> PathBuf {
    notes_dir(project_path).join("notes.yaml")
}

/// Helper: path to a note Markdown file.
fn note_path(project_path: &str, slug: &str) -> PathBuf {
    notes_dir(project_path).join(format!("{}.md", slug))
}

/// Read the notes config, returning an empty config if the file doesn't exist.
#[tauri::command]
pub fn get_notes_config(project_path: String) -> Result<NotesConfig, AppError> {
    let path = config_path(&project_path);
    if !path.exists() {
        return Ok(NotesConfig { notes: vec![] });
    }
    read_yaml(&path)
}

/// Write the notes config to disk, creating the directory if needed.
#[tauri::command]
pub fn save_notes_config(project_path: String, config: NotesConfig) -> Result<(), AppError> {
    let path = config_path(&project_path);
    write_yaml(&path, &config)
}

/// Read a note file, parsing its frontmatter and body.
#[tauri::command]
pub fn get_note(project_path: String, slug: String) -> Result<NoteContent, AppError> {
    let path = note_path(&project_path, &slug);
    if !path.exists() {
        return Err(AppError::NotFound(format!("Note not found: {}", slug)));
    }

    let content = std::fs::read_to_string(&path)?;
    let doc: frontmatter::ParsedDocument<NoteFrontmatter> = frontmatter::parse(&content)?;
    let fm = doc.frontmatter;

    Ok(NoteContent {
        slug: fm.slug,
        title: fm.title,
        body: doc.body,
    })
}

/// Write a note file with the given frontmatter and body.
#[tauri::command]
pub fn save_note(
    project_path: String,
    slug: String,
    title: String,
    body: String,
) -> Result<(), AppError> {
    let dir = notes_dir(&project_path);
    std::fs::create_dir_all(&dir)?;

    let fm = NoteFrontmatter {
        title,
        slug: slug.clone(),
    };

    let path = note_path(&project_path, &slug);
    let content = frontmatter::serialize(&fm, &body)?;
    std::fs::write(&path, content)?;
    Ok(())
}

/// Create a new note: generate slug, write file, add to config, return NoteContent.
#[tauri::command]
pub fn create_note(project_path: String, title: String) -> Result<NoteContent, AppError> {
    let slug = slugify(&title);
    if slug.is_empty() {
        return Err(AppError::Validation(
            "Title must produce a non-empty slug".to_string(),
        ));
    }

    let path = note_path(&project_path, &slug);
    if path.exists() {
        return Err(AppError::AlreadyExists(format!(
            "Note already exists: {}",
            slug
        )));
    }

    // Save the note file
    save_note(
        project_path.clone(),
        slug.clone(),
        title.clone(),
        String::new(),
    )?;

    // Update config
    let mut config = get_notes_config(project_path.clone())?;
    config.notes.push(NoteEntry {
        slug: slug.clone(),
        title: title.clone(),
        color: None,
        label: None,
        position: None,
    });
    save_notes_config(project_path, config)?;

    Ok(NoteContent {
        slug,
        title,
        body: String::new(),
    })
}

/// Delete a note file and remove it from the notes config.
#[tauri::command]
pub fn delete_note(project_path: String, slug: String) -> Result<(), AppError> {
    let path = note_path(&project_path, &slug);
    if !path.exists() {
        return Err(AppError::NotFound(format!("Note not found: {}", slug)));
    }

    std::fs::remove_file(&path)?;

    // Remove from config
    let mut config = get_notes_config(project_path.clone())?;
    config.notes.retain(|n| n.slug != slug);
    save_notes_config(project_path, config)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::notes::CorkboardPosition;
    use crate::test_helpers::setup_test_dir;

    // ── get_notes_config ──────────────────────────────────────────

    #[test]
    fn get_config_missing_dir_returns_empty() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let config = get_notes_config(pp).unwrap();
        assert!(config.notes.is_empty());
    }

    #[test]
    fn get_config_empty_dir_returns_empty() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();
        std::fs::create_dir_all(dir.path().join("notes")).unwrap();

        let config = get_notes_config(pp).unwrap();
        assert!(config.notes.is_empty());
    }

    #[test]
    fn get_config_reads_existing() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let config = NotesConfig {
            notes: vec![
                NoteEntry {
                    slug: "idea-one".to_string(),
                    title: "Idea One".to_string(),
                    color: Some("#ff0000".to_string()),
                    label: Some("important".to_string()),
                    position: Some(CorkboardPosition { x: 10.0, y: 20.0 }),
                },
                NoteEntry {
                    slug: "idea-two".to_string(),
                    title: "Idea Two".to_string(),
                    color: None,
                    label: None,
                    position: None,
                },
            ],
        };
        let path = dir.path().join("notes/notes.yaml");
        write_yaml(&path, &config).unwrap();

        let loaded = get_notes_config(pp).unwrap();
        assert_eq!(loaded.notes.len(), 2);
        assert_eq!(loaded.notes[0].slug, "idea-one");
        assert_eq!(loaded.notes[0].title, "Idea One");
        assert_eq!(loaded.notes[0].color, Some("#ff0000".to_string()));
        assert_eq!(loaded.notes[0].label, Some("important".to_string()));
        assert!(loaded.notes[0].position.is_some());
        let pos = loaded.notes[0].position.as_ref().unwrap();
        assert!((pos.x - 10.0).abs() < f64::EPSILON);
        assert!((pos.y - 20.0).abs() < f64::EPSILON);
        assert_eq!(loaded.notes[1].slug, "idea-two");
        assert!(loaded.notes[1].color.is_none());
        assert!(loaded.notes[1].position.is_none());
    }

    // ── save_notes_config ─────────────────────────────────────────

    #[test]
    fn save_config_creates_file() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let config = NotesConfig {
            notes: vec![NoteEntry {
                slug: "first".to_string(),
                title: "First".to_string(),
                color: None,
                label: None,
                position: None,
            }],
        };
        save_notes_config(pp.clone(), config).unwrap();

        assert!(dir.path().join("notes/notes.yaml").exists());

        let loaded = get_notes_config(pp).unwrap();
        assert_eq!(loaded.notes.len(), 1);
        assert_eq!(loaded.notes[0].slug, "first");
    }

    #[test]
    fn save_config_round_trips() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let config = NotesConfig {
            notes: vec![
                NoteEntry {
                    slug: "a".to_string(),
                    title: "Note A".to_string(),
                    color: Some("blue".to_string()),
                    label: Some("label-a".to_string()),
                    position: Some(CorkboardPosition { x: 100.5, y: 200.3 }),
                },
                NoteEntry {
                    slug: "b".to_string(),
                    title: "Note B".to_string(),
                    color: None,
                    label: None,
                    position: None,
                },
            ],
        };
        save_notes_config(pp.clone(), config.clone()).unwrap();

        let loaded = get_notes_config(pp).unwrap();
        assert_eq!(loaded.notes.len(), 2);
        assert_eq!(loaded.notes[0].slug, config.notes[0].slug);
        assert_eq!(loaded.notes[0].color, config.notes[0].color);
        assert_eq!(loaded.notes[1].slug, config.notes[1].slug);
        assert_eq!(loaded.notes[1].color, config.notes[1].color);
    }

    #[test]
    fn save_config_overwrites_existing() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let config1 = NotesConfig {
            notes: vec![NoteEntry {
                slug: "old".to_string(),
                title: "Old".to_string(),
                color: None,
                label: None,
                position: None,
            }],
        };
        save_notes_config(pp.clone(), config1).unwrap();

        let config2 = NotesConfig {
            notes: vec![
                NoteEntry {
                    slug: "new-a".to_string(),
                    title: "New A".to_string(),
                    color: None,
                    label: None,
                    position: None,
                },
                NoteEntry {
                    slug: "new-b".to_string(),
                    title: "New B".to_string(),
                    color: None,
                    label: None,
                    position: None,
                },
            ],
        };
        save_notes_config(pp.clone(), config2).unwrap();

        let loaded = get_notes_config(pp).unwrap();
        assert_eq!(loaded.notes.len(), 2);
        assert_eq!(loaded.notes[0].slug, "new-a");
        assert_eq!(loaded.notes[1].slug, "new-b");
    }

    // ── create_note ───────────────────────────────────────────────

    #[test]
    fn create_note_creates_file_and_updates_config() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let result = create_note(pp.clone(), "My First Note".to_string()).unwrap();

        assert_eq!(result.slug, "my-first-note");
        assert_eq!(result.title, "My First Note");
        assert!(result.body.is_empty());

        // Verify file on disk
        assert!(dir.path().join("notes/my-first-note.md").exists());

        // Verify config updated
        let config = get_notes_config(pp).unwrap();
        assert_eq!(config.notes.len(), 1);
        assert_eq!(config.notes[0].slug, "my-first-note");
        assert_eq!(config.notes[0].title, "My First Note");
        assert!(config.notes[0].color.is_none());
        assert!(config.notes[0].label.is_none());
        assert!(config.notes[0].position.is_none());
    }

    #[test]
    fn create_note_multiple_notes() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        create_note(pp.clone(), "Note One".to_string()).unwrap();
        create_note(pp.clone(), "Note Two".to_string()).unwrap();
        create_note(pp.clone(), "Note Three".to_string()).unwrap();

        let config = get_notes_config(pp).unwrap();
        assert_eq!(config.notes.len(), 3);
        assert_eq!(config.notes[0].slug, "note-one");
        assert_eq!(config.notes[1].slug, "note-two");
        assert_eq!(config.notes[2].slug, "note-three");
    }

    #[test]
    fn create_note_duplicate_title_returns_already_exists() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        create_note(pp.clone(), "Duplicate".to_string()).unwrap();
        let result = create_note(pp, "Duplicate".to_string());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Already exists"),
            "Expected 'Already exists' error, got: {}",
            err_msg
        );
    }

    #[test]
    fn create_note_special_characters_in_title() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let result = create_note(pp.clone(), "The Hero's Journey & Beyond!".to_string()).unwrap();

        assert_eq!(result.slug, "the-hero-s-journey-beyond");
        assert_eq!(result.title, "The Hero's Journey & Beyond!");
        assert!(dir
            .path()
            .join("notes/the-hero-s-journey-beyond.md")
            .exists());
    }

    #[test]
    fn create_note_creates_notes_directory() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        assert!(!dir.path().join("notes").exists());

        create_note(pp, "First Note".to_string()).unwrap();

        assert!(dir.path().join("notes").exists());
    }

    #[test]
    fn create_note_empty_title_returns_validation_error() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let result = create_note(pp, "".to_string());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Validation") || err_msg.contains("slug"),
            "Expected validation error, got: {}",
            err_msg
        );
    }

    // ── get_note ──────────────────────────────────────────────────

    #[test]
    fn get_note_reads_created_note() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        create_note(pp.clone(), "Readable".to_string()).unwrap();

        let note = get_note(pp, "readable".to_string()).unwrap();
        assert_eq!(note.slug, "readable");
        assert_eq!(note.title, "Readable");
        assert!(note.body.is_empty());
    }

    #[test]
    fn get_note_nonexistent_returns_not_found() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let result = get_note(pp, "nonexistent".to_string());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not found") || err_msg.contains("Not found"),
            "Expected 'not found' error, got: {}",
            err_msg
        );
    }

    #[test]
    fn get_note_reads_body_and_frontmatter() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        // Create note then update it with body
        create_note(pp.clone(), "Detailed".to_string()).unwrap();

        let body = "Some important note content.\n\nWith multiple paragraphs.\n";
        save_note(
            pp.clone(),
            "detailed".to_string(),
            "Detailed".to_string(),
            body.to_string(),
        )
        .unwrap();

        let loaded = get_note(pp, "detailed".to_string()).unwrap();
        assert_eq!(loaded.title, "Detailed");
        assert_eq!(loaded.slug, "detailed");
        assert_eq!(
            loaded.body,
            "Some important note content.\n\nWith multiple paragraphs.\n"
        );
    }

    // ── save_note ─────────────────────────────────────────────────

    #[test]
    fn save_note_updates_fields_and_body() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        create_note(pp.clone(), "Editable".to_string()).unwrap();

        save_note(
            pp.clone(),
            "editable".to_string(),
            "Editable Note (Updated)".to_string(),
            "Updated body content.\n".to_string(),
        )
        .unwrap();

        let loaded = get_note(pp, "editable".to_string()).unwrap();
        assert_eq!(loaded.title, "Editable Note (Updated)");
        assert_eq!(loaded.body, "Updated body content.\n");
    }

    #[test]
    fn save_note_round_trips_frontmatter() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        save_note(
            pp.clone(),
            "round-trip".to_string(),
            "Round Trip Test".to_string(),
            "Body here.\n".to_string(),
        )
        .unwrap();

        let loaded = get_note(pp, "round-trip".to_string()).unwrap();
        assert_eq!(loaded.slug, "round-trip");
        assert_eq!(loaded.title, "Round Trip Test");
        assert_eq!(loaded.body, "Body here.\n");
    }

    #[test]
    fn save_note_empty_body() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        save_note(
            pp.clone(),
            "empty-body".to_string(),
            "Empty Body".to_string(),
            String::new(),
        )
        .unwrap();

        let loaded = get_note(pp, "empty-body".to_string()).unwrap();
        assert!(loaded.body.is_empty());
    }

    #[test]
    fn save_note_creates_notes_directory() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        assert!(!dir.path().join("notes").exists());

        save_note(pp, "first".to_string(), "First".to_string(), String::new()).unwrap();

        assert!(dir.path().join("notes").exists());
        assert!(dir.path().join("notes/first.md").exists());
    }

    #[test]
    fn save_note_preserves_multiline_body() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let body = "# Heading\n\nParagraph one.\n\n## Subheading\n\n- Item 1\n- Item 2\n- Item 3\n\nFinal paragraph.\n";
        save_note(
            pp.clone(),
            "multiline".to_string(),
            "Multiline".to_string(),
            body.to_string(),
        )
        .unwrap();

        let loaded = get_note(pp, "multiline".to_string()).unwrap();
        assert_eq!(loaded.body, body);
    }

    // ── delete_note ───────────────────────────────────────────────

    #[test]
    fn delete_note_removes_file_and_config_entry() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        create_note(pp.clone(), "Doomed".to_string()).unwrap();
        create_note(pp.clone(), "Survivor".to_string()).unwrap();

        assert!(dir.path().join("notes/doomed.md").exists());

        delete_note(pp.clone(), "doomed".to_string()).unwrap();

        assert!(!dir.path().join("notes/doomed.md").exists());

        let config = get_notes_config(pp).unwrap();
        assert_eq!(config.notes.len(), 1);
        assert_eq!(config.notes[0].slug, "survivor");
    }

    #[test]
    fn delete_note_nonexistent_returns_not_found() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let result = delete_note(pp, "ghost".to_string());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not found") || err_msg.contains("Not found"),
            "Expected 'not found' error, got: {}",
            err_msg
        );
    }

    #[test]
    fn delete_note_last_note_leaves_empty_config() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        create_note(pp.clone(), "Only One".to_string()).unwrap();
        delete_note(pp.clone(), "only-one".to_string()).unwrap();

        let config = get_notes_config(pp).unwrap();
        assert!(config.notes.is_empty());
    }

    // ── Config with CorkboardPosition ─────────────────────────────

    #[test]
    fn config_round_trips_corkboard_positions() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let config = NotesConfig {
            notes: vec![
                NoteEntry {
                    slug: "positioned".to_string(),
                    title: "Positioned Note".to_string(),
                    color: Some("#abcdef".to_string()),
                    label: Some("plot".to_string()),
                    position: Some(CorkboardPosition { x: 42.5, y: 99.9 }),
                },
                NoteEntry {
                    slug: "unpositioned".to_string(),
                    title: "Unpositioned Note".to_string(),
                    color: None,
                    label: None,
                    position: None,
                },
            ],
        };
        save_notes_config(pp.clone(), config).unwrap();

        let loaded = get_notes_config(pp).unwrap();
        assert_eq!(loaded.notes.len(), 2);

        let pos = loaded.notes[0].position.as_ref().unwrap();
        assert!((pos.x - 42.5).abs() < f64::EPSILON);
        assert!((pos.y - 99.9).abs() < f64::EPSILON);
        assert_eq!(loaded.notes[0].color, Some("#abcdef".to_string()));
        assert_eq!(loaded.notes[0].label, Some("plot".to_string()));

        assert!(loaded.notes[1].position.is_none());
        assert!(loaded.notes[1].color.is_none());
        assert!(loaded.notes[1].label.is_none());
    }

    #[test]
    fn config_with_negative_positions() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let config = NotesConfig {
            notes: vec![NoteEntry {
                slug: "negative".to_string(),
                title: "Negative Pos".to_string(),
                color: None,
                label: None,
                position: Some(CorkboardPosition {
                    x: -100.0,
                    y: -50.5,
                }),
            }],
        };
        save_notes_config(pp.clone(), config).unwrap();

        let loaded = get_notes_config(pp).unwrap();
        let pos = loaded.notes[0].position.as_ref().unwrap();
        assert!((pos.x - (-100.0)).abs() < f64::EPSILON);
        assert!((pos.y - (-50.5)).abs() < f64::EPSILON);
    }

    // ── Integration / multi-step scenarios ─────────────────────────

    #[test]
    fn full_lifecycle_create_edit_delete() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        // Create three notes
        create_note(pp.clone(), "Plot Idea".to_string()).unwrap();
        create_note(pp.clone(), "Character Sketch".to_string()).unwrap();
        create_note(pp.clone(), "World Building".to_string()).unwrap();

        // Verify initial state
        let config = get_notes_config(pp.clone()).unwrap();
        assert_eq!(config.notes.len(), 3);
        assert_eq!(config.notes[0].slug, "plot-idea");
        assert_eq!(config.notes[1].slug, "character-sketch");
        assert_eq!(config.notes[2].slug, "world-building");

        // Edit a note with body content
        save_note(
            pp.clone(),
            "character-sketch".to_string(),
            "Character Sketch".to_string(),
            "Alice is a curious explorer.\n\nShe carries a silver compass.\n".to_string(),
        )
        .unwrap();

        // Read back the edited note
        let note = get_note(pp.clone(), "character-sketch".to_string()).unwrap();
        assert_eq!(note.title, "Character Sketch");
        assert!(note.body.contains("silver compass"));

        // Delete one note
        delete_note(pp.clone(), "plot-idea".to_string()).unwrap();

        let config = get_notes_config(pp.clone()).unwrap();
        assert_eq!(config.notes.len(), 2);
        assert_eq!(config.notes[0].slug, "character-sketch");
        assert_eq!(config.notes[1].slug, "world-building");
        assert!(!dir.path().join("notes/plot-idea.md").exists());

        // Remaining notes still accessible
        assert!(get_note(pp.clone(), "character-sketch".to_string()).is_ok());
        assert!(get_note(pp.clone(), "world-building".to_string()).is_ok());

        // Delete all remaining notes
        delete_note(pp.clone(), "character-sketch".to_string()).unwrap();
        delete_note(pp.clone(), "world-building".to_string()).unwrap();

        let config = get_notes_config(pp).unwrap();
        assert!(config.notes.is_empty());
    }

    #[test]
    fn create_after_delete_reuses_slug() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        create_note(pp.clone(), "Reusable".to_string()).unwrap();
        delete_note(pp.clone(), "reusable".to_string()).unwrap();

        // Should be able to recreate with the same title
        let result = create_note(pp.clone(), "Reusable".to_string()).unwrap();
        assert_eq!(result.slug, "reusable");
        assert_eq!(result.title, "Reusable");

        let config = get_notes_config(pp).unwrap();
        assert_eq!(config.notes.len(), 1);
        assert_eq!(config.notes[0].slug, "reusable");
    }

    #[test]
    fn save_note_for_nonexistent_slug_creates_file() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        // save_note can write to any slug, even without create_note
        save_note(
            pp.clone(),
            "manual".to_string(),
            "Manual Note".to_string(),
            "Created directly.\n".to_string(),
        )
        .unwrap();

        let loaded = get_note(pp, "manual".to_string()).unwrap();
        assert_eq!(loaded.title, "Manual Note");
        assert_eq!(loaded.body, "Created directly.\n");
    }

    #[test]
    fn note_body_with_frontmatter_like_content() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        // Body that contains --- which could confuse frontmatter parsing
        let body = "Some text with --- dashes in it.\n\nAnother --- line.\n";
        save_note(
            pp.clone(),
            "tricky".to_string(),
            "Tricky".to_string(),
            body.to_string(),
        )
        .unwrap();

        let loaded = get_note(pp, "tricky".to_string()).unwrap();
        assert_eq!(loaded.body, body);
    }

    #[test]
    fn note_with_unicode_body() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let body = "This note contains unicode: \u{00e9}\u{00e8}\u{00ea}\u{00eb}, \u{4e16}\u{754c}, \u{1f600}\n";
        save_note(
            pp.clone(),
            "unicode".to_string(),
            "Unicode Note".to_string(),
            body.to_string(),
        )
        .unwrap();

        let loaded = get_note(pp, "unicode".to_string()).unwrap();
        assert_eq!(loaded.body, body);
    }
}
