use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ChapterStatus {
    Draft,
    Revised,
    Final,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManuscriptConfig {
    pub chapters: Vec<String>, // ordered slugs
}

/// Frontmatter stored in chapter Markdown files.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterFrontmatter {
    pub title: String,
    pub slug: String,
    pub status: ChapterStatus,
    #[serde(default)]
    pub pov: Option<String>,
    #[serde(default)]
    pub synopsis: Option<String>,
    #[serde(default)]
    pub target_words: Option<u32>,
    #[serde(default)]
    pub order: u32,
}

/// Chapter summary for listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chapter {
    pub slug: String,
    pub title: String,
    pub status: ChapterStatus,
    #[serde(default)]
    pub pov: Option<String>,
    #[serde(default)]
    pub synopsis: Option<String>,
    #[serde(default)]
    pub target_words: Option<u32>,
    pub order: u32,
}

/// Full chapter with body content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterContent {
    pub slug: String,
    pub frontmatter: Chapter,
    pub body: String,
}
