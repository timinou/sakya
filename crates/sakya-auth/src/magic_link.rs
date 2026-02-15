//! Magic link generation and verification.
//!
//! Tokens are generated as 32 random bytes, base64url-encoded.
//! Only a BLAKE2b-256 hash of the token is stored in the database.
//! Verification checks expiry (15 min) and marks the link as used.

use crate::db::AuthDb;
use crate::error::AuthError;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use blake2::digest::{consts::U32, Digest};
use chrono::{Duration, Utc};
use rand::RngCore;
use uuid::Uuid;

/// BLAKE2b with 32-byte output.
type Blake2b256 = blake2::Blake2b<U32>;

/// Service for creating and verifying magic link tokens.
pub struct MagicLinkService {
    db: AuthDb,
}

impl MagicLinkService {
    /// Create a new service backed by the given database.
    pub fn new(db: AuthDb) -> Self {
        Self { db }
    }

    /// Generate a random 32-byte token, base64url-encoded (no padding).
    pub fn generate_token() -> String {
        let mut bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        URL_SAFE_NO_PAD.encode(bytes)
    }

    /// Hash a token with BLAKE2b-256, returning the hex-encoded digest.
    pub fn hash_token(token: &str) -> String {
        let mut hasher = Blake2b256::new();
        hasher.update(token.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Create a magic link for the given email address.
    ///
    /// Returns the raw token (to be sent to the user). Only the hash is stored.
    pub fn create_magic_link(&self, email: &str) -> Result<String, AuthError> {
        self.check_rate_limit(email)?;

        let token = Self::generate_token();
        let token_hash = Self::hash_token(&token);
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = now + Duration::minutes(15);

        self.db
            .conn()
            .execute(
                "INSERT INTO magic_links (id, email, token_hash, expires_at, used, created_at) \
                 VALUES (?1, ?2, ?3, ?4, 0, ?5)",
                rusqlite::params![
                    id,
                    email,
                    token_hash,
                    expires_at.to_rfc3339(),
                    now.to_rfc3339(),
                ],
            )
            .map_err(|e| AuthError::Database(e.to_string()))?;

        Ok(token)
    }

    /// Verify a magic link token.
    ///
    /// On success, creates (or retrieves) the account for the email and returns
    /// `(account_id, email)`. The link is marked as used.
    pub fn verify_magic_link(&self, token: &str) -> Result<(Uuid, String), AuthError> {
        let token_hash = Self::hash_token(token);

        let (link_id, email, expires_at_str, used): (String, String, String, bool) = self
            .db
            .conn()
            .query_row(
                "SELECT id, email, expires_at, used FROM magic_links WHERE token_hash = ?1",
                rusqlite::params![token_hash],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get::<_, i32>(3)? != 0,
                    ))
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    AuthError::InvalidToken("token not found".to_string())
                }
                other => AuthError::Database(other.to_string()),
            })?;

        if used {
            return Err(AuthError::InvalidToken("token already used".to_string()));
        }

        let expires_at = chrono::DateTime::parse_from_rfc3339(&expires_at_str)
            .map_err(|e| AuthError::Database(format!("bad expiry date: {e}")))?;

        if Utc::now() > expires_at {
            return Err(AuthError::TokenExpired);
        }

        // Mark as used
        self.db
            .conn()
            .execute(
                "UPDATE magic_links SET used = 1 WHERE id = ?1",
                rusqlite::params![link_id],
            )
            .map_err(|e| AuthError::Database(e.to_string()))?;

        // Create or get account
        let account_id = self.get_or_create_account(&email)?;

        Ok((account_id, email))
    }

    /// Check rate limit: max 3 magic links per email per hour.
    pub fn check_rate_limit(&self, email: &str) -> Result<(), AuthError> {
        let one_hour_ago = (Utc::now() - Duration::hours(1)).to_rfc3339();

        let count: i64 = self
            .db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM magic_links WHERE email = ?1 AND created_at > ?2",
                rusqlite::params![email, one_hour_ago],
                |row| row.get(0),
            )
            .map_err(|e| AuthError::Database(e.to_string()))?;

        if count >= 3 {
            return Err(AuthError::RateLimited(format!(
                "max 3 magic links per hour for {email}"
            )));
        }

        Ok(())
    }

    /// Get an existing account by email, or create one.
    fn get_or_create_account(&self, email: &str) -> Result<Uuid, AuthError> {
        // Try to find existing
        let existing: Result<String, _> = self.db.conn().query_row(
            "SELECT id FROM accounts WHERE email = ?1",
            rusqlite::params![email],
            |row| row.get(0),
        );

        match existing {
            Ok(id_str) => {
                let id = Uuid::parse_str(&id_str)
                    .map_err(|e| AuthError::Database(format!("bad account id: {e}")))?;
                Ok(id)
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                let id = Uuid::new_v4();
                let now = Utc::now().to_rfc3339();
                self.db
                    .conn()
                    .execute(
                        "INSERT INTO accounts (id, email, created_at) VALUES (?1, ?2, ?3)",
                        rusqlite::params![id.to_string(), email, now],
                    )
                    .map_err(|e| AuthError::Database(e.to_string()))?;
                Ok(id)
            }
            Err(e) => Err(AuthError::Database(e.to_string())),
        }
    }
}

