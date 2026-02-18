//! Error types for the sync client.

use thiserror::Error;

/// Errors that can occur in the sync client.
#[derive(Debug, Error)]
pub enum SyncClientError {
    #[error("WebSocket connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Decryption error: {0}")]
    DecryptionError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Engine has stopped")]
    EngineStopped,

    #[error("Offline queue error: {0}")]
    OfflineQueueError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<sakya_crypto::CryptoError> for SyncClientError {
    fn from(e: sakya_crypto::CryptoError) -> Self {
        SyncClientError::EncryptionError(e.to_string())
    }
}
