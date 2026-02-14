use crate::error::AppError;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::path::Path;

/// Read and deserialize a YAML file.
pub fn read_yaml<T: DeserializeOwned>(path: &Path) -> Result<T, AppError> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AppError::NotFound(format!("File not found: {}", path.display()))
        } else {
            AppError::Io(e)
        }
    })?;
    let value: T = serde_yaml::from_str(&content)?;
    Ok(value)
}

/// Serialize and write a value to a YAML file.
pub fn write_yaml<T: Serialize>(path: &Path, value: &T) -> Result<(), AppError> {
    let content = serde_yaml::to_string(value)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::TempDir;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Config {
        name: String,
        count: i32,
    }

    #[test]
    fn round_trip_yaml() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.yaml");
        let config = Config {
            name: "test".to_string(),
            count: 42,
        };
        write_yaml(&path, &config).unwrap();
        let loaded: Config = read_yaml(&path).unwrap();
        assert_eq!(loaded, config);
    }

    #[test]
    fn read_missing_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("missing.yaml");
        let result: Result<Config, _> = read_yaml(&path);
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[test]
    fn write_creates_parent_dirs() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nested").join("dir").join("config.yaml");
        let config = Config {
            name: "nested".to_string(),
            count: 1,
        };
        write_yaml(&path, &config).unwrap();
        assert!(path.exists());
    }
}