/// We need the `hex` encoding. Since it is not a crate dep, implement inline.
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().map(|b| format!("{b:02x}")).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> MagicLinkService {
        let db = AuthDb::new_in_memory().unwrap();
        MagicLinkService::new(db)
    }

    #[test]
    fn create_and_verify_magic_link() {
        let svc = setup();
        let token = svc.create_magic_link("alice@example.com").unwrap();
        assert!(!token.is_empty(), "token should be non-empty");

        let (account_id, email) = svc.verify_magic_link(&token).unwrap();
        assert_eq!(email, "alice@example.com");
        assert!(!account_id.is_nil(), "account_id should be a valid UUID");
    }

    #[test]
    fn expired_link_rejected() {
        let db = AuthDb::new_in_memory().unwrap();

        // Manually insert an expired link
        let token = MagicLinkService::generate_token();
        let token_hash = MagicLinkService::hash_token(&token);
        let expired = (Utc::now() - Duration::minutes(1)).to_rfc3339();
        let now = Utc::now().to_rfc3339();

        db.conn()
            .execute(
                "INSERT INTO magic_links (id, email, token_hash, expires_at, used, created_at) \
                 VALUES (?1, ?2, ?3, ?4, 0, ?5)",
                rusqlite::params!["ml-exp", "bob@example.com", token_hash, expired, now],
            )
            .unwrap();

        let svc = MagicLinkService::new(db);
        let result = svc.verify_magic_link(&token);
        assert!(result.is_err(), "expired link should be rejected");
        match result.unwrap_err() {
            AuthError::TokenExpired => {}
            other => panic!("expected TokenExpired, got: {other}"),
        }
    }

    #[test]
    fn used_link_rejected() {
        let svc = setup();
        let token = svc.create_magic_link("carol@example.com").unwrap();

        // First verification succeeds
        let result = svc.verify_magic_link(&token);
        assert!(result.is_ok(), "first verify should succeed");

        // Second verification fails
        let result = svc.verify_magic_link(&token);
        assert!(result.is_err(), "second verify should fail");
        match result.unwrap_err() {
            AuthError::InvalidToken(msg) => {
                assert!(
                    msg.contains("already used"),
                    "error should mention 'already used'"
                );
            }
            other => panic!("expected InvalidToken, got: {other}"),
        }
    }

    #[test]
    fn rate_limit_after_three() {
        let svc = setup();
        let email = "ratelimit@example.com";

        // First three should succeed
        svc.create_magic_link(email).unwrap();
        svc.create_magic_link(email).unwrap();
        svc.create_magic_link(email).unwrap();

        // Fourth should be rate limited
        let result = svc.create_magic_link(email);
        assert!(result.is_err(), "4th link should be rate-limited");
        match result.unwrap_err() {
            AuthError::RateLimited(msg) => {
                assert!(msg.contains("max 3"), "error should mention limit");
            }
            other => panic!("expected RateLimited, got: {other}"),
        }
    }

    #[test]
    fn creates_account_on_first_verify() {
        let svc = setup();
        let token = svc.create_magic_link("newuser@example.com").unwrap();

        // Before verification, no account exists
        let count: i64 = svc
            .db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM accounts WHERE email = ?1",
                rusqlite::params!["newuser@example.com"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0, "no account should exist before verification");

        let (account_id, _) = svc.verify_magic_link(&token).unwrap();

        // After verification, account exists
        let count: i64 = svc
            .db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM accounts WHERE email = ?1",
                rusqlite::params!["newuser@example.com"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "account should exist after verification");
        assert!(!account_id.is_nil());
    }

    #[test]
    fn existing_account_on_second_verify() {
        let svc = setup();

        // First magic link + verify creates account
        let token1 = svc.create_magic_link("returning@example.com").unwrap();
        let (account_id_1, _) = svc.verify_magic_link(&token1).unwrap();

        // Second magic link + verify reuses account
        let token2 = svc.create_magic_link("returning@example.com").unwrap();
        let (account_id_2, _) = svc.verify_magic_link(&token2).unwrap();

        assert_eq!(
            account_id_1, account_id_2,
            "same email should return same account"
        );
    }

    #[test]
    fn invalid_token_rejected() {
        let svc = setup();
        let result = svc.verify_magic_link("totally-bogus-token");
        assert!(result.is_err(), "invalid token should be rejected");
        match result.unwrap_err() {
            AuthError::InvalidToken(msg) => {
                assert!(
                    msg.contains("not found"),
                    "error should mention 'not found'"
                );
            }
            other => panic!("expected InvalidToken, got: {other}"),
        }
    }

    #[test]
    fn token_hash_is_deterministic() {
        let token = "some-fixed-token-value";
        let hash1 = MagicLinkService::hash_token(token);
        let hash2 = MagicLinkService::hash_token(token);
        assert_eq!(hash1, hash2, "same token should produce same hash");
        assert!(!hash1.is_empty(), "hash should be non-empty");
    }

    #[test]
    fn generated_tokens_are_unique() {
        let t1 = MagicLinkService::generate_token();
        let t2 = MagicLinkService::generate_token();
        assert_ne!(t1, t2, "two generated tokens should differ");
    }

    #[test]
    fn generated_token_length() {
        let token = MagicLinkService::generate_token();
        // 32 bytes base64url without padding = 43 chars
        assert_eq!(
            token.len(),
            43,
            "32 bytes base64url no-pad should be 43 chars"
        );
    }
}
