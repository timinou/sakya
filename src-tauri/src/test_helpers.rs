#![cfg(test)]

use std::path::PathBuf;
use tempfile::TempDir;

/// Creates a temporary directory for test isolation.
pub fn setup_test_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

/// Creates a mock Sakya project structure in a temp directory.
/// Returns the TempDir (keeps it alive) and the project root path.
pub fn setup_test_project() -> (TempDir, PathBuf) {
    let dir = setup_test_dir();
    let root = dir.path().to_path_buf();

    // Create standard Sakya project directories
    let dirs = ["schemas", "entities", "manuscript"];
    for d in &dirs {
        std::fs::create_dir_all(root.join(d)).expect("Failed to create project directory");
    }

    // Create a minimal sakya.yaml
    let manifest = r#"name: "Test Project"
version: "0.1.0"
"#;
    std::fs::write(root.join("sakya.yaml"), manifest).expect("Failed to write sakya.yaml");

    (dir, root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_test_dir() {
        let dir = setup_test_dir();
        assert!(dir.path().exists());
    }

    #[test]
    fn test_setup_test_project() {
        let (_dir, root) = setup_test_project();
        assert!(root.join("schemas").is_dir());
        assert!(root.join("entities").is_dir());
        assert!(root.join("manuscript").is_dir());
        assert!(root.join("sakya.yaml").is_file());

        let content = std::fs::read_to_string(root.join("sakya.yaml")).unwrap();
        assert!(content.contains("Test Project"));
    }
}
