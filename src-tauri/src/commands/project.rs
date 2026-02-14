use std::path::PathBuf;

use crate::error::AppError;
use crate::models::project::ProjectManifest;
use crate::services::slug_service::slugify;
use crate::services::yaml_service::{read_yaml, write_yaml};

/// Default entity schema content for a given entity type.
fn default_schema_yaml(entity_type: &str) -> String {
    format!(
        "name: {entity_type}\nentity_type: {entity_type}\nfields: []\nspider_axes: []\n",
        entity_type = entity_type
    )
}

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

    // Write default entity schemas
    let entity_types = ["character", "place", "item", "idea"];
    for entity_type in &entity_types {
        let schema_path = project_root
            .join("schemas")
            .join(format!("{}.yaml", entity_type));
        std::fs::write(&schema_path, default_schema_yaml(entity_type))?;
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
            assert!(
                content.contains(&format!("entity_type: {}", entity_type)),
                "Schema {} missing entity_type field",
                entity_type
            );
            assert!(
                content.contains("fields: []"),
                "Schema {} missing fields",
                entity_type
            );
            assert!(
                content.contains("spider_axes: []"),
                "Schema {} missing spider_axes",
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
}
