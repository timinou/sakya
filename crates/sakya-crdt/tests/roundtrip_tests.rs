//! Round-trip integration tests for sakya-crdt import/export.
//!
//! Verifies: project files → CrdtProject → export → files are semantically equivalent.
//! Includes property-based tests for CRDT consistency guarantees.

use proptest::prelude::*;
use sakya_crdt::{export_to_files, import_from_files, CrdtProject};
use std::path::Path;
use tempfile::TempDir;
use uuid::Uuid;

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Create a minimal test project (1 chapter, no notes/entities).
fn create_minimal_project(root: &Path) {
    std::fs::create_dir_all(root.join("manuscript")).unwrap();

    std::fs::write(
        root.join("manuscript/manuscript.yaml"),
        "chapters:\n  - intro\n",
    )
    .unwrap();

    std::fs::write(
        root.join("manuscript/intro.md"),
        "---\ntitle: Introduction\nslug: intro\nstatus: draft\norder: 0\n---\nOnce upon a time.\n",
    )
    .unwrap();
}

/// Create a medium test project (5 chapters, notes, entities).
fn create_medium_project(root: &Path) {
    std::fs::create_dir_all(root.join("manuscript")).unwrap();
    std::fs::create_dir_all(root.join("notes")).unwrap();
    std::fs::create_dir_all(root.join("entities/character")).unwrap();
    std::fs::create_dir_all(root.join("entities/place")).unwrap();
    std::fs::create_dir_all(root.join(".sakya")).unwrap();

    // Manuscript with 5 chapters
    std::fs::write(
        root.join("manuscript/manuscript.yaml"),
        "chapters:\n  - chapter-one\n  - chapter-two\n  - chapter-three\n  - chapter-four\n  - chapter-five\n",
    )
    .unwrap();

    let chapters = vec![
        (
            "chapter-one",
            "The Beginning",
            "draft",
            Some("Alice"),
            Some("Alice enters the world"),
            Some(3000),
        ),
        (
            "chapter-two",
            "The Journey",
            "draft",
            Some("Bob"),
            None,
            Some(4000),
        ),
        (
            "chapter-three",
            "The Trial",
            "revised",
            None,
            Some("A test of courage"),
            None,
        ),
        (
            "chapter-four",
            "The Battle",
            "draft",
            Some("Alice"),
            Some("The final confrontation"),
            Some(5000),
        ),
        ("chapter-five", "The Return", "final", None, None, None),
    ];

    for (i, (slug, title, status, pov, synopsis, tw)) in chapters.iter().enumerate() {
        let mut fm = format!(
            "---\ntitle: {}\nslug: {}\nstatus: {}\n",
            title, slug, status
        );
        if let Some(pov) = pov {
            fm.push_str(&format!("pov: {}\n", pov));
        }
        if let Some(synopsis) = synopsis {
            fm.push_str(&format!("synopsis: {}\n", synopsis));
        }
        if let Some(tw) = tw {
            fm.push_str(&format!("targetWords: {}\n", tw));
        }
        fm.push_str(&format!("order: {}\n", i));
        fm.push_str("---\n");
        fm.push_str(&format!(
            "This is the body of {}.\n\nIt has multiple paragraphs.",
            title
        ));

        std::fs::write(root.join(format!("manuscript/{}.md", slug)), fm).unwrap();
    }

    // Notes
    std::fs::write(
        root.join("notes/notes.yaml"),
        "notes:\n  - slug: plot-twist\n    title: Plot Twist\n    color: '#ff0000'\n    label: critical\n    position:\n      x: 100.0\n      y: 200.0\n  - slug: world-building\n    title: World Building\n  - slug: theme-notes\n    title: Theme Notes\n    color: '#00ff00'\n",
    )
    .unwrap();

    for (slug, title, body) in &[
        (
            "plot-twist",
            "Plot Twist",
            "The villain was the hero's mentor all along.",
        ),
        (
            "world-building",
            "World Building",
            "The world has two moons and three continents.",
        ),
        (
            "theme-notes",
            "Theme Notes",
            "Coming of age through adversity.",
        ),
    ] {
        std::fs::write(
            root.join(format!("notes/{}.md", slug)),
            format!("---\ntitle: {}\nslug: {}\n---\n{}", title, slug, body),
        )
        .unwrap();
    }

    // Entities
    std::fs::write(
        root.join("entities/character/alice.md"),
        "---\ntitle: Alice\nslug: alice\nschemaType: character\ntags:\n  - protagonist\n  - hero\nspiderValues:\n  Courage: 8.5\n  Wisdom: 6.0\nfields:\n  role: Protagonist\n  age: 25\n---\nAlice is brave and determined.",
    )
    .unwrap();

    std::fs::write(
        root.join("entities/character/bob.md"),
        "---\ntitle: Bob\nslug: bob\nschemaType: character\ntags:\n  - sidekick\nfields:\n  role: Sidekick\n---\nBob is loyal and resourceful.",
    )
    .unwrap();

    std::fs::write(
        root.join("entities/place/forest.md"),
        "---\ntitle: Enchanted Forest\nslug: forest\nschemaType: place\ntags:\n  - dangerous\nfields:\n  type: wilderness\n---\nA dark and mysterious forest.",
    )
    .unwrap();

    // Sessions
    std::fs::write(
        root.join(".sakya/sessions.yaml"),
        "sessions:\n  - id: '2026-02-01T10:00:00Z'\n    start: '2026-02-01T10:00:00Z'\n    end: '2026-02-01T10:30:00Z'\n    durationMinutes: 30.0\n    wordsWritten: 500\n    chapterSlug: chapter-one\n    sprintGoal: 400\n  - id: '2026-02-02T14:00:00Z'\n    start: '2026-02-02T14:00:00Z'\n    end: '2026-02-02T15:00:00Z'\n    durationMinutes: 60.0\n    wordsWritten: 1200\n    chapterSlug: chapter-two\n",
    )
    .unwrap();
}

