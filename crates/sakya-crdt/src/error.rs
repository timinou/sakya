//! Error types for the CRDT document model.

use thiserror::Error;

/// Errors that can occur during CRDT operations.
#[derive(Debug, Error)]
pub enum CrdtError {
    /// An error from the Loro CRDT engine.
    #[error("Loro error: {0}")]
    Loro(String),

    /// The requested chapter was not found.
    #[error("Chapter not found: {0}")]
    ChapterNotFound(String),

    /// The requested note was not found.
    #[error("Note not found: {0}")]
    NoteNotFound(String),

    /// The requested entity was not found.
    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    /// A serialization or deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// An IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<loro::LoroError> for CrdtError {
    fn from(e: loro::LoroError) -> Self {
        CrdtError::Loro(e.to_string())
    }
}

impl From<loro::LoroEncodeError> for CrdtError {
    fn from(e: loro::LoroEncodeError) -> Self {
        CrdtError::Loro(e.to_string())
    }
}
