use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Sync error: {0}")]
    Sync(String),
}

// Serialize AppError for Tauri IPC (Tauri requires this)
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Serialization tests ─────────────────────────────────────────

    #[test]
    fn serialize_not_found() {
        let err = AppError::NotFound("chapter-1".into());
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, r#""Not found: chapter-1""#);
    }

    #[test]
    fn serialize_already_exists() {
        let err = AppError::AlreadyExists("project.yml".into());
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, r#""Already exists: project.yml""#);
    }

    #[test]
    fn serialize_invalid_operation() {
        let err = AppError::InvalidOperation("cannot delete root".into());
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, r#""Invalid operation: cannot delete root""#);
    }

    #[test]
    fn serialize_validation() {
        let err = AppError::Validation("title must not be empty".into());
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, r#""Validation error: title must not be empty""#);
    }

    #[test]
    fn serialize_sync() {
        let err = AppError::Sync("merge conflict on chapter-3".into());
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, r#""Sync error: merge conflict on chapter-3""#);
    }

    #[test]
    fn serialize_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err = AppError::Io(io_err);
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, r#""IO error: file missing""#);
    }

    #[test]
    fn serialize_json_error() {
        let json_err = serde_json::from_str::<String>("not json").unwrap_err();
        let err = AppError::Json(json_err);
        let json = serde_json::to_string(&err).unwrap();
        // The serialized form must be a JSON string whose content starts with "JSON error:"
        let deserialized: String = serde_json::from_str(&json).unwrap();
        assert!(
            deserialized.starts_with("JSON error:"),
            "expected prefix 'JSON error:', got: {deserialized}"
        );
    }

    #[test]
    fn serialize_yaml_error() {
        let yaml_err = serde_yaml::from_str::<String>("invalid: [yaml").unwrap_err();
        let err = AppError::Yaml(yaml_err);
        let json = serde_json::to_string(&err).unwrap();
        let deserialized: String = serde_json::from_str(&json).unwrap();
        assert!(
            deserialized.starts_with("YAML error:"),
            "expected prefix 'YAML error:', got: {deserialized}"
        );
    }

    // ── From conversion tests ───────────────────────────────────────

    #[test]
    fn from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let app_err: AppError = io_err.into();
        assert!(
            matches!(app_err, AppError::Io(_)),
            "expected AppError::Io, got: {app_err:?}"
        );
    }

    #[test]
    fn from_serde_json_error() {
        let json_err = serde_json::from_str::<String>("{bad}").unwrap_err();
        let app_err: AppError = json_err.into();
        assert!(
            matches!(app_err, AppError::Json(_)),
            "expected AppError::Json, got: {app_err:?}"
        );
    }

    #[test]
    fn from_serde_yaml_error() {
        let yaml_err = serde_yaml::from_str::<String>("invalid: [yaml").unwrap_err();
        let app_err: AppError = yaml_err.into();
        assert!(
            matches!(app_err, AppError::Yaml(_)),
            "expected AppError::Yaml, got: {app_err:?}"
        );
    }

    // ── Display formatting tests ────────────────────────────────────

    #[test]
    fn display_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        let err = AppError::Io(io_err);
        let msg = err.to_string();
        assert!(
            msg.starts_with("IO error:"),
            "expected 'IO error:' prefix, got: {msg}"
        );
        assert!(msg.contains("gone"));
    }

    #[test]
    fn display_yaml_error() {
        let yaml_err = serde_yaml::from_str::<String>("invalid: [yaml").unwrap_err();
        let err = AppError::Yaml(yaml_err);
        let msg = err.to_string();
        assert!(
            msg.starts_with("YAML error:"),
            "expected 'YAML error:' prefix, got: {msg}"
        );
    }

    #[test]
    fn display_json_error() {
        let json_err = serde_json::from_str::<String>("{{").unwrap_err();
        let err = AppError::Json(json_err);
        let msg = err.to_string();
        assert!(
            msg.starts_with("JSON error:"),
            "expected 'JSON error:' prefix, got: {msg}"
        );
    }

    #[test]
    fn display_not_found() {
        let err = AppError::NotFound("chapter-7".into());
        assert_eq!(err.to_string(), "Not found: chapter-7");
    }

    #[test]
    fn display_already_exists() {
        let err = AppError::AlreadyExists("scene-2".into());
        assert_eq!(err.to_string(), "Already exists: scene-2");
    }

    #[test]
    fn display_invalid_operation() {
        let err = AppError::InvalidOperation("move to self".into());
        assert_eq!(err.to_string(), "Invalid operation: move to self");
    }

    #[test]
    fn display_validation() {
        let err = AppError::Validation("slug is empty".into());
        assert_eq!(err.to_string(), "Validation error: slug is empty");
    }

    #[test]
    fn display_sync() {
        let err = AppError::Sync("clock skew detected".into());
        assert_eq!(err.to_string(), "Sync error: clock skew detected");
    }

    // ── Round-trip serialization test ───────────────────────────────

    #[test]
    fn round_trip_serialization() {
        let errors: Vec<AppError> = vec![
            AppError::NotFound("item-42".into()),
            AppError::AlreadyExists("dup".into()),
            AppError::InvalidOperation("nope".into()),
            AppError::Validation("bad input".into()),
            AppError::Sync("conflict".into()),
            AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, "disk full")),
        ];

        for err in &errors {
            let display = err.to_string();
            // Serialize to JSON string
            let json = serde_json::to_string(err).unwrap();
            // Deserialize the JSON — it should be a plain string
            let recovered: String = serde_json::from_str(&json).unwrap();
            assert_eq!(
                recovered, display,
                "round-trip mismatch for {err:?}: json={json}"
            );
        }
    }
}