/// Create a project with varied formatting in chapter bodies.
fn create_formatted_project(root: &Path) {
    std::fs::create_dir_all(root.join("manuscript")).unwrap();

    std::fs::write(
        root.join("manuscript/manuscript.yaml"),
        "chapters:\n  - formatted\n",
    )
    .unwrap();

    std::fs::write(
        root.join("manuscript/formatted.md"),
        r#"---
title: Formatted Chapter
slug: formatted
status: draft
order: 0
---
The hero walked **boldly** into the *sunset*.

She held the `ancient-scroll` in her hands.

> "To be or not to be," she whispered.

The ~~old path~~ new road lay ahead.

Visit [the castle](https://example.com/castle) for more."#,
    )
    .unwrap();
}

// ── Minimal project tests ────────────────────────────────────────────────────

#[test]
fn roundtrip_minimal_project() {
    let src = TempDir::new().unwrap();
    create_minimal_project(src.path());

    let project = import_from_files(src.path()).unwrap();
    let dst = TempDir::new().unwrap();
    export_to_files(&project, dst.path()).unwrap();

    // Verify manuscript.yaml exists and contains the slug
    let manifest = std::fs::read_to_string(dst.path().join("manuscript/manuscript.yaml")).unwrap();
    assert!(manifest.contains("intro"));

    // Verify chapter file exists with correct content
    let chapter = std::fs::read_to_string(dst.path().join("manuscript/intro.md")).unwrap();
    assert!(chapter.contains("title: Introduction"));
    assert!(chapter.contains("slug: intro"));
    assert!(chapter.contains("status: draft"));
    assert!(chapter.contains("Once upon a time."));
}

#[test]
fn roundtrip_minimal_chapter_count() {
    let src = TempDir::new().unwrap();
    create_minimal_project(src.path());

    let project = import_from_files(src.path()).unwrap();
    assert_eq!(project.list_chapters().unwrap().len(), 1);
}

// ── Medium project tests ─────────────────────────────────────────────────────

#[test]
fn roundtrip_medium_project_chapters() {
    let src = TempDir::new().unwrap();
    create_medium_project(src.path());

    let project = import_from_files(src.path()).unwrap();
    let dst = TempDir::new().unwrap();
    export_to_files(&project, dst.path()).unwrap();

    // Verify all 5 chapters exist
    let chapters = project.list_chapters().unwrap();
    assert_eq!(chapters.len(), 5);
    assert_eq!(chapters[0].slug, "chapter-one");
    assert_eq!(chapters[4].slug, "chapter-five");

    // Verify chapter ordering in manuscript.yaml
    let manifest = std::fs::read_to_string(dst.path().join("manuscript/manuscript.yaml")).unwrap();
    let first_pos = manifest.find("chapter-one").unwrap();
    let last_pos = manifest.find("chapter-five").unwrap();
    assert!(first_pos < last_pos, "Chapter ordering should be preserved");
}

#[test]
fn roundtrip_medium_project_chapter_metadata() {
    let src = TempDir::new().unwrap();
    create_medium_project(src.path());

    let project = import_from_files(src.path()).unwrap();

    // Chapter with full metadata
    let ch1 = project.get_chapter("chapter-one").unwrap();
    assert_eq!(ch1.title, "The Beginning");
    assert_eq!(ch1.status, "draft");
    assert_eq!(ch1.pov, Some("Alice".to_string()));
    assert_eq!(ch1.synopsis, Some("Alice enters the world".to_string()));
    assert_eq!(ch1.target_words, Some(3000));
    assert!(ch1.body.contains("This is the body of The Beginning."));

    // Chapter with minimal metadata
    let ch5 = project.get_chapter("chapter-five").unwrap();
    assert_eq!(ch5.status, "final");
    assert_eq!(ch5.pov, None);
    assert_eq!(ch5.synopsis, None);
    assert_eq!(ch5.target_words, None);
}

#[test]
fn roundtrip_medium_project_notes() {
    let src = TempDir::new().unwrap();
    create_medium_project(src.path());

    let project = import_from_files(src.path()).unwrap();
    let dst = TempDir::new().unwrap();
    export_to_files(&project, dst.path()).unwrap();

    // Verify note count
    let notes = project.list_notes().unwrap();
    assert_eq!(notes.len(), 3);

    // Verify note with full metadata
    let (note, pos_x, pos_y) = project.get_note_full("plot-twist").unwrap();
    assert_eq!(note.title, "Plot Twist");
    assert_eq!(note.color, Some("#ff0000".to_string()));
    assert_eq!(note.label, Some("critical".to_string()));
    assert_eq!(pos_x, Some(100.0));
    assert_eq!(pos_y, Some(200.0));
    assert_eq!(note.body, "The villain was the hero's mentor all along.");

    // Verify note without metadata
    let note2 = project.get_note("world-building").unwrap();
    assert_eq!(note2.color, None);
    assert_eq!(note2.label, None);

    // Verify exported note file
    let exported_note = std::fs::read_to_string(dst.path().join("notes/plot-twist.md")).unwrap();
    assert!(exported_note.contains("Plot Twist"));
    assert!(exported_note.contains("villain was the hero's mentor"));
}

#[test]
fn roundtrip_medium_project_entities() {
    let src = TempDir::new().unwrap();
    create_medium_project(src.path());

    let project = import_from_files(src.path()).unwrap();
    let dst = TempDir::new().unwrap();
    export_to_files(&project, dst.path()).unwrap();

    // Verify entities per schema
    let characters = project.list_entities("character").unwrap();
    assert_eq!(characters.len(), 2);

    let places = project.list_entities("place").unwrap();
    assert_eq!(places.len(), 1);

    // Verify entity data
    let alice = project.get_entity("character", "alice").unwrap();
    assert_eq!(alice.title, "Alice");
    let fields = alice.fields.as_object().unwrap();
    assert_eq!(fields.get("role").unwrap(), "Protagonist");

    // Verify tags round-tripped
    let tags = fields.get("tags").unwrap().as_array().unwrap();
    assert!(tags.contains(&serde_json::Value::String("protagonist".to_string())));
    assert!(tags.contains(&serde_json::Value::String("hero".to_string())));

    // Verify exported entity file
    let exported_entity =
        std::fs::read_to_string(dst.path().join("entities/character/alice.md")).unwrap();
    assert!(exported_entity.contains("title: Alice"));
    assert!(exported_entity.contains("Alice is brave and determined."));
}

#[test]
fn roundtrip_medium_project_sessions() {
    let src = TempDir::new().unwrap();
    create_medium_project(src.path());

    let project = import_from_files(src.path()).unwrap();
    let dst = TempDir::new().unwrap();
    export_to_files(&project, dst.path()).unwrap();

    // Verify session data
    let session = project.get_session("2026-02-01T10:00:00Z").unwrap();
    assert_eq!(session.words_written, 500);
    assert_eq!(session.chapter_slug, "chapter-one");
    assert_eq!(session.sprint_goal, Some(400));
    assert_eq!(session.duration_minutes, Some(30.0));

    // Verify exported sessions.yaml
    let exported_sessions =
        std::fs::read_to_string(dst.path().join(".sakya/sessions.yaml")).unwrap();
    assert!(exported_sessions.contains("2026-02-01T10:00:00Z"));
    assert!(exported_sessions.contains("wordsWritten: 500"));
}

// ── Formatted content tests ──────────────────────────────────────────────────

#[test]
fn roundtrip_formatted_chapter_body() {
    let src = TempDir::new().unwrap();
    create_formatted_project(src.path());

    let project = import_from_files(src.path()).unwrap();

    let ch = project.get_chapter("formatted").unwrap();
    // Body should contain raw markdown (preserved as-is)
    assert!(ch.body.contains("**boldly**"));
    assert!(ch.body.contains("*sunset*"));
    assert!(ch.body.contains("`ancient-scroll`"));
    assert!(ch.body.contains("~~old path~~"));
    assert!(ch.body.contains("[the castle](https://example.com/castle)"));
}

#[test]
fn roundtrip_formatted_chapter_export() {
    let src = TempDir::new().unwrap();
    create_formatted_project(src.path());

    let project = import_from_files(src.path()).unwrap();
    let dst = TempDir::new().unwrap();
    export_to_files(&project, dst.path()).unwrap();

    let exported = std::fs::read_to_string(dst.path().join("manuscript/formatted.md")).unwrap();
    assert!(exported.contains("**boldly**"));
    assert!(exported.contains("*sunset*"));
    assert!(exported.contains("`ancient-scroll`"));
    assert!(exported.contains("~~old path~~"));
    assert!(exported.contains("[the castle](https://example.com/castle)"));
}

// ── Edge case tests ──────────────────────────────────────────────────────────

#[test]
fn roundtrip_empty_project() {
    let src = TempDir::new().unwrap();
    let project = import_from_files(src.path()).unwrap();
    let dst = TempDir::new().unwrap();
    export_to_files(&project, dst.path()).unwrap();

    assert!(!dst.path().join("manuscript").exists());
    assert!(!dst.path().join("notes").exists());
}

#[test]
fn roundtrip_chapter_empty_body() {
    let src = TempDir::new().unwrap();
    std::fs::create_dir_all(src.path().join("manuscript")).unwrap();
    std::fs::write(
        src.path().join("manuscript/manuscript.yaml"),
        "chapters:\n  - empty\n",
    )
    .unwrap();
    std::fs::write(
        src.path().join("manuscript/empty.md"),
        "---\ntitle: Empty Chapter\nslug: empty\nstatus: draft\norder: 0\n---\n",
    )
    .unwrap();

    let project = import_from_files(src.path()).unwrap();
    let ch = project.get_chapter("empty").unwrap();
    assert_eq!(ch.body, "");

    let dst = TempDir::new().unwrap();
    export_to_files(&project, dst.path()).unwrap();
    let exported = std::fs::read_to_string(dst.path().join("manuscript/empty.md")).unwrap();
    assert!(exported.contains("title: Empty Chapter"));
}

#[test]
fn roundtrip_chapter_body_with_dashes() {
    let src = TempDir::new().unwrap();
    std::fs::create_dir_all(src.path().join("manuscript")).unwrap();
    std::fs::write(
        src.path().join("manuscript/manuscript.yaml"),
        "chapters:\n  - dashes\n",
    )
    .unwrap();
    std::fs::write(
        src.path().join("manuscript/dashes.md"),
        "---\ntitle: Dashes Chapter\nslug: dashes\nstatus: draft\norder: 0\n---\nHere is a scene break:\n\n---\n\nAnd another line with --- dashes.",
    )
    .unwrap();

    let project = import_from_files(src.path()).unwrap();
    let ch = project.get_chapter("dashes").unwrap();
    assert!(ch.body.contains("---"));
    assert!(ch.body.contains("scene break"));
}

#[test]
fn roundtrip_multiple_entity_schemas() {
    let src = TempDir::new().unwrap();
    std::fs::create_dir_all(src.path().join("entities/character")).unwrap();
    std::fs::create_dir_all(src.path().join("entities/place")).unwrap();
    std::fs::create_dir_all(src.path().join("entities/item")).unwrap();

    for (schema, slug, title) in &[
        ("character", "hero", "The Hero"),
        ("character", "villain", "The Villain"),
        ("place", "castle", "The Castle"),
        ("item", "sword", "Magic Sword"),
    ] {
        std::fs::write(
            src.path().join(format!("entities/{}/{}.md", schema, slug)),
            format!(
                "---\ntitle: {}\nslug: {}\nschemaType: {}\n---\nDescription of {}.",
                title, slug, schema, title
            ),
        )
        .unwrap();
    }

    let project = import_from_files(src.path()).unwrap();
    assert_eq!(project.list_entities("character").unwrap().len(), 2);
    assert_eq!(project.list_entities("place").unwrap().len(), 1);
    assert_eq!(project.list_entities("item").unwrap().len(), 1);

    let dst = TempDir::new().unwrap();
    export_to_files(&project, dst.path()).unwrap();

    assert!(dst.path().join("entities/character/hero.md").exists());
    assert!(dst.path().join("entities/place/castle.md").exists());
    assert!(dst.path().join("entities/item/sword.md").exists());
}

// ── CRDT sync tests ──────────────────────────────────────────────────────────

#[test]
fn two_doc_sync_after_import() {
    let src = TempDir::new().unwrap();
    create_medium_project(src.path());

    let project1 = import_from_files(src.path()).unwrap();

    // Export as snapshot, import into new doc
    let snapshot = project1.export_snapshot().unwrap();
    let project2 = CrdtProject::from_snapshot(Uuid::new_v4(), &snapshot).unwrap();

    // Both should have same data
    let ch1_a = project1.get_chapter("chapter-one").unwrap();
    let ch1_b = project2.get_chapter("chapter-one").unwrap();
    assert_eq!(ch1_a.title, ch1_b.title);
    assert_eq!(ch1_a.body, ch1_b.body);
    assert_eq!(ch1_a.pov, ch1_b.pov);
}

#[test]
fn concurrent_edit_after_import() {
    let src = TempDir::new().unwrap();
    create_minimal_project(src.path());

    let project1 = import_from_files(src.path()).unwrap();
    let snapshot = project1.export_snapshot().unwrap();
    let project2 = CrdtProject::from_snapshot(Uuid::new_v4(), &snapshot).unwrap();

    let vv1 = project1.version_vector();
    let vv2 = project2.version_vector();

    // Project1 adds a chapter
    project1
        .import_chapter(
            "new-chapter",
            "New Chapter",
            "draft",
            None,
            None,
            None,
            "Content A.",
        )
        .unwrap();

    // Project2 adds a different chapter
    project2
        .import_chapter(
            "other-chapter",
            "Other Chapter",
            "draft",
            None,
            None,
            None,
            "Content B.",
        )
        .unwrap();

    // Sync
    let updates1 = project1.export_updates(&vv1).unwrap();
    let updates2 = project2.export_updates(&vv2).unwrap();
    project1.import_updates(&updates2).unwrap();
    project2.import_updates(&updates1).unwrap();

    // Both should have all chapters (original + 2 new ones)
    let chapters1 = project1.list_chapters().unwrap();
    let chapters2 = project2.list_chapters().unwrap();
    assert_eq!(chapters1.len(), 3);
    assert_eq!(chapters2.len(), 3);
}

// ── Double round-trip test ───────────────────────────────────────────────────

#[test]
fn double_roundtrip_produces_identical_output() {
    let src = TempDir::new().unwrap();
    create_medium_project(src.path());

    // First round-trip
    let project1 = import_from_files(src.path()).unwrap();
    let dst1 = TempDir::new().unwrap();
    export_to_files(&project1, dst1.path()).unwrap();

    // Second round-trip (import from first export)
    let project2 = import_from_files(dst1.path()).unwrap();
    let dst2 = TempDir::new().unwrap();
    export_to_files(&project2, dst2.path()).unwrap();

    // Compare key files between dst1 and dst2
    let manifest1 =
        std::fs::read_to_string(dst1.path().join("manuscript/manuscript.yaml")).unwrap();
    let manifest2 =
        std::fs::read_to_string(dst2.path().join("manuscript/manuscript.yaml")).unwrap();
    assert_eq!(
        manifest1, manifest2,
        "manuscript.yaml should be identical after double round-trip"
    );

    // Compare a chapter file
    let ch1 = std::fs::read_to_string(dst1.path().join("manuscript/chapter-one.md")).unwrap();
    let ch2 = std::fs::read_to_string(dst2.path().join("manuscript/chapter-one.md")).unwrap();
    assert_eq!(
        ch1, ch2,
        "Chapter files should be identical after double round-trip"
    );
}

// ── Property-based tests ─────────────────────────────────────────────────────

/// Generate a valid slug (alphanumeric + hyphens, non-empty, starts with letter).
fn arb_slug() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,20}".prop_filter("slug must not end with hyphen", |s| {
        !s.ends_with('-') && !s.contains("--")
    })
}

