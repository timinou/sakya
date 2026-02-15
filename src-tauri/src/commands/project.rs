use std::path::{Path, PathBuf};

use chrono::Utc;
use tauri::Manager;

use crate::error::AppError;
use crate::models::project::{ProjectManifest, RecentProject};
use crate::services::slug_service::slugify;
use crate::services::yaml_service::{read_yaml, write_yaml};

use super::entity::default_schemas;

/// Create a new Sakya project at `path/slugified-name`.
///
/// Generates the full folder structure, default entity schemas,
/// empty manuscript.yaml, empty notes.yaml, and the sakya.yaml manifest.
#[tauri::command]
pub fn create_project(name: String, path: String) -> Result<ProjectManifest, AppError> {
    let slug = slugify(&name);
    let project_root = PathBuf::from(&path).join(&slug);

    // Error if folder already exists
    if project_root.exists() {
        return Err(AppError::InvalidOperation(format!(
            "Directory already exists: {}",
            project_root.display()
        )));
    }

    // Create project directories
    let dirs = ["schemas", "entities", "manuscript", "notes", ".sakya"];
    for d in &dirs {
        std::fs::create_dir_all(project_root.join(d))?;
    }

    // Write default entity schemas (rich defaults with fields, spider axes, etc.)
    for schema in default_schemas() {
        let schema_path = project_root
            .join("schemas")
            .join(format!("{}.yaml", schema.entity_type));
        write_yaml(&schema_path, &schema)?;
    }

    // Write empty manuscript.yaml
    let manuscript_path = project_root.join("manuscript.yaml");
    std::fs::write(&manuscript_path, "chapters: []\n")?;

    // Write empty notes.yaml
    let notes_path = project_root.join("notes.yaml");
    std::fs::write(&notes_path, "notes: []\n")?;

    // Create and write manifest
    let manifest = ProjectManifest::new(name);
    let manifest_path = project_root.join("sakya.yaml");
    write_yaml(&manifest_path, &manifest)?;

    Ok(manifest)
}

/// Open an existing Sakya project by reading its sakya.yaml manifest.
#[tauri::command]
pub fn open_project(path: String) -> Result<ProjectManifest, AppError> {
    let project_root = PathBuf::from(&path);

    if !project_root.exists() {
        return Err(AppError::NotFound(format!(
            "Project path does not exist: {}",
            project_root.display()
        )));
    }

    let manifest_path = project_root.join("sakya.yaml");
    let manifest: ProjectManifest = read_yaml(&manifest_path)?;

    Ok(manifest)
}

/// Save an updated project manifest to sakya.yaml at the given path.
#[tauri::command]
pub fn save_project_manifest(path: String, manifest: ProjectManifest) -> Result<(), AppError> {
    let project_root = PathBuf::from(&path);
    let manifest_path = project_root.join("sakya.yaml");
    write_yaml(&manifest_path, &manifest)?;
    Ok(())
}

// ── recent projects ────────────────────────────────────────────────────

const MAX_RECENT_PROJECTS: usize = 10;

/// Get the path to the recent-projects.json file.
fn recent_projects_file(config_dir: &Path) -> PathBuf {
    config_dir.join("recent-projects.json")
}

