//! Error types for cryptographic operations.

use serde::Serialize;

/// Errors that can occur during cryptographic operations.
#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum CryptoError {
    /// Encryption operation failed.
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    /// Decryption operation failed.
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    /// Signature verification failed.
    #[error("Signature verification failed")]
    VerificationFailed,

    /// Invalid key material.
    #[error("Invalid key: {0}")]
    InvalidKey(String),

    /// Hash chain integrity error.
    #[error("Hash chain integrity error: {0}")]
    HashChainError(String),
}

/// Convenience type alias for results using [`CryptoError`].
pub type Result<T> = std::result::Result<T, CryptoError>;