/// Generate a valid title (YAML trims trailing whitespace, so we do too).
fn arb_title() -> impl Strategy<Value = String> {
    "[A-Z][a-zA-Z0-9 ]{0,40}".prop_map(|s| s.trim_end().to_string())
}

/// Generate body text (plain text, no YAML frontmatter delimiters at line start).
fn arb_body() -> impl Strategy<Value = String> {
    prop::collection::vec("[a-zA-Z0-9 .,!?;:'\"-]{1,80}", 0..5)
        .prop_map(|lines| lines.join("\n").trim_end().to_string())
}

/// Generate a non-whitespace-only field value (YAML trims trailing whitespace).
fn arb_field_value() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z ]{0,19}".prop_map(|s| s.trim_end().to_string())
}

/// Generate a chapter status.
fn arb_status() -> impl Strategy<Value = String> {
    prop_oneof![Just("draft"), Just("revised"), Just("final"),].prop_map(String::from)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// Property: random chapter CRUD maintains internal consistency.
    /// Creating N chapters and listing them should return exactly N.
    #[test]
    fn prop_chapter_crud_consistency(
        slugs in prop::collection::hash_set(arb_slug(), 1..8),
    ) {
        let project = CrdtProject::new(Uuid::new_v4());
        let slugs: Vec<_> = slugs.into_iter().collect();

        for (i, slug) in slugs.iter().enumerate() {
            project.import_chapter(
                slug,
                &format!("Title {}", i),
                "draft",
                None,
                None,
                None,
                &format!("Body of chapter {}.", i),
            ).unwrap();
        }

        let chapters = project.list_chapters().unwrap();
        prop_assert_eq!(chapters.len(), slugs.len());

        // Every slug should be retrievable
        for slug in &slugs {
            let ch = project.get_chapter(slug).unwrap();
            prop_assert_eq!(&ch.slug, slug);
        }
    }

    /// Property: import → export → import produces identical CRDT state.
    #[test]
    fn prop_roundtrip_preserves_chapter_data(
        slug in arb_slug(),
        title in arb_title(),
        status in arb_status(),
        body in arb_body(),
    ) {
        // Create project files
        let src = TempDir::new().unwrap();
        std::fs::create_dir_all(src.path().join("manuscript")).unwrap();
        std::fs::write(
            src.path().join("manuscript/manuscript.yaml"),
            format!("chapters:\n  - {}\n", slug),
        ).unwrap();
        std::fs::write(
            src.path().join(format!("manuscript/{}.md", slug)),
            format!("---\ntitle: {}\nslug: {}\nstatus: {}\norder: 0\n---\n{}", title, slug, status, body),
        ).unwrap();

        // Import
        let project = import_from_files(src.path()).unwrap();
        let ch = project.get_chapter(&slug).unwrap();
        prop_assert_eq!(&ch.title, &title);
        prop_assert_eq!(&ch.status, &status);
        prop_assert_eq!(&ch.body, &body);

        // Export and re-import
        let dst = TempDir::new().unwrap();
        export_to_files(&project, dst.path()).unwrap();

        let project2 = import_from_files(dst.path()).unwrap();
        let ch2 = project2.get_chapter(&slug).unwrap();
        prop_assert_eq!(ch.title, ch2.title);
        prop_assert_eq!(ch.status, ch2.status);
        prop_assert_eq!(ch.body, ch2.body);
    }

    /// Property: snapshot round-trip preserves all data for any set of chapters.
    #[test]
    fn prop_snapshot_preserves_chapters(
        slugs in prop::collection::hash_set(arb_slug(), 1..6),
    ) {
        let project = CrdtProject::new(Uuid::new_v4());
        let slugs: Vec<_> = slugs.into_iter().collect();

        for (i, slug) in slugs.iter().enumerate() {
            project.import_chapter(slug, &format!("T{}", i), "draft", None, None, None, "body").unwrap();
        }

        let snapshot = project.export_snapshot().unwrap();
        let restored = CrdtProject::from_snapshot(Uuid::new_v4(), &snapshot).unwrap();

        let orig_chapters = project.list_chapters().unwrap();
        let rest_chapters = restored.list_chapters().unwrap();
        prop_assert_eq!(orig_chapters.len(), rest_chapters.len());

        for slug in &slugs {
            let orig = project.get_chapter(slug).unwrap();
            let rest = restored.get_chapter(slug).unwrap();
            prop_assert_eq!(orig.title, rest.title);
            prop_assert_eq!(orig.body, rest.body);
        }
    }

    /// Property: concurrent edits on two docs converge after sync.
    #[test]
    fn prop_concurrent_edits_converge(
        slug_a in arb_slug(),
        slug_b in arb_slug().prop_filter("must differ from slug_a", |s| s.len() > 1),
        body_a in arb_body(),
        body_b in arb_body(),
    ) {
        // Skip when slugs collide
        prop_assume!(slug_a != slug_b);

        let doc1 = CrdtProject::new(Uuid::new_v4());
        let snapshot = doc1.export_snapshot().unwrap();
        let doc2 = CrdtProject::from_snapshot(Uuid::new_v4(), &snapshot).unwrap();

        let vv1 = doc1.version_vector();
        let vv2 = doc2.version_vector();

        doc1.import_chapter(&slug_a, "A", "draft", None, None, None, &body_a).unwrap();
        doc2.import_chapter(&slug_b, "B", "draft", None, None, None, &body_b).unwrap();

        let updates1 = doc1.export_updates(&vv1).unwrap();
        let updates2 = doc2.export_updates(&vv2).unwrap();
        doc1.import_updates(&updates2).unwrap();
        doc2.import_updates(&updates1).unwrap();

        // Both should have both chapters
        let chs1 = doc1.list_chapters().unwrap();
        let chs2 = doc2.list_chapters().unwrap();
        prop_assert_eq!(chs1.len(), chs2.len());

        // Data should match
        let a1 = doc1.get_chapter(&slug_a).unwrap();
        let a2 = doc2.get_chapter(&slug_a).unwrap();
        prop_assert_eq!(a1.body, a2.body);

        let b1 = doc1.get_chapter(&slug_b).unwrap();
        let b2 = doc2.get_chapter(&slug_b).unwrap();
        prop_assert_eq!(b1.body, b2.body);
    }

    /// Property: entity CRUD operations maintain consistency.
    #[test]
    fn prop_entity_crud_consistency(
        slugs in prop::collection::hash_set(arb_slug(), 1..6),
    ) {
        let project = CrdtProject::new(Uuid::new_v4());
        let slugs: Vec<_> = slugs.into_iter().collect();

        for (i, slug) in slugs.iter().enumerate() {
            let fields = serde_json::json!({ "name": format!("Entity {}", i) });
            project.create_entity("character", slug, &format!("Entity {}", i), fields).unwrap();
        }

        let entities = project.list_entities("character").unwrap();
        prop_assert_eq!(entities.len(), slugs.len());

        for slug in &slugs {
            let entity = project.get_entity("character", slug).unwrap();
            prop_assert_eq!(&entity.slug, slug);
        }
    }

    /// Property: note CRUD operations maintain consistency.
    #[test]
    fn prop_note_crud_consistency(
        slugs in prop::collection::hash_set(arb_slug(), 1..6),
    ) {
        let project = CrdtProject::new(Uuid::new_v4());
        let slugs: Vec<_> = slugs.into_iter().collect();

        for (i, slug) in slugs.iter().enumerate() {
            project.import_note(
                slug,
                &format!("Note {}", i),
                None,
                None,
                None,
                None,
                "note body",
            ).unwrap();
        }

        let notes = project.list_notes().unwrap();
        prop_assert_eq!(notes.len(), slugs.len());

        for slug in &slugs {
            let note = project.get_note(slug).unwrap();
            prop_assert_eq!(&note.slug, slug);
        }
    }

    /// Property: delete then list should have one fewer item.
    #[test]
    fn prop_delete_chapter_reduces_count(
        slugs in prop::collection::hash_set(arb_slug(), 2..6),
    ) {
        let project = CrdtProject::new(Uuid::new_v4());
        let slugs: Vec<_> = slugs.into_iter().collect();

        for (i, slug) in slugs.iter().enumerate() {
            project.import_chapter(slug, &format!("T{}", i), "draft", None, None, None, "body").unwrap();
        }

        let before = project.list_chapters().unwrap().len();
        project.delete_chapter(&slugs[0]).unwrap();
        let after = project.list_chapters().unwrap().len();

        prop_assert_eq!(after, before - 1);
        prop_assert!(project.get_chapter(&slugs[0]).is_err());
    }

    /// Property: entity round-trip through export preserves field data.
    #[test]
    fn prop_entity_export_roundtrip(
        slug in arb_slug(),
        title in arb_title(),
        role in arb_field_value(),
    ) {
        let src = TempDir::new().unwrap();
        std::fs::create_dir_all(src.path().join("entities/character")).unwrap();
        std::fs::write(
            src.path().join(format!("entities/character/{}.md", slug)),
            format!("---\ntitle: {}\nslug: {}\nschemaType: character\nfields:\n  role: {}\n---\nDescription.", title, slug, role),
        ).unwrap();

        let project = import_from_files(src.path()).unwrap();
        let entity = project.get_entity("character", &slug).unwrap();
        prop_assert_eq!(&entity.title, &title);

        let fields = entity.fields.as_object().unwrap();
        prop_assert_eq!(fields.get("role").unwrap().as_str().unwrap(), role.as_str());

        // Export and re-import
        let dst = TempDir::new().unwrap();
        export_to_files(&project, dst.path()).unwrap();
        let project2 = import_from_files(dst.path()).unwrap();
        let entity2 = project2.get_entity("character", &slug).unwrap();
        prop_assert_eq!(entity.title, entity2.title);
    }
}
