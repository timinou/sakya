//! JWT token generation and validation.
//!
//! Uses HS256 (HMAC-SHA256) via the `jsonwebtoken` crate.

use crate::error::AuthError;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT claims embedded in every access token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject: the account identifier.
    pub sub: Uuid,
    /// The device that created this token.
    pub device_id: Uuid,
    /// Expiration time (Unix timestamp).
    pub exp: i64,
    /// Issued-at time (Unix timestamp).
    pub iat: i64,
}

/// Service for creating and validating JWT access tokens.
pub struct JwtService {
    secret: String,
    access_token_expiry_secs: i64,
}

impl JwtService {
    /// Create a new JWT service with the given HMAC secret.
    ///
    /// The default access token lifetime is 24 hours.
    pub fn new(secret: &str) -> Self {
        Self {
            secret: secret.to_string(),
            access_token_expiry_secs: 86400, // 24 hours
        }
    }

    /// Create a JWT service with a custom expiry duration.
    pub fn with_expiry(secret: &str, expiry_secs: i64) -> Self {
        Self {
            secret: secret.to_string(),
            access_token_expiry_secs: expiry_secs,
        }
    }

    /// Generate a signed JWT for the given account and device.
    pub fn generate_token(&self, account_id: Uuid, device_id: Uuid) -> Result<String, AuthError> {
        let now = chrono::Utc::now().timestamp();
        let claims = Claims {
            sub: account_id,
            device_id,
            iat: now,
            exp: now + self.access_token_expiry_secs,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AuthError::InvalidToken(e.to_string()))
    }

    /// Validate a JWT string, returning the embedded claims.
    pub fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        let mut validation = Validation::default();
        validation.leeway = 0;
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            _ => AuthError::InvalidToken(e.to_string()),
        })?;

        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_and_validate_round_trip() {
        let svc = JwtService::new("test-secret-key");
        let account_id = Uuid::new_v4();
        let device_id = Uuid::new_v4();

        let token = svc.generate_token(account_id, device_id).unwrap();
        let claims = svc.validate_token(&token).unwrap();

        assert_eq!(claims.sub, account_id);
        assert_eq!(claims.device_id, device_id);
    }

    #[test]
    fn expired_token_rejected() {
        // Create a service with 0 second expiry
        let svc = JwtService::with_expiry("test-secret-key", -10);
        let token = svc.generate_token(Uuid::new_v4(), Uuid::new_v4()).unwrap();

        let result = svc.validate_token(&token);
        assert!(result.is_err(), "expired token should be rejected");
        match result.unwrap_err() {
            AuthError::TokenExpired => {}
            other => panic!("expected TokenExpired, got: {other}"),
        }
    }

    #[test]
    fn wrong_secret_rejected() {
        let svc1 = JwtService::new("secret-A");
        let svc2 = JwtService::new("secret-B");

        let token = svc1.generate_token(Uuid::new_v4(), Uuid::new_v4()).unwrap();

        let result = svc2.validate_token(&token);
        assert!(result.is_err(), "wrong secret should reject token");
        match result.unwrap_err() {
            AuthError::InvalidToken(_) => {}
            other => panic!("expected InvalidToken, got: {other}"),
        }
    }

    #[test]
    fn claims_are_correct() {
        let svc = JwtService::new("test-secret");
        let account_id = Uuid::new_v4();
        let device_id = Uuid::new_v4();

        let token = svc.generate_token(account_id, device_id).unwrap();
        let claims = svc.validate_token(&token).unwrap();

        assert_eq!(claims.sub, account_id, "sub should match account_id");
        assert_eq!(claims.device_id, device_id, "device_id should match");
    }

    #[test]
    fn token_contains_iat_and_exp() {
        let svc = JwtService::new("test-secret");
        let token = svc.generate_token(Uuid::new_v4(), Uuid::new_v4()).unwrap();
        let claims = svc.validate_token(&token).unwrap();

        assert!(claims.iat > 0, "iat should be set");
        assert!(claims.exp > claims.iat, "exp should be after iat");
        assert_eq!(
            claims.exp - claims.iat,
            86400,
            "default expiry should be 24 hours"
        );
    }

    #[test]
    fn invalid_token_string_rejected() {
        let svc = JwtService::new("test-secret");
        let result = svc.validate_token("not.a.valid.jwt");
        assert!(result.is_err(), "garbage string should be rejected");
        match result.unwrap_err() {
            AuthError::InvalidToken(_) => {}
            other => panic!("expected InvalidToken, got: {other}"),
        }
    }

    #[test]
    fn empty_token_rejected() {
        let svc = JwtService::new("test-secret");
        let result = svc.validate_token("");
        assert!(result.is_err(), "empty string should be rejected");
    }
}
