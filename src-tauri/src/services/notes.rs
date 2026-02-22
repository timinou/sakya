use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::models::notes::{NoteContent, NoteEntry, NoteFrontmatter, NotesConfig};
use crate::services::frontmatter;
use crate::services::slug_service::slugify;
use crate::services::yaml_service::{read_yaml, write_yaml};

/// Path to notes directory within the given base path.
pub fn notes_dir(base_path: &Path) -> PathBuf {
    base_path.join("notes")
}

/// Path to notes config YAML within the given base path.
pub fn config_path(base_path: &Path) -> PathBuf {
    notes_dir(base_path).join("notes.yaml")
}

/// Path to a note Markdown file within the given base path.
pub fn note_path(base_path: &Path, slug: &str) -> PathBuf {
    notes_dir(base_path).join(format!("{}.md", slug))
}

/// Read the notes config, returning an empty config if the file doesn't exist.
pub fn get_notes_config(base_path: &Path) -> Result<NotesConfig, AppError> {
    let path = config_path(base_path);
    if !path.exists() {
        return Ok(NotesConfig { notes: vec![] });
    }
    read_yaml(&path)
}

/// Write the notes config to disk, creating the directory if needed.
pub fn save_notes_config(base_path: &Path, config: &NotesConfig) -> Result<(), AppError> {
    let path = config_path(base_path);
    write_yaml(&path, config)
}

