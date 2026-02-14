use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectManifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
