use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectManifest {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "default_timestamp")]
    pub updated_at: DateTime<Utc>,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_timestamp() -> DateTime<Utc> {
    Utc::now()
}

impl ProjectManifest {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            name,
            version: "0.1.0".to_string(),
            author: None,
            description: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentProject {
    pub name: String,
    pub path: String,
    pub last_opened: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialization_with_minimal_yaml() {
        let yaml = "name: Legacy Novel\n";
        let manifest: ProjectManifest = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(manifest.name, "Legacy Novel");
        assert_eq!(manifest.version, "0.1.0");
        assert!(manifest.author.is_none());
        assert!(manifest.description.is_none());
        // Timestamps should be defaulted (not panic)
        assert!(manifest.created_at <= Utc::now());
        assert!(manifest.updated_at <= Utc::now());
    }

    #[test]
    fn deserialization_preserves_provided_fields() {
        let yaml = r#"
name: Full Novel
version: "1.2.3"
author: Jane Doe
description: A great story
createdAt: "2025-06-15T10:30:00Z"
updatedAt: "2025-12-01T14:00:00Z"
"#;
        let manifest: ProjectManifest = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(manifest.name, "Full Novel");
        assert_eq!(manifest.version, "1.2.3");
        assert_eq!(manifest.author, Some("Jane Doe".to_string()));
        assert_eq!(manifest.description, Some("A great story".to_string()));
        assert_eq!(
            manifest.created_at,
            "2025-06-15T10:30:00Z".parse::<DateTime<Utc>>().unwrap()
        );
        assert_eq!(
            manifest.updated_at,
            "2025-12-01T14:00:00Z".parse::<DateTime<Utc>>().unwrap()
        );
    }
}
