use std::path::PathBuf;

use crate::error::AppError;
use crate::models::manuscript::{
    Chapter, ChapterContent, ChapterFrontmatter, ChapterStatus, ManuscriptConfig,
};
use crate::services::frontmatter;
use crate::services::slug_service::slugify;
use crate::services::yaml_service::{read_yaml, write_yaml};

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

/// Read the manuscript config, returning an empty config if the file doesn't exist.
#[tauri::command]
pub fn get_manuscript_config(project_path: String) -> Result<ManuscriptConfig, AppError> {
    let path = config_path(&project_path);
    if !path.exists() {
        return Ok(ManuscriptConfig { chapters: vec![] });
    }
    read_yaml(&path)
}

/// Write the manuscript config to disk, creating the directory if needed.
#[tauri::command]
pub fn save_manuscript_config(
    project_path: String,
    config: ManuscriptConfig,
) -> Result<(), AppError> {
    let path = config_path(&project_path);
    write_yaml(&path, &config)
}

/// Read a chapter file, parsing its frontmatter and body.
#[tauri::command]
pub fn get_chapter(project_path: String, slug: String) -> Result<ChapterContent, AppError> {
    let path = chapter_path(&project_path, &slug);
    if !path.exists() {
        return Err(AppError::NotFound(format!("Chapter not found: {}", slug)));
    }

    let content = std::fs::read_to_string(&path)?;
    let doc: frontmatter::ParsedDocument<ChapterFrontmatter> = frontmatter::parse(&content)?;
    let fm = doc.frontmatter;

    Ok(ChapterContent {
        slug: fm.slug.clone(),
        frontmatter: Chapter {
            slug: fm.slug,
            title: fm.title,
            status: fm.status,
            pov: fm.pov,
            synopsis: fm.synopsis,
            target_words: fm.target_words,
            order: fm.order,
        },
        body: doc.body,
    })
}

/// Write a chapter file with the given frontmatter and body.
#[tauri::command]
pub fn save_chapter(
    project_path: String,
    slug: String,
    chapter: Chapter,
    body: String,
) -> Result<(), AppError> {
    let dir = manuscript_dir(&project_path);
    std::fs::create_dir_all(&dir)?;

    let fm = ChapterFrontmatter {
        title: chapter.title,
        slug: chapter.slug,
        status: chapter.status,
        pov: chapter.pov,
        synopsis: chapter.synopsis,
        target_words: chapter.target_words,
        order: chapter.order,
    };

    let path = chapter_path(&project_path, &slug);
    let content = frontmatter::serialize(&fm, &body)?;
    std::fs::write(&path, content)?;
    Ok(())
}

/// Create a new chapter: generate slug, assign order, write files, update config.
#[tauri::command]
pub fn create_chapter(project_path: String, title: String) -> Result<ChapterContent, AppError> {
    let slug = slugify(&title);
    if slug.is_empty() {
        return Err(AppError::Validation(
            "Title must produce a non-empty slug".to_string(),
        ));
    }

    let path = chapter_path(&project_path, &slug);
    if path.exists() {
        return Err(AppError::AlreadyExists(format!(
            "Chapter already exists: {}",
            slug
        )));
    }

    // Read current config to determine next order index
    let mut config = get_manuscript_config(project_path.clone())?;
    let order = config.chapters.len() as u32;

    let chapter = Chapter {
        slug: slug.clone(),
        title: title.clone(),
        status: ChapterStatus::Draft,
        pov: None,
        synopsis: None,
        target_words: None,
        order,
    };

    // Save the chapter file
    save_chapter(
        project_path.clone(),
        slug.clone(),
        chapter.clone(),
        String::new(),
    )?;

    // Update and save config
    config.chapters.push(slug.clone());
    save_manuscript_config(project_path, config)?;

    Ok(ChapterContent {
        slug,
        frontmatter: chapter,
        body: String::new(),
    })
}

/// Delete a chapter file and remove it from the manuscript config.
#[tauri::command]
pub fn delete_chapter(project_path: String, slug: String) -> Result<(), AppError> {
    let path = chapter_path(&project_path, &slug);
    if !path.exists() {
        return Err(AppError::NotFound(format!("Chapter not found: {}", slug)));
    }

    std::fs::remove_file(&path)?;

    // Remove from config
    let mut config = get_manuscript_config(project_path.clone())?;
    config.chapters.retain(|s| s != &slug);
    save_manuscript_config(project_path, config)?;

    Ok(())
}

