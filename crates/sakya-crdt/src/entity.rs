//! Entity CRUD operations via CRDT.

use serde::{Deserialize, Serialize};

/// An entity with dynamic fields, organized by schema type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntityData {
    /// URL-safe slug identifier.
    pub slug: String,
    /// Human-readable title.
    pub title: String,
    /// The schema type this entity belongs to (e.g., "characters", "locations").
    pub schema_type: String,
    /// Dynamic fields as a JSON value.
    pub fields: serde_json::Value,
}
