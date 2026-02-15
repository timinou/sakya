//! Error types for the auth crate.

/// Errors that can occur during authentication operations.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// A database operation failed.
    #[error("Database error: {0}")]
    Database(String),

    /// A token was invalid (malformed, wrong signature, etc.).
    #[error("Invalid token: {0}")]
    InvalidToken(String),

    /// A token has expired.
    #[error("Token expired")]
    TokenExpired,

    /// The requested account does not exist.
    #[error("Account not found")]
    AccountNotFound,

    /// The requested device does not exist.
    #[error("Device not found")]
    DeviceNotFound,

    /// The caller has been rate-limited.
    #[error("Rate limited: {0}")]
    RateLimited(String),

    /// An email-sending operation failed.
    #[error("Email error: {0}")]
    EmailError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_database() {
        let err = AuthError::Database("connection failed".to_string());
        assert_eq!(err.to_string(), "Database error: connection failed");
    }

    #[test]
    fn error_display_invalid_token() {
        let err = AuthError::InvalidToken("bad signature".to_string());
        assert_eq!(err.to_string(), "Invalid token: bad signature");
    }

    #[test]
    fn error_display_token_expired() {
        let err = AuthError::TokenExpired;
        assert_eq!(err.to_string(), "Token expired");
    }

    #[test]
    fn error_display_account_not_found() {
        let err = AuthError::AccountNotFound;
        assert_eq!(err.to_string(), "Account not found");
    }

    #[test]
    fn error_display_device_not_found() {
        let err = AuthError::DeviceNotFound;
        assert_eq!(err.to_string(), "Device not found");
    }

    #[test]
    fn error_display_rate_limited() {
        let err = AuthError::RateLimited("too many requests".to_string());
        assert_eq!(err.to_string(), "Rate limited: too many requests");
    }

    #[test]
    fn error_display_email_error() {
        let err = AuthError::EmailError("SMTP down".to_string());
        assert_eq!(err.to_string(), "Email error: SMTP down");
    }

    #[test]
    fn error_is_std_error() {
        let err: Box<dyn std::error::Error> = Box::new(AuthError::Database("test".to_string()));
        assert!(err.to_string().contains("Database error"));
    }
}
