//! Chapter CRUD operations via CRDT.

use serde::{Deserialize, Serialize};

/// Full chapter data including body text.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChapterData {
    /// URL-safe slug identifier.
    pub slug: String,
    /// Human-readable title.
    pub title: String,
    /// Writing status: "draft", "revised", "final".
    pub status: String,
    /// Point-of-view character or narrator.
    pub pov: Option<String>,
    /// Brief synopsis of the chapter.
    pub synopsis: Option<String>,
    /// Target word count.
    pub target_words: Option<u32>,
    /// The chapter body text.
    pub body: String,
}

/// Partial update for chapter metadata.
#[derive(Debug, Clone, Default)]
pub struct ChapterMetaUpdate {
    /// New title.
    pub title: Option<String>,
    /// New status.
    pub status: Option<String>,
    /// New POV (Some(None) clears it).
    pub pov: Option<Option<String>>,
    /// New synopsis (Some(None) clears it).
    pub synopsis: Option<Option<String>>,
    /// New target word count (Some(None) clears it).
    pub target_words: Option<Option<u32>>,
}

/// Summary info for listing chapters.
#[derive(Debug, Clone, PartialEq)]
pub struct ChapterSummary {
    /// URL-safe slug identifier.
    pub slug: String,
    /// Human-readable title.
    pub title: String,
    /// Writing status.
    pub status: String,
}

/// Writing session data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SessionData {
    /// Session identifier (ISO 8601 timestamp).
    pub id: String,
    /// Session start time.
    pub start: String,
    /// Session end time.
    pub end: Option<String>,
    /// Duration in minutes.
    pub duration_minutes: Option<f64>,
    /// Words written during the session.
    pub words_written: u32,
    /// Chapter being worked on.
    pub chapter_slug: String,
    /// Sprint goal (target word count).
    pub sprint_goal: Option<u32>,
}