/// Read recent projects from disk. Returns empty vec if file missing or corrupt.
fn read_recent_projects(config_dir: &Path) -> Vec<RecentProject> {
    let file_path = recent_projects_file(config_dir);
    match std::fs::read_to_string(&file_path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

/// Write recent projects list to disk.
fn write_recent_projects(config_dir: &Path, projects: &[RecentProject]) -> Result<(), AppError> {
    std::fs::create_dir_all(config_dir)?;
    let content = serde_json::to_string_pretty(projects)?;
    std::fs::write(recent_projects_file(config_dir), content)?;
    Ok(())
}

/// Validate that a path still contains a sakya.yaml (i.e., is a valid Sakya project).
fn is_valid_project_path(path: &str) -> bool {
    PathBuf::from(path).join("sakya.yaml").exists()
}

/// Core logic for listing recent projects: read, validate paths, prune stale, write back.
fn list_recent_projects_impl(config_dir: &Path) -> Result<Vec<RecentProject>, AppError> {
    let projects = read_recent_projects(config_dir);
    let valid: Vec<RecentProject> = projects
        .into_iter()
        .filter(|p| is_valid_project_path(&p.path))
        .collect();
    // Write back pruned list if it changed
    write_recent_projects(config_dir, &valid)?;
    Ok(valid)
}

/// Core logic for adding a recent project: upsert, sort by recency, cap at MAX.
fn add_recent_project_impl(
    config_dir: &Path,
    name: String,
    path: String,
) -> Result<Vec<RecentProject>, AppError> {
    let mut projects = read_recent_projects(config_dir);

    // Remove existing entry with same path (upsert)
    projects.retain(|p| p.path != path);

    // Add new entry at the front
    projects.insert(
        0,
        RecentProject {
            name,
            path,
            last_opened: Utc::now(),
        },
    );

    // Cap at MAX_RECENT_PROJECTS
    projects.truncate(MAX_RECENT_PROJECTS);

    write_recent_projects(config_dir, &projects)?;
    Ok(projects)
}

/// Core logic for removing a recent project by path.
fn remove_recent_project_impl(
    config_dir: &Path,
    path: String,
) -> Result<Vec<RecentProject>, AppError> {
    let mut projects = read_recent_projects(config_dir);
    projects.retain(|p| p.path != path);
    write_recent_projects(config_dir, &projects)?;
    Ok(projects)
}

/// List recent projects, pruning entries whose paths no longer contain sakya.yaml.
#[tauri::command]
pub fn list_recent_projects(app: tauri::AppHandle) -> Result<Vec<RecentProject>, AppError> {
    let config_dir = app.path().app_data_dir().map_err(|e| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            e.to_string(),
        ))
    })?;
    list_recent_projects_impl(&config_dir)
}

/// Add (or update) a project in the recent projects list.
#[tauri::command]
pub fn add_recent_project(
    app: tauri::AppHandle,
    name: String,
    path: String,
) -> Result<Vec<RecentProject>, AppError> {
    let config_dir = app.path().app_data_dir().map_err(|e| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            e.to_string(),
        ))
    })?;
    add_recent_project_impl(&config_dir, name, path)
}