/// Read a note file, parsing its frontmatter and body.
pub fn get_note(base_path: &Path, slug: &str) -> Result<NoteContent, AppError> {
    let path = note_path(base_path, slug);
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
pub fn save_note(base_path: &Path, slug: &str, title: &str, body: &str) -> Result<(), AppError> {
    let dir = notes_dir(base_path);
    std::fs::create_dir_all(&dir)?;

    let fm = NoteFrontmatter {
        title: title.to_string(),
        slug: slug.to_string(),
    };

    let path = note_path(base_path, slug);
    let content = frontmatter::serialize(&fm, body)?;
    std::fs::write(&path, content)?;
    Ok(())
}

/// Create a new note: generate slug, write file, add to config, return NoteContent.
pub fn create_note(base_path: &Path, title: &str) -> Result<NoteContent, AppError> {
    let slug = slugify(title);
    if slug.is_empty() {
        return Err(AppError::Validation(
            "Title must produce a non-empty slug".to_string(),
        ));
    }

    let path = note_path(base_path, &slug);
    if path.exists() {
        return Err(AppError::AlreadyExists(format!(
            "Note already exists: {}",
            slug
        )));
    }

    // Save the note file
    save_note(base_path, &slug, title, "")?;

    // Update config
    let mut config = get_notes_config(base_path)?;
    config.notes.push(NoteEntry {
        slug: slug.clone(),
        title: title.to_string(),
        color: None,
        label: None,
        position: None,
        size: None,
    });
    save_notes_config(base_path, &config)?;

    Ok(NoteContent {
        slug,
        title: title.to_string(),
        body: String::new(),
    })
}

/// Delete a note file and remove it from the notes config.
pub fn delete_note(base_path: &Path, slug: &str) -> Result<(), AppError> {
    let path = note_path(base_path, slug);
    if !path.exists() {
        return Err(AppError::NotFound(format!("Note not found: {}", slug)));
    }

    std::fs::remove_file(&path)?;

    // Remove from config
    let mut config = get_notes_config(base_path)?;
    config.notes.retain(|n| n.slug != slug);
    save_notes_config(base_path, &config)?;

    Ok(())
}

/// Rename a note: update its title (and slug/filename if the slug changes).
pub fn rename_note(base_path: &Path, slug: &str, new_title: &str) -> Result<NoteContent, AppError> {
    let existing = get_note(base_path, slug)?;
    let new_slug = slugify(new_title);

    if new_slug.is_empty() {
        return Err(AppError::Validation(
            "Title must produce a non-empty slug".to_string(),
        ));
    }

    let body = existing.body;

    if new_slug == slug {
        // Same slug — just update the title in place
        save_note(base_path, slug, new_title, &body)?;

        // Update the notes config title
        let mut config = get_notes_config(base_path)?;
        if let Some(entry) = config.notes.iter_mut().find(|n| n.slug == slug) {
            entry.title = new_title.to_string();
        }
        save_notes_config(base_path, &config)?;

        return Ok(NoteContent {
            slug: slug.to_string(),
            title: new_title.to_string(),
            body,
        });
    }

    // Different slug — write new file, delete old, update config
    save_note(base_path, &new_slug, new_title, &body)?;

    // Delete the old file
    let old_path = note_path(base_path, slug);
    std::fs::remove_file(&old_path)?;

    // Update the notes config: replace old slug/title with new
    let mut config = get_notes_config(base_path)?;
    if let Some(entry) = config.notes.iter_mut().find(|n| n.slug == slug) {
        entry.slug = new_slug.clone();
        entry.title = new_title.to_string();
    }
    save_notes_config(base_path, &config)?;

    Ok(NoteContent {
        slug: new_slug,
        title: new_title.to_string(),
        body,
    })
}

/// Generate a unique slug for a note, adding a numeric suffix if needed.
fn unique_slug(base_path: &Path, base_slug: &str) -> String {
    let path = note_path(base_path, base_slug);
    if !path.exists() {
        return base_slug.to_string();
    }

    let mut counter = 2;
    loop {
        let candidate = format!("{}-{}", base_slug, counter);
        let candidate_path = note_path(base_path, &candidate);
        if !candidate_path.exists() {
            return candidate;
        }
        counter += 1;
    }
}

/// Copy a note from one base path to another.
/// Returns the (possibly modified) slug used in the destination.
pub fn copy_note(src_base: &Path, dst_base: &Path, slug: &str) -> Result<NoteContent, AppError> {
    // Read source note content
    let source = get_note(src_base, slug)?;

    // Read source config entry for metadata (color, label, position, size)
    let src_config = get_notes_config(src_base)?;
    let src_entry = src_config.notes.iter().find(|n| n.slug == slug);

    // Determine destination slug (handle collisions)
    let dest_slug = unique_slug(dst_base, &slugify(&source.title));

    // Write note file to destination
    save_note(dst_base, &dest_slug, &source.title, &source.body)?;

    // Add entry to destination config
    let mut dst_config = get_notes_config(dst_base)?;
    dst_config.notes.push(NoteEntry {
        slug: dest_slug.clone(),
        title: source.title.clone(),
        color: src_entry.and_then(|e| e.color.clone()),
        label: src_entry.and_then(|e| e.label.clone()),
        position: src_entry.and_then(|e| e.position.clone()),
        size: src_entry.and_then(|e| e.size.clone()),
    });
    save_notes_config(dst_base, &dst_config)?;

    Ok(NoteContent {
        slug: dest_slug,
        title: source.title,
        body: source.body,
    })
}

/// Move a note from one base path to another (copy then delete from source).
/// Returns the (possibly modified) slug used in the destination.
pub fn move_note(src_base: &Path, dst_base: &Path, slug: &str) -> Result<NoteContent, AppError> {
    let result = copy_note(src_base, dst_base, slug)?;
    delete_note(src_base, slug)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::notes::CorkboardPosition;
    use crate::test_helpers::setup_test_dir;

    // ── Service functions work identically to the old commands ─────

    #[test]
    fn get_config_missing_dir_returns_empty() {
        let dir = setup_test_dir();
        let config = get_notes_config(dir.path()).unwrap();
        assert!(config.notes.is_empty());
    }

    #[test]
    fn get_config_reads_existing() {
        let dir = setup_test_dir();
        let config = NotesConfig {
            notes: vec![NoteEntry {
                slug: "idea-one".to_string(),
                title: "Idea One".to_string(),
                color: Some("#ff0000".to_string()),
                label: Some("important".to_string()),
                position: Some(CorkboardPosition { x: 10.0, y: 20.0 }),
                size: None,
            }],
        };
        write_yaml(&config_path(dir.path()), &config).unwrap();

        let loaded = get_notes_config(dir.path()).unwrap();
        assert_eq!(loaded.notes.len(), 1);
        assert_eq!(loaded.notes[0].slug, "idea-one");
    }

    #[test]
    fn create_and_get_note_round_trip() {
        let dir = setup_test_dir();
        let result = create_note(dir.path(), "My First Note").unwrap();
        assert_eq!(result.slug, "my-first-note");
        assert_eq!(result.title, "My First Note");
        assert!(result.body.is_empty());

        let loaded = get_note(dir.path(), "my-first-note").unwrap();
        assert_eq!(loaded.title, "My First Note");
    }

    #[test]
    fn save_and_read_note_body() {
        let dir = setup_test_dir();
        save_note(dir.path(), "test", "Test", "Hello world.\n").unwrap();
        let loaded = get_note(dir.path(), "test").unwrap();
        assert_eq!(loaded.body, "Hello world.\n");
    }

    #[test]
    fn delete_note_removes_file_and_config() {
        let dir = setup_test_dir();
        create_note(dir.path(), "Doomed").unwrap();
        delete_note(dir.path(), "doomed").unwrap();
        let config = get_notes_config(dir.path()).unwrap();
        assert!(config.notes.is_empty());
        assert!(!note_path(dir.path(), "doomed").exists());
    }

    #[test]
    fn rename_note_with_slug_change() {
        let dir = setup_test_dir();
        create_note(dir.path(), "Plot Ideas").unwrap();
        let renamed = rename_note(dir.path(), "plot-ideas", "Story Arcs").unwrap();
        assert_eq!(renamed.slug, "story-arcs");
        assert!(!note_path(dir.path(), "plot-ideas").exists());
        assert!(note_path(dir.path(), "story-arcs").exists());
    }

    // ── copy_note tests ──────────────────────────────────────────

    #[test]
    fn copy_note_basic() {
        let src = setup_test_dir();
        let dst = setup_test_dir();

        create_note(src.path(), "My Note").unwrap();
        save_note(src.path(), "my-note", "My Note", "Content here.\n").unwrap();

        // Set metadata on source
        let mut config = get_notes_config(src.path()).unwrap();
        config.notes[0].color = Some("blue".to_string());
        config.notes[0].label = Some("important".to_string());
        save_notes_config(src.path(), &config).unwrap();

        let result = copy_note(src.path(), dst.path(), "my-note").unwrap();
        assert_eq!(result.slug, "my-note");
        assert_eq!(result.title, "My Note");
        assert_eq!(result.body, "Content here.\n");

        // Source still exists
        assert!(note_path(src.path(), "my-note").exists());

        // Destination has note + config
        let dst_note = get_note(dst.path(), "my-note").unwrap();
        assert_eq!(dst_note.body, "Content here.\n");

        let dst_config = get_notes_config(dst.path()).unwrap();
        assert_eq!(dst_config.notes.len(), 1);
        assert_eq!(dst_config.notes[0].color, Some("blue".to_string()));
        assert_eq!(dst_config.notes[0].label, Some("important".to_string()));
    }

    #[test]
    fn copy_note_slug_collision() {
        let src = setup_test_dir();
        let dst = setup_test_dir();

        create_note(src.path(), "My Note").unwrap();
        create_note(dst.path(), "My Note").unwrap(); // collision!

        let result = copy_note(src.path(), dst.path(), "my-note").unwrap();
        assert_eq!(result.slug, "my-note-2"); // auto-suffixed

        let dst_config = get_notes_config(dst.path()).unwrap();
        assert_eq!(dst_config.notes.len(), 2);
    }

    #[test]
    fn copy_note_slug_collision_multiple() {
        let src = setup_test_dir();
        let dst = setup_test_dir();

        create_note(src.path(), "My Note").unwrap();
        create_note(dst.path(), "My Note").unwrap();
        // Create my-note-2 in destination to force -3
        save_note(dst.path(), "my-note-2", "My Note 2", "").unwrap();

        let result = copy_note(src.path(), dst.path(), "my-note").unwrap();
        assert_eq!(result.slug, "my-note-3");
    }

    #[test]
    fn copy_note_nonexistent_source() {
        let src = setup_test_dir();
        let dst = setup_test_dir();

        let result = copy_note(src.path(), dst.path(), "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn copy_note_auto_creates_destination_dir() {
        let src = setup_test_dir();
        let dst = setup_test_dir();
        // Don't create notes dir in destination

        create_note(src.path(), "Test").unwrap();
        let result = copy_note(src.path(), dst.path(), "test").unwrap();
        assert_eq!(result.slug, "test");
        assert!(notes_dir(dst.path()).exists());
    }

    // ── move_note tests ──────────────────────────────────────────

    #[test]
    fn move_note_basic() {
        let src = setup_test_dir();
        let dst = setup_test_dir();

        create_note(src.path(), "Moving Note").unwrap();
        save_note(src.path(), "moving-note", "Moving Note", "Body.\n").unwrap();

        let result = move_note(src.path(), dst.path(), "moving-note").unwrap();
        assert_eq!(result.slug, "moving-note");
        assert_eq!(result.body, "Body.\n");

        // Source should be gone
        assert!(!note_path(src.path(), "moving-note").exists());
        let src_config = get_notes_config(src.path()).unwrap();
        assert!(src_config.notes.is_empty());

        // Destination should have it
        let dst_note = get_note(dst.path(), "moving-note").unwrap();
        assert_eq!(dst_note.body, "Body.\n");
    }

    #[test]
    fn move_note_with_slug_collision() {
        let src = setup_test_dir();
        let dst = setup_test_dir();

        create_note(src.path(), "Shared Name").unwrap();
        create_note(dst.path(), "Shared Name").unwrap();

        let result = move_note(src.path(), dst.path(), "shared-name").unwrap();
        assert_eq!(result.slug, "shared-name-2");

        // Source gone
        assert!(!note_path(src.path(), "shared-name").exists());
    }

    #[test]
    fn move_note_preserves_metadata() {
        let src = setup_test_dir();
        let dst = setup_test_dir();

        create_note(src.path(), "Metadata Note").unwrap();
        let mut config = get_notes_config(src.path()).unwrap();
        config.notes[0].color = Some("red".to_string());
        config.notes[0].position = Some(CorkboardPosition { x: 42.0, y: 99.0 });
        save_notes_config(src.path(), &config).unwrap();

        move_note(src.path(), dst.path(), "metadata-note").unwrap();

        let dst_config = get_notes_config(dst.path()).unwrap();
        assert_eq!(dst_config.notes[0].color, Some("red".to_string()));
        let pos = dst_config.notes[0].position.as_ref().unwrap();
        assert!((pos.x - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn unique_slug_no_collision() {
        let dir = setup_test_dir();
        assert_eq!(unique_slug(dir.path(), "new-note"), "new-note");
    }

    #[test]
    fn unique_slug_with_collision() {
        let dir = setup_test_dir();
        create_note(dir.path(), "Test Note").unwrap();
        assert_eq!(unique_slug(dir.path(), "test-note"), "test-note-2");
    }
}
