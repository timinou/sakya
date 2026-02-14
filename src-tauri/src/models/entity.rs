use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    ShortText,
    LongText,
    Number,
    Select,
    Date,
    Boolean,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityField {
    pub name: String,
    pub label: String,
    pub field_type: FieldType,
    pub required: bool,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub options: Option<Vec<String>>,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpiderAxis {
    pub name: String,
    pub min: f64,
    pub max: f64,
    pub default: f64,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntitySchema {
    pub name: String,
    pub entity_type: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    pub fields: Vec<EntityField>,
    pub spider_axes: Vec<SpiderAxis>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaSummary {
    pub name: String,
    pub entity_type: String,
    pub field_count: usize,
    pub axis_count: usize,
}

// ── Entity Instance Models ──────────────────────────────────────

/// Frontmatter stored in entity Markdown files.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityFrontmatter {
    pub title: String,
    pub slug: String,
    pub schema_type: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub spider_values: HashMap<String, f64>,
    #[serde(default)]
    pub fields: HashMap<String, serde_json::Value>,
}

/// Lightweight summary of an entity instance (for listing).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntitySummary {
    pub title: String,
    pub slug: String,
    pub schema_type: String,
    pub tags: Vec<String>,
}

/// Full entity instance with body content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityInstance {
    pub title: String,
    pub slug: String,
    pub schema_slug: String,
    pub tags: Vec<String>,
    pub spider_values: HashMap<String, f64>,
    pub fields: HashMap<String, serde_json::Value>,
    pub body: String,
}