/// Remove a project from the recent projects list.
#[tauri::command]
pub fn remove_recent_project(
    app: tauri::AppHandle,
    path: String,
) -> Result<Vec<RecentProject>, AppError> {
    let config_dir = app.path().app_data_dir().map_err(|e| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            e.to_string(),
        ))
    })?;
    remove_recent_project_impl(&config_dir, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{setup_test_dir, setup_test_project};

    // ── create_project ──────────────────────────────────────────────

    #[test]
    fn create_project_creates_all_directories() {
        let dir = setup_test_dir();
        let parent = dir.path().to_str().unwrap().to_string();

        create_project("My Novel".to_string(), parent.clone()).unwrap();

        let root = dir.path().join("my-novel");
        assert!(root.join("schemas").is_dir());
        assert!(root.join("entities").is_dir());
        assert!(root.join("manuscript").is_dir());
        assert!(root.join("notes").is_dir());
        assert!(root.join(".sakya").is_dir());
    }

    #[test]
    fn create_project_writes_sakya_yaml() {
        let dir = setup_test_dir();
        let parent = dir.path().to_str().unwrap().to_string();

        create_project("My Novel".to_string(), parent.clone()).unwrap();

        let root = dir.path().join("my-novel");
        let manifest_path = root.join("sakya.yaml");
        assert!(manifest_path.is_file());

        let manifest: ProjectManifest = read_yaml(&manifest_path).unwrap();
        assert_eq!(manifest.name, "My Novel");
        assert_eq!(manifest.version, "0.1.0");
    }

    #[test]
    fn create_project_writes_default_entity_schemas() {
        let dir = setup_test_dir();
        let parent = dir.path().to_str().unwrap().to_string();

        create_project("My Novel".to_string(), parent.clone()).unwrap();

        let root = dir.path().join("my-novel");
        let schema_types = ["character", "place", "item", "idea"];
        for entity_type in &schema_types {
            let schema_path = root.join("schemas").join(format!("{}.yaml", entity_type));
            assert!(schema_path.is_file(), "Missing schema: {}", entity_type);

            let content = std::fs::read_to_string(&schema_path).unwrap();
            // Rich schemas use camelCase (serde rename_all)
            assert!(
                content.contains(&format!("entityType: {}", entity_type)),
                "Schema {} missing entityType field",
                entity_type
            );
            // Rich schemas have actual fields and spider axes
            assert!(
                content.contains("fields:"),
                "Schema {} missing fields",
                entity_type
            );
            assert!(
                content.contains("spiderAxes:"),
                "Schema {} missing spiderAxes",
                entity_type
            );
        }
    }

    #[test]
    fn create_project_writes_manuscript_yaml() {
        let dir = setup_test_dir();
        let parent = dir.path().to_str().unwrap().to_string();

        create_project("My Novel".to_string(), parent.clone()).unwrap();

        let root = dir.path().join("my-novel");
        let manuscript_path = root.join("manuscript.yaml");
        assert!(manuscript_path.is_file());

        let content = std::fs::read_to_string(&manuscript_path).unwrap();
        assert!(content.contains("chapters: []"));
    }

    #[test]
    fn create_project_writes_notes_yaml() {
        let dir = setup_test_dir();
        let parent = dir.path().to_str().unwrap().to_string();

        create_project("My Novel".to_string(), parent.clone()).unwrap();

        let root = dir.path().join("my-novel");
        let notes_path = root.join("notes.yaml");
        assert!(notes_path.is_file());

        let content = std::fs::read_to_string(&notes_path).unwrap();
        assert!(content.contains("notes: []"));
    }

    #[test]
    fn create_project_returns_valid_manifest() {
        let dir = setup_test_dir();
        let parent = dir.path().to_str().unwrap().to_string();

        let manifest = create_project("My Novel".to_string(), parent.clone()).unwrap();

        assert_eq!(manifest.name, "My Novel");
        assert_eq!(manifest.version, "0.1.0");
        assert!(manifest.author.is_none());
        assert!(manifest.description.is_none());
        assert!(manifest.created_at <= chrono::Utc::now());
        assert!(manifest.updated_at <= chrono::Utc::now());
    }

    #[test]
    fn create_project_errors_when_directory_exists() {
        let dir = setup_test_dir();
        let parent = dir.path().to_str().unwrap().to_string();

        // First creation should succeed
        create_project("My Novel".to_string(), parent.clone()).unwrap();

        // Second creation should fail
        let result = create_project("My Novel".to_string(), parent.clone());
        assert!(result.is_err());

        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("already exists"),
            "Expected 'already exists' error, got: {}",
            err_msg
        );
    }

    #[test]
    fn create_project_slugifies_name() {
        let dir = setup_test_dir();
        let parent = dir.path().to_str().unwrap().to_string();

        create_project("The Great Gatsby".to_string(), parent.clone()).unwrap();

        let root = dir.path().join("the-great-gatsby");
        assert!(root.is_dir());
    }

    #[test]
    fn create_project_with_special_characters_in_name() {
        let dir = setup_test_dir();
        let parent = dir.path().to_str().unwrap().to_string();

        let manifest = create_project("O'Brien & Friends".to_string(), parent.clone()).unwrap();

        assert_eq!(manifest.name, "O'Brien & Friends");
        let root = dir.path().join("o-brien-friends");
        assert!(root.is_dir());
    }

    // ── open_project ────────────────────────────────────────────────

    #[test]
    fn open_project_reads_valid_manifest() {
        let (_dir, root) = setup_test_project();
        let path = root.to_str().unwrap().to_string();

        // Write a proper manifest with all required fields
        let manifest = ProjectManifest::new("Test Project".to_string());
        write_yaml(&root.join("sakya.yaml"), &manifest).unwrap();

        let loaded = open_project(path).unwrap();
        assert_eq!(loaded.name, "Test Project");
        assert_eq!(loaded.version, "0.1.0");
    }

    #[test]
    fn open_project_errors_on_nonexistent_path() {
        let result = open_project("/nonexistent/path/to/project".to_string());
        assert!(result.is_err());

        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("does not exist"),
            "Expected 'does not exist' error, got: {}",
            err_msg
        );
    }

    #[test]
    fn open_project_errors_on_missing_sakya_yaml() {
        let dir = setup_test_dir();
        let path = dir.path().to_str().unwrap().to_string();

        // Directory exists but has no sakya.yaml
        let result = open_project(path);
        assert!(result.is_err());

        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("not found") || err_msg.contains("Not found"),
            "Expected 'not found' error, got: {}",
            err_msg
        );
    }

    #[test]
    fn open_project_errors_on_corrupt_manifest() {
        let dir = setup_test_dir();
        let root = dir.path().to_path_buf();

        // Write corrupt YAML
        std::fs::write(root.join("sakya.yaml"), "{{{{invalid yaml!!!!").unwrap();

        let result = open_project(root.to_str().unwrap().to_string());
        assert!(result.is_err());
    }

    // ── save_project_manifest ───────────────────────────────────────

    #[test]
    fn save_project_manifest_round_trips() {
        let (_dir, root) = setup_test_project();
        let path = root.to_str().unwrap().to_string();

        let mut manifest = ProjectManifest::new("Updated Name".to_string());
        manifest.author = Some("Jane Doe".to_string());
        manifest.description = Some("A great novel".to_string());

        save_project_manifest(path.clone(), manifest.clone()).unwrap();

        let loaded = open_project(path).unwrap();
        assert_eq!(loaded.name, "Updated Name");
        assert_eq!(loaded.author, Some("Jane Doe".to_string()));
        assert_eq!(loaded.description, Some("A great novel".to_string()));
    }

    #[test]
    fn save_project_manifest_overwrites_existing() {
        let (_dir, root) = setup_test_project();
        let path = root.to_str().unwrap().to_string();

        // Write a proper manifest first
        let manifest = ProjectManifest::new("First Name".to_string());
        save_project_manifest(path.clone(), manifest).unwrap();

        // Overwrite with new data
        let manifest2 = ProjectManifest::new("Second Name".to_string());
        save_project_manifest(path.clone(), manifest2).unwrap();

        let loaded = open_project(path).unwrap();
        assert_eq!(loaded.name, "Second Name");
    }

    // ── create + open integration ───────────────────────────────────

    #[test]
    fn create_then_open_round_trips() {
        let dir = setup_test_dir();
        let parent = dir.path().to_str().unwrap().to_string();

        let created = create_project("Round Trip".to_string(), parent.clone()).unwrap();

        let project_path = dir.path().join("round-trip");
        let opened = open_project(project_path.to_str().unwrap().to_string()).unwrap();

        assert_eq!(created.name, opened.name);
        assert_eq!(created.version, opened.version);
    }

    // ── legacy YAML backward compatibility ─────────────────────────

    #[test]
    fn open_project_handles_legacy_yaml_without_version() {
        let dir = setup_test_dir();
        let root = dir.path().to_path_buf();

        // Legacy YAML: only name, no version/timestamps
        std::fs::write(
            root.join("sakya.yaml"),
            "name: Old Novel\nauthor: Legacy Author\n",
        )
        .unwrap();

        let manifest = open_project(root.to_str().unwrap().to_string()).unwrap();
        assert_eq!(manifest.name, "Old Novel");
        assert_eq!(manifest.version, "0.1.0");
        assert_eq!(manifest.author, Some("Legacy Author".to_string()));
        assert!(manifest.created_at <= chrono::Utc::now());
        assert!(manifest.updated_at <= chrono::Utc::now());
    }

    #[test]
    fn open_project_handles_legacy_yaml_with_extra_fields() {
        let dir = setup_test_dir();
        let root = dir.path().to_path_buf();

        // Exact copy of the example project YAML (has extra fields like genre, schemas, created)
        let legacy_yaml = r#"name: The Warmth of Distant Things
author: Sakya Example Project
created: 2026-02-14
genre: Magical Realism
description: >
  A person in their thirties looks back at their twenties—the friendships,
  the bars, the late nights, the slow drift apart.
schemas:
  - character
  - place
  - item
  - idea
"#;
        std::fs::write(root.join("sakya.yaml"), legacy_yaml).unwrap();

        let manifest = open_project(root.to_str().unwrap().to_string()).unwrap();
        assert_eq!(manifest.name, "The Warmth of Distant Things");
        assert_eq!(manifest.version, "0.1.0"); // defaulted
        assert_eq!(manifest.author, Some("Sakya Example Project".to_string()));
    }

    #[test]
    fn open_project_preserves_explicit_version() {
        let dir = setup_test_dir();
        let root = dir.path().to_path_buf();

        let yaml = r#"name: Versioned Novel
version: "2.0.0"
createdAt: "2025-01-01T00:00:00Z"
updatedAt: "2025-06-15T12:00:00Z"
"#;
        std::fs::write(root.join("sakya.yaml"), yaml).unwrap();

        let manifest = open_project(root.to_str().unwrap().to_string()).unwrap();
        assert_eq!(manifest.name, "Versioned Novel");
        assert_eq!(manifest.version, "2.0.0");
        assert_eq!(
            manifest.created_at,
            "2025-01-01T00:00:00Z"
                .parse::<chrono::DateTime<chrono::Utc>>()
                .unwrap()
        );
    }

    // ── recent projects ────────────────────────────────────────────

    #[test]
    fn list_recent_projects_returns_empty_on_first_run() {
        let dir = setup_test_dir();
        let config_dir = dir.path().to_path_buf();
        let result = list_recent_projects_impl(&config_dir).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn add_recent_project_creates_entry() {
        let dir = setup_test_dir();
        let config_dir = dir.path().to_path_buf();

        // Create a fake project with sakya.yaml
        let project_dir = config_dir.join("my-project");
        std::fs::create_dir_all(&project_dir).unwrap();
        std::fs::write(project_dir.join("sakya.yaml"), "name: Test\n").unwrap();

        let result = add_recent_project_impl(
            &config_dir,
            "My Project".to_string(),
            project_dir.to_str().unwrap().to_string(),
        )
        .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "My Project");
        assert_eq!(result[0].path, project_dir.to_str().unwrap());
    }

    #[test]
    fn add_recent_project_upserts_existing_entry() {
        let dir = setup_test_dir();
        let config_dir = dir.path().to_path_buf();

        let path = "/some/path".to_string();

        add_recent_project_impl(&config_dir, "Old Name".to_string(), path.clone()).unwrap();
        let result =
            add_recent_project_impl(&config_dir, "New Name".to_string(), path.clone()).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "New Name");
    }

    #[test]
    fn add_recent_project_caps_at_max() {
        let dir = setup_test_dir();
        let config_dir = dir.path().to_path_buf();

        for i in 0..15 {
            add_recent_project_impl(
                &config_dir,
                format!("Project {}", i),
                format!("/path/{}", i),
            )
            .unwrap();
        }

        let result = read_recent_projects(&config_dir);
        assert_eq!(result.len(), MAX_RECENT_PROJECTS);
        // Most recent should be first
        assert_eq!(result[0].name, "Project 14");
    }

    #[test]
    fn add_recent_project_moves_existing_to_front() {
        let dir = setup_test_dir();
        let config_dir = dir.path().to_path_buf();

        add_recent_project_impl(&config_dir, "First".to_string(), "/path/1".to_string()).unwrap();
        add_recent_project_impl(&config_dir, "Second".to_string(), "/path/2".to_string()).unwrap();
        let result = add_recent_project_impl(
            &config_dir,
            "First Updated".to_string(),
            "/path/1".to_string(),
        )
        .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "First Updated");
        assert_eq!(result[0].path, "/path/1");
        assert_eq!(result[1].name, "Second");
    }

    #[test]
    fn remove_recent_project_removes_entry() {
        let dir = setup_test_dir();
        let config_dir = dir.path().to_path_buf();

        add_recent_project_impl(&config_dir, "Keep".to_string(), "/keep".to_string()).unwrap();
        add_recent_project_impl(&config_dir, "Remove".to_string(), "/remove".to_string()).unwrap();

        let result = remove_recent_project_impl(&config_dir, "/remove".to_string()).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Keep");
    }

    #[test]
    fn remove_recent_project_noop_for_missing_path() {
        let dir = setup_test_dir();
        let config_dir = dir.path().to_path_buf();

        add_recent_project_impl(&config_dir, "Test".to_string(), "/test".to_string()).unwrap();
        let result = remove_recent_project_impl(&config_dir, "/nonexistent".to_string()).unwrap();

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn list_recent_projects_prunes_stale_paths() {
        let dir = setup_test_dir();
        let config_dir = dir.path().to_path_buf();

        // Create a valid project
        let valid_dir = config_dir.join("valid-project");
        std::fs::create_dir_all(&valid_dir).unwrap();
        std::fs::write(valid_dir.join("sakya.yaml"), "name: Valid\n").unwrap();

        // Add both a valid and invalid path
        add_recent_project_impl(
            &config_dir,
            "Valid".to_string(),
            valid_dir.to_str().unwrap().to_string(),
        )
        .unwrap();
        add_recent_project_impl(
            &config_dir,
            "Stale".to_string(),
            "/nonexistent/stale/path".to_string(),
        )
        .unwrap();

        // list should prune the stale entry
        let result = list_recent_projects_impl(&config_dir).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Valid");
    }

    #[test]
    fn list_recent_projects_handles_corrupt_json() {
        let dir = setup_test_dir();
        let config_dir = dir.path().to_path_buf();

        // Write corrupt JSON
        std::fs::write(recent_projects_file(&config_dir), "{{not valid json!!!").unwrap();

        let result = list_recent_projects_impl(&config_dir).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn recent_projects_file_persists_across_calls() {
        let dir = setup_test_dir();
        let config_dir = dir.path().to_path_buf();

        add_recent_project_impl(
            &config_dir,
            "Persistent".to_string(),
            "/persist".to_string(),
        )
        .unwrap();

        // Read from a fresh call (simulating app restart)
        let projects = read_recent_projects(&config_dir);
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Persistent");
    }
}