/// Rename a chapter: update its title (and slug/filename if the slug changes).
#[tauri::command]
pub fn rename_chapter(
    project_path: String,
    slug: String,
    new_title: String,
) -> Result<ChapterContent, AppError> {
    let existing = get_chapter(project_path.clone(), slug.clone())?;
    let new_slug = slugify(&new_title);

    if new_slug.is_empty() {
        return Err(AppError::Validation(
            "Title must produce a non-empty slug".to_string(),
        ));
    }

    let mut chapter = existing.frontmatter;
    chapter.title = new_title;
    let body = existing.body;

    if new_slug == slug {
        // Same slug — just update the title in place
        save_chapter(project_path, slug.clone(), chapter.clone(), body.clone())?;
        return Ok(ChapterContent {
            slug,
            frontmatter: chapter,
            body,
        });
    }

    // Different slug — write new file, delete old, update config
    chapter.slug = new_slug.clone();
    save_chapter(
        project_path.clone(),
        new_slug.clone(),
        chapter.clone(),
        body.clone(),
    )?;

    // Delete the old file
    let old_path = chapter_path(&project_path, &slug);
    std::fs::remove_file(&old_path)?;

    // Update the manuscript config: replace old slug with new slug
    let mut config = get_manuscript_config(project_path.clone())?;
    if let Some(entry) = config.chapters.iter_mut().find(|s| **s == slug) {
        *entry = new_slug.clone();
    }
    save_manuscript_config(project_path, config)?;

    Ok(ChapterContent {
        slug: new_slug,
        frontmatter: chapter,
        body,
    })
}

