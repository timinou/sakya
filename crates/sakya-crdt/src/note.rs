//! Note CRUD operations via CRDT.

use serde::{Deserialize, Serialize};

/// Full note data including body text.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NoteData {
    /// URL-safe slug identifier.
    pub slug: String,
    /// Human-readable title.
    pub title: String,
    /// Color label for the note.
    pub color: Option<String>,
    /// Category or label.
    pub label: Option<String>,
    /// The note body text.
    pub body: String,
}

/// Summary info for listing notes.
#[derive(Debug, Clone, PartialEq)]
pub struct NoteSummary {
    /// URL-safe slug identifier.
    pub slug: String,
    /// Human-readable title.
    pub title: String,
    /// Color label.
    pub color: Option<String>,
}