/// Reorder chapters: replace the config ordering and update each chapter file's order field.
#[tauri::command]
pub fn reorder_chapters(project_path: String, chapter_slugs: Vec<String>) -> Result<(), AppError> {
    // Save the new ordering to config
    let config = ManuscriptConfig {
        chapters: chapter_slugs.clone(),
    };
    save_manuscript_config(project_path.clone(), config)?;

    // Update each chapter file's order field
    for (i, slug) in chapter_slugs.iter().enumerate() {
        let path = chapter_path(&project_path, slug);
        if !path.exists() {
            return Err(AppError::NotFound(format!("Chapter not found: {}", slug)));
        }

        let content = std::fs::read_to_string(&path)?;
        let doc: frontmatter::ParsedDocument<ChapterFrontmatter> = frontmatter::parse(&content)?;
        let mut fm = doc.frontmatter;
        fm.order = i as u32;

        let chapter = Chapter {
            slug: fm.slug,
            title: fm.title,
            status: fm.status,
            pov: fm.pov,
            synopsis: fm.synopsis,
            target_words: fm.target_words,
            order: fm.order,
        };

        save_chapter(project_path.clone(), slug.clone(), chapter, doc.body)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::setup_test_dir;

    // ── get_manuscript_config ──────────────────────────────────────

    #[test]
    fn get_config_missing_dir_returns_empty() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let config = get_manuscript_config(pp).unwrap();
        assert!(config.chapters.is_empty());
    }

    #[test]
    fn get_config_empty_dir_returns_empty() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();
        std::fs::create_dir_all(dir.path().join("manuscript")).unwrap();

        let config = get_manuscript_config(pp).unwrap();
        assert!(config.chapters.is_empty());
    }

    #[test]
    fn get_config_reads_existing() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let config = ManuscriptConfig {
            chapters: vec!["chapter-one".to_string(), "chapter-two".to_string()],
        };
        let path = dir.path().join("manuscript/manuscript.yaml");
        write_yaml(&path, &config).unwrap();

        let loaded = get_manuscript_config(pp).unwrap();
        assert_eq!(loaded.chapters, vec!["chapter-one", "chapter-two"]);
    }

    // ── save_manuscript_config ─────────────────────────────────────

    #[test]
    fn save_config_creates_file() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let config = ManuscriptConfig {
            chapters: vec!["intro".to_string()],
        };
        save_manuscript_config(pp.clone(), config).unwrap();

        assert!(dir.path().join("manuscript/manuscript.yaml").exists());

        let loaded = get_manuscript_config(pp).unwrap();
        assert_eq!(loaded.chapters, vec!["intro"]);
    }

    #[test]
    fn save_config_round_trips() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let config = ManuscriptConfig {
            chapters: vec![
                "prologue".to_string(),
                "chapter-1".to_string(),
                "epilogue".to_string(),
            ],
        };
        save_manuscript_config(pp.clone(), config.clone()).unwrap();

        let loaded = get_manuscript_config(pp).unwrap();
        assert_eq!(loaded.chapters, config.chapters);
    }

    #[test]
    fn save_config_overwrites_existing() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let config1 = ManuscriptConfig {
            chapters: vec!["a".to_string()],
        };
        save_manuscript_config(pp.clone(), config1).unwrap();

        let config2 = ManuscriptConfig {
            chapters: vec!["b".to_string(), "c".to_string()],
        };
        save_manuscript_config(pp.clone(), config2).unwrap();

        let loaded = get_manuscript_config(pp).unwrap();
        assert_eq!(loaded.chapters, vec!["b", "c"]);
    }

    // ── create_chapter ─────────────────────────────────────────────

    #[test]
    fn create_chapter_creates_file_and_updates_config() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let result = create_chapter(pp.clone(), "The Beginning".to_string()).unwrap();

        assert_eq!(result.slug, "the-beginning");
        assert_eq!(result.frontmatter.title, "The Beginning");
        assert_eq!(result.frontmatter.status, ChapterStatus::Draft);
        assert_eq!(result.frontmatter.order, 0);
        assert!(result.frontmatter.pov.is_none());
        assert!(result.frontmatter.synopsis.is_none());
        assert!(result.frontmatter.target_words.is_none());
        assert!(result.body.is_empty());

        // Verify file on disk
        assert!(dir.path().join("manuscript/the-beginning.md").exists());

        // Verify config updated
        let config = get_manuscript_config(pp).unwrap();
        assert_eq!(config.chapters, vec!["the-beginning"]);
    }

    #[test]
    fn create_chapter_assigns_sequential_order() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let ch1 = create_chapter(pp.clone(), "Chapter One".to_string()).unwrap();
        let ch2 = create_chapter(pp.clone(), "Chapter Two".to_string()).unwrap();
        let ch3 = create_chapter(pp.clone(), "Chapter Three".to_string()).unwrap();

        assert_eq!(ch1.frontmatter.order, 0);
        assert_eq!(ch2.frontmatter.order, 1);
        assert_eq!(ch3.frontmatter.order, 2);

        let config = get_manuscript_config(pp).unwrap();
        assert_eq!(
            config.chapters,
            vec!["chapter-one", "chapter-two", "chapter-three"]
        );
    }

    #[test]
    fn create_chapter_duplicate_title_returns_already_exists() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        create_chapter(pp.clone(), "Prologue".to_string()).unwrap();
        let result = create_chapter(pp, "Prologue".to_string());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Already exists"),
            "Expected 'Already exists' error, got: {}",
            err_msg
        );
    }

    #[test]
    fn create_chapter_special_characters_in_title() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let result =
            create_chapter(pp.clone(), "The Hero's Journey & Beyond!".to_string()).unwrap();

        assert_eq!(result.slug, "the-hero-s-journey-beyond");
        assert_eq!(result.frontmatter.title, "The Hero's Journey & Beyond!");
        assert!(dir
            .path()
            .join("manuscript/the-hero-s-journey-beyond.md")
            .exists());
    }

    #[test]
    fn create_chapter_creates_manuscript_directory() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        assert!(!dir.path().join("manuscript").exists());

        create_chapter(pp, "First Chapter".to_string()).unwrap();

        assert!(dir.path().join("manuscript").exists());
    }

    // ── get_chapter ────────────────────────────────────────────────

    #[test]
    fn get_chapter_reads_created_chapter() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        create_chapter(pp.clone(), "Opening".to_string()).unwrap();

        let chapter = get_chapter(pp, "opening".to_string()).unwrap();

        assert_eq!(chapter.slug, "opening");
        assert_eq!(chapter.frontmatter.title, "Opening");
        assert_eq!(chapter.frontmatter.status, ChapterStatus::Draft);
        assert_eq!(chapter.frontmatter.order, 0);
        assert!(chapter.body.is_empty());
    }

    #[test]
    fn get_chapter_nonexistent_returns_not_found() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let result = get_chapter(pp, "nonexistent".to_string());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not found") || err_msg.contains("Not found"),
            "Expected 'not found' error, got: {}",
            err_msg
        );
    }

    #[test]
    fn get_chapter_reads_body_and_frontmatter() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        // Create chapter then update it with body
        create_chapter(pp.clone(), "Narrative".to_string()).unwrap();

        let chapter = Chapter {
            slug: "narrative".to_string(),
            title: "Narrative".to_string(),
            status: ChapterStatus::Revised,
            pov: Some("Alice".to_string()),
            synopsis: Some("Alice explores the garden.".to_string()),
            target_words: Some(5000),
            order: 0,
        };
        let body = "The garden was vast and green.\n\nAlice stepped through the gate.\n";
        save_chapter(
            pp.clone(),
            "narrative".to_string(),
            chapter,
            body.to_string(),
        )
        .unwrap();

        let loaded = get_chapter(pp, "narrative".to_string()).unwrap();
        assert_eq!(loaded.frontmatter.title, "Narrative");
        assert_eq!(loaded.frontmatter.status, ChapterStatus::Revised);
        assert_eq!(loaded.frontmatter.pov, Some("Alice".to_string()));
        assert_eq!(
            loaded.frontmatter.synopsis,
            Some("Alice explores the garden.".to_string())
        );
        assert_eq!(loaded.frontmatter.target_words, Some(5000));
        assert_eq!(
            loaded.body,
            "The garden was vast and green.\n\nAlice stepped through the gate.\n"
        );
    }

    // ── save_chapter ───────────────────────────────────────────────

    #[test]
    fn save_chapter_updates_fields_and_body() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        create_chapter(pp.clone(), "Editable".to_string()).unwrap();

        let updated = Chapter {
            slug: "editable".to_string(),
            title: "Editable Chapter".to_string(),
            status: ChapterStatus::Final,
            pov: Some("Narrator".to_string()),
            synopsis: Some("A fully edited chapter.".to_string()),
            target_words: Some(3000),
            order: 0,
        };
        save_chapter(
            pp.clone(),
            "editable".to_string(),
            updated,
            "Final body content.\n".to_string(),
        )
        .unwrap();

        let loaded = get_chapter(pp, "editable".to_string()).unwrap();
        assert_eq!(loaded.frontmatter.title, "Editable Chapter");
        assert_eq!(loaded.frontmatter.status, ChapterStatus::Final);
        assert_eq!(loaded.frontmatter.pov, Some("Narrator".to_string()));
        assert_eq!(
            loaded.frontmatter.synopsis,
            Some("A fully edited chapter.".to_string())
        );
        assert_eq!(loaded.frontmatter.target_words, Some(3000));
        assert_eq!(loaded.body, "Final body content.\n");
    }

    #[test]
    fn save_chapter_round_trips_all_statuses() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        for (i, status) in [
            ChapterStatus::Draft,
            ChapterStatus::Revised,
            ChapterStatus::Final,
        ]
        .iter()
        .enumerate()
        {
            let slug = format!("ch-{}", i);
            let chapter = Chapter {
                slug: slug.clone(),
                title: format!("Chapter {}", i),
                status: status.clone(),
                pov: None,
                synopsis: None,
                target_words: None,
                order: i as u32,
            };
            save_chapter(pp.clone(), slug.clone(), chapter, String::new()).unwrap();

            let loaded = get_chapter(pp.clone(), slug).unwrap();
            assert_eq!(loaded.frontmatter.status, *status);
        }
    }

    #[test]
    fn save_chapter_empty_body() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let chapter = Chapter {
            slug: "empty-body".to_string(),
            title: "Empty Body".to_string(),
            status: ChapterStatus::Draft,
            pov: None,
            synopsis: None,
            target_words: None,
            order: 0,
        };
        save_chapter(pp.clone(), "empty-body".to_string(), chapter, String::new()).unwrap();

        let loaded = get_chapter(pp, "empty-body".to_string()).unwrap();
        assert!(loaded.body.is_empty());
    }

    #[test]
    fn save_chapter_creates_manuscript_directory() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        assert!(!dir.path().join("manuscript").exists());

        let chapter = Chapter {
            slug: "first".to_string(),
            title: "First".to_string(),
            status: ChapterStatus::Draft,
            pov: None,
            synopsis: None,
            target_words: None,
            order: 0,
        };
        save_chapter(pp, "first".to_string(), chapter, String::new()).unwrap();

        assert!(dir.path().join("manuscript").exists());
        assert!(dir.path().join("manuscript/first.md").exists());
    }

    // ── delete_chapter ─────────────────────────────────────────────

    #[test]
    fn delete_chapter_removes_file_and_config_entry() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        create_chapter(pp.clone(), "Doomed".to_string()).unwrap();
        create_chapter(pp.clone(), "Survivor".to_string()).unwrap();

        assert!(dir.path().join("manuscript/doomed.md").exists());

        delete_chapter(pp.clone(), "doomed".to_string()).unwrap();

        assert!(!dir.path().join("manuscript/doomed.md").exists());

        let config = get_manuscript_config(pp).unwrap();
        assert_eq!(config.chapters, vec!["survivor"]);
    }

    #[test]
    fn delete_chapter_nonexistent_returns_not_found() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let result = delete_chapter(pp, "ghost".to_string());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not found") || err_msg.contains("Not found"),
            "Expected 'not found' error, got: {}",
            err_msg
        );
    }

    #[test]
    fn delete_chapter_last_chapter_leaves_empty_config() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        create_chapter(pp.clone(), "Only One".to_string()).unwrap();
        delete_chapter(pp.clone(), "only-one".to_string()).unwrap();

        let config = get_manuscript_config(pp).unwrap();
        assert!(config.chapters.is_empty());
    }

    // ── reorder_chapters ───────────────────────────────────────────

    #[test]
    fn reorder_chapters_updates_config_and_order_fields() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        create_chapter(pp.clone(), "Alpha".to_string()).unwrap();
        create_chapter(pp.clone(), "Beta".to_string()).unwrap();
        create_chapter(pp.clone(), "Gamma".to_string()).unwrap();

        // Verify original order
        let config = get_manuscript_config(pp.clone()).unwrap();
        assert_eq!(config.chapters, vec!["alpha", "beta", "gamma"]);

        // Reorder: gamma, alpha, beta
        reorder_chapters(
            pp.clone(),
            vec!["gamma".to_string(), "alpha".to_string(), "beta".to_string()],
        )
        .unwrap();

        // Config updated
        let config = get_manuscript_config(pp.clone()).unwrap();
        assert_eq!(config.chapters, vec!["gamma", "alpha", "beta"]);

        // Order fields updated in chapter files
        let gamma = get_chapter(pp.clone(), "gamma".to_string()).unwrap();
        assert_eq!(gamma.frontmatter.order, 0);

        let alpha = get_chapter(pp.clone(), "alpha".to_string()).unwrap();
        assert_eq!(alpha.frontmatter.order, 1);

        let beta = get_chapter(pp, "beta".to_string()).unwrap();
        assert_eq!(beta.frontmatter.order, 2);
    }

    #[test]
    fn reorder_chapters_with_missing_slug_returns_not_found() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        create_chapter(pp.clone(), "Real".to_string()).unwrap();

        let result = reorder_chapters(pp, vec!["real".to_string(), "fake".to_string()]);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not found") || err_msg.contains("Not found"),
            "Expected 'not found' error, got: {}",
            err_msg
        );
    }

    #[test]
    fn reorder_chapters_preserves_body_and_other_fields() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        create_chapter(pp.clone(), "First".to_string()).unwrap();
        create_chapter(pp.clone(), "Second".to_string()).unwrap();

        // Update first chapter with body and extra fields
        let ch = Chapter {
            slug: "first".to_string(),
            title: "First".to_string(),
            status: ChapterStatus::Revised,
            pov: Some("Hero".to_string()),
            synopsis: Some("The hero arrives.".to_string()),
            target_words: Some(2000),
            order: 0,
        };
        save_chapter(
            pp.clone(),
            "first".to_string(),
            ch,
            "Once upon a time...\n".to_string(),
        )
        .unwrap();

        // Reorder: second, first
        reorder_chapters(pp.clone(), vec!["second".to_string(), "first".to_string()]).unwrap();

        // Verify first chapter kept all its data, only order changed
        let loaded = get_chapter(pp, "first".to_string()).unwrap();
        assert_eq!(loaded.frontmatter.order, 1);
        assert_eq!(loaded.frontmatter.status, ChapterStatus::Revised);
        assert_eq!(loaded.frontmatter.pov, Some("Hero".to_string()));
        assert_eq!(
            loaded.frontmatter.synopsis,
            Some("The hero arrives.".to_string())
        );
        assert_eq!(loaded.frontmatter.target_words, Some(2000));
        assert_eq!(loaded.body, "Once upon a time...\n");
    }

    #[test]
    fn reorder_chapters_empty_list() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        create_chapter(pp.clone(), "Lonely".to_string()).unwrap();

        // Reorder with empty list
        reorder_chapters(pp.clone(), vec![]).unwrap();

        let config = get_manuscript_config(pp).unwrap();
        assert!(config.chapters.is_empty());
    }

    // ── Integration / multi-step scenarios ──────────────────────────

    #[test]
    fn full_lifecycle_create_edit_reorder_delete() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        // Create three chapters
        create_chapter(pp.clone(), "Prologue".to_string()).unwrap();
        create_chapter(pp.clone(), "The Middle".to_string()).unwrap();
        create_chapter(pp.clone(), "Epilogue".to_string()).unwrap();

        // Verify initial state
        let config = get_manuscript_config(pp.clone()).unwrap();
        assert_eq!(config.chapters, vec!["prologue", "the-middle", "epilogue"]);

        // Edit the middle chapter
        let updated = Chapter {
            slug: "the-middle".to_string(),
            title: "The Middle".to_string(),
            status: ChapterStatus::Revised,
            pov: Some("Narrator".to_string()),
            synopsis: Some("The climax of the story.".to_string()),
            target_words: Some(8000),
            order: 1,
        };
        save_chapter(
            pp.clone(),
            "the-middle".to_string(),
            updated,
            "It was a dark and stormy night...\n".to_string(),
        )
        .unwrap();

        // Reorder: epilogue first
        reorder_chapters(
            pp.clone(),
            vec![
                "epilogue".to_string(),
                "prologue".to_string(),
                "the-middle".to_string(),
            ],
        )
        .unwrap();

        let config = get_manuscript_config(pp.clone()).unwrap();
        assert_eq!(config.chapters, vec!["epilogue", "prologue", "the-middle"]);

        // Verify order fields
        assert_eq!(
            get_chapter(pp.clone(), "epilogue".to_string())
                .unwrap()
                .frontmatter
                .order,
            0
        );
        assert_eq!(
            get_chapter(pp.clone(), "prologue".to_string())
                .unwrap()
                .frontmatter
                .order,
            1
        );
        assert_eq!(
            get_chapter(pp.clone(), "the-middle".to_string())
                .unwrap()
                .frontmatter
                .order,
            2
        );

        // Verify edited content preserved after reorder
        let middle = get_chapter(pp.clone(), "the-middle".to_string()).unwrap();
        assert_eq!(middle.frontmatter.status, ChapterStatus::Revised);
        assert_eq!(middle.frontmatter.pov, Some("Narrator".to_string()));
        assert_eq!(middle.body, "It was a dark and stormy night...\n");

        // Delete prologue
        delete_chapter(pp.clone(), "prologue".to_string()).unwrap();

        let config = get_manuscript_config(pp.clone()).unwrap();
        assert_eq!(config.chapters, vec!["epilogue", "the-middle"]);
        assert!(!dir.path().join("manuscript/prologue.md").exists());

        // Remaining chapters still accessible
        assert!(get_chapter(pp.clone(), "epilogue".to_string()).is_ok());
        assert!(get_chapter(pp, "the-middle".to_string()).is_ok());
    }

    #[test]
    fn chapter_status_serializes_as_snake_case() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let chapter = Chapter {
            slug: "status-test".to_string(),
            title: "Status Test".to_string(),
            status: ChapterStatus::Final,
            pov: None,
            synopsis: None,
            target_words: None,
            order: 0,
        };
        save_chapter(
            pp.clone(),
            "status-test".to_string(),
            chapter,
            String::new(),
        )
        .unwrap();

        // Read raw file to verify snake_case serialization
        let raw = std::fs::read_to_string(dir.path().join("manuscript/status-test.md")).unwrap();
        assert!(
            raw.contains("final"),
            "Expected 'final' in raw file, got: {}",
            raw
        );
    }

    #[test]
    fn multiple_chapters_different_statuses() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let statuses = vec![
            ("Draft Chapter", ChapterStatus::Draft),
            ("Revised Chapter", ChapterStatus::Revised),
            ("Final Chapter", ChapterStatus::Final),
        ];

        for (title, status) in &statuses {
            let slug = slugify(title);
            let ch = Chapter {
                slug: slug.clone(),
                title: title.to_string(),
                status: status.clone(),
                pov: None,
                synopsis: None,
                target_words: None,
                order: 0,
            };
            save_chapter(pp.clone(), slug, ch, String::new()).unwrap();
        }

        let draft = get_chapter(pp.clone(), "draft-chapter".to_string()).unwrap();
        assert_eq!(draft.frontmatter.status, ChapterStatus::Draft);

        let revised = get_chapter(pp.clone(), "revised-chapter".to_string()).unwrap();
        assert_eq!(revised.frontmatter.status, ChapterStatus::Revised);

        let final_ch = get_chapter(pp, "final-chapter".to_string()).unwrap();
        assert_eq!(final_ch.frontmatter.status, ChapterStatus::Final);
    }

    // ── rename_chapter ─────────────────────────────────────────────

    #[test]
    fn rename_chapter_with_slug_change_moves_file() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        create_chapter(pp.clone(), "The Beginning".to_string()).unwrap();

        let renamed = rename_chapter(
            pp.clone(),
            "the-beginning".to_string(),
            "A New Dawn".to_string(),
        )
        .unwrap();

        assert_eq!(renamed.slug, "a-new-dawn");
        assert_eq!(renamed.frontmatter.title, "A New Dawn");
        assert_eq!(renamed.frontmatter.slug, "a-new-dawn");

        // Old file should be gone
        assert!(!dir.path().join("manuscript/the-beginning.md").exists());
        // New file should exist
        assert!(dir.path().join("manuscript/a-new-dawn.md").exists());

        // Should be retrievable by new slug
        let loaded = get_chapter(pp, "a-new-dawn".to_string()).unwrap();
        assert_eq!(loaded.frontmatter.title, "A New Dawn");
    }

    #[test]
    fn rename_chapter_same_slug_updates_title_only() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        // "The Beginning" and "THE BEGINNING" both slugify to "the-beginning"
        create_chapter(pp.clone(), "The Beginning".to_string()).unwrap();

        let renamed = rename_chapter(
            pp.clone(),
            "the-beginning".to_string(),
            "THE BEGINNING".to_string(),
        )
        .unwrap();

        assert_eq!(renamed.slug, "the-beginning");
        assert_eq!(renamed.frontmatter.title, "THE BEGINNING");

        // File should still exist at the same path
        assert!(dir.path().join("manuscript/the-beginning.md").exists());

        let loaded = get_chapter(pp, "the-beginning".to_string()).unwrap();
        assert_eq!(loaded.frontmatter.title, "THE BEGINNING");
    }

    #[test]
    fn rename_chapter_updates_manuscript_config() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        create_chapter(pp.clone(), "Prologue".to_string()).unwrap();
        create_chapter(pp.clone(), "Chapter One".to_string()).unwrap();
        create_chapter(pp.clone(), "Epilogue".to_string()).unwrap();

        // Rename the middle chapter
        rename_chapter(
            pp.clone(),
            "chapter-one".to_string(),
            "The First Act".to_string(),
        )
        .unwrap();

        let config = get_manuscript_config(pp).unwrap();
        assert_eq!(
            config.chapters,
            vec!["prologue", "the-first-act", "epilogue"]
        );
    }

    #[test]
    fn rename_chapter_preserves_body_and_metadata() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        create_chapter(pp.clone(), "Original Title".to_string()).unwrap();

        // Edit the chapter with body and metadata
        let chapter = Chapter {
            slug: "original-title".to_string(),
            title: "Original Title".to_string(),
            status: ChapterStatus::Revised,
            pov: Some("Alice".to_string()),
            synopsis: Some("A test chapter.".to_string()),
            target_words: Some(5000),
            order: 0,
        };
        save_chapter(
            pp.clone(),
            "original-title".to_string(),
            chapter,
            "Once upon a time...\n".to_string(),
        )
        .unwrap();

        let renamed = rename_chapter(
            pp.clone(),
            "original-title".to_string(),
            "New Title".to_string(),
        )
        .unwrap();

        assert_eq!(renamed.body, "Once upon a time...\n");
        assert_eq!(renamed.frontmatter.status, ChapterStatus::Revised);
        assert_eq!(renamed.frontmatter.pov, Some("Alice".to_string()));
        assert_eq!(
            renamed.frontmatter.synopsis,
            Some("A test chapter.".to_string())
        );
        assert_eq!(renamed.frontmatter.target_words, Some(5000));
    }

    #[test]
    fn rename_chapter_nonexistent_returns_not_found() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let result = rename_chapter(pp, "does-not-exist".to_string(), "New Name".to_string());
        assert!(result.is_err());
    }
}
