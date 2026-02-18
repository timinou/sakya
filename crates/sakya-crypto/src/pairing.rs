//! Device pairing payload encoding for QR code and string-based exchange.
//!
//! Provides [`PairingPayload`] which encapsulates the data exchanged during
//! device pairing: the device ID, X25519 public key, and relay server URL.
//! The payload can be serialized to a QR code (SVG) or a human-readable
//! one-time pairing string with the `sk-pair_v1.` prefix.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{CryptoError, Result};

/// Version prefix for pairing strings.
const PAIRING_STRING_PREFIX: &str = "sk-pair_v1.";

/// Payload exchanged during device pairing.
///
/// Contains everything the remote device needs to establish a secure
/// connection: the originating device's identity, its X25519 public key
/// for Diffie-Hellman key agreement, and the relay server URL to connect to.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PairingPayload {
    /// Unique identifier of the device initiating pairing.
    pub device_id: Uuid,
    /// X25519 public key bytes (32 bytes) for key exchange.
    pub public_key: [u8; 32],
    /// URL of the relay server to use for sync.
    pub server_url: String,
}

impl PairingPayload {
    /// Create a new pairing payload.
    pub fn new(device_id: Uuid, public_key: [u8; 32], server_url: String) -> Self {
        Self {
            device_id,
            public_key,
            server_url,
        }
    }

    /// Encode this payload as a QR code in SVG format.
    ///
    /// The payload is serialized to JSON, then base64url-encoded (no padding),
    /// and rendered as an SVG QR code with moderate (M-level) error correction.
    pub fn to_qr_svg(&self) -> Result<String> {
        let data = self.encode_base64()?;

        let code = qrcode::QrCode::with_error_correction_level(data.as_bytes(), qrcode::EcLevel::M)
            .map_err(|e| CryptoError::InvalidKey(format!("QR code generation failed: {e}")))?;

        let svg = code
            .render::<qrcode::render::svg::Color>()
            .min_dimensions(200, 200)
            .dark_color(qrcode::render::svg::Color("#000000"))
            .light_color(qrcode::render::svg::Color("#ffffff"))
            .build();

        Ok(svg)
    }

    /// Encode as a human-readable one-time pairing string.
    ///
    /// Format: `sk-pair_v1.<base64url_payload>`
    ///
    /// The base64url encoding uses URL-safe characters with no padding,
    /// making it safe to embed in QR codes, URLs, or copy-paste workflows.
    pub fn to_pairing_string(&self) -> Result<String> {
        let encoded = self.encode_base64()?;
        Ok(format!("{PAIRING_STRING_PREFIX}{encoded}"))
    }

    /// Decode a pairing payload from a pairing string.
    ///
    /// Expects the `sk-pair_v1.` prefix followed by a base64url-encoded
    /// JSON payload.
    pub fn from_pairing_string(s: &str) -> Result<Self> {
        let data = s.strip_prefix(PAIRING_STRING_PREFIX).ok_or_else(|| {
            CryptoError::InvalidKey(format!(
                "pairing string must start with '{PAIRING_STRING_PREFIX}'"
            ))
        })?;

        if data.is_empty() {
            return Err(CryptoError::InvalidKey(
                "pairing string payload is empty".to_string(),
            ));
        }

        let json_bytes = URL_SAFE_NO_PAD.decode(data).map_err(|e| {
            CryptoError::InvalidKey(format!("invalid base64 in pairing string: {e}"))
        })?;

        serde_json::from_slice(&json_bytes)
            .map_err(|e| CryptoError::InvalidKey(format!("invalid JSON in pairing payload: {e}")))
    }

    /// Internal helper: serialize to JSON then base64url-encode.
    fn encode_base64(&self) -> Result<String> {
        let json = serde_json::to_vec(self).map_err(|e| {
            CryptoError::InvalidKey(format!("failed to serialize pairing payload: {e}"))
        })?;
        Ok(URL_SAFE_NO_PAD.encode(&json))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a sample payload for testing.
    fn sample_payload() -> PairingPayload {
        PairingPayload::new(
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            [42u8; 32],
            "https://relay.sakya.io:8443/v1".to_string(),
        )
    }

    #[test]
    fn encode_decode_round_trip() {
        let original = sample_payload();
        let pairing_str = original
            .to_pairing_string()
            .expect("encoding should succeed");
        let decoded =
            PairingPayload::from_pairing_string(&pairing_str).expect("decoding should succeed");

        assert_eq!(original, decoded, "round-trip should preserve all fields");
    }

    #[test]
    fn qr_svg_is_valid_svg() {
        let payload = sample_payload();
        let svg = payload
            .to_qr_svg()
            .expect("QR SVG generation should succeed");

        assert!(
            svg.contains("<svg"),
            "SVG output must contain opening <svg tag"
        );
        assert!(
            svg.contains("</svg>"),
            "SVG output must contain closing </svg> tag"
        );
        // Sanity: SVG should have reasonable length (not empty/trivial)
        assert!(
            svg.len() > 100,
            "SVG should have non-trivial content, got {} bytes",
            svg.len()
        );
    }

    #[test]
    fn pairing_string_format() {
        let payload = sample_payload();
        let pairing_str = payload
            .to_pairing_string()
            .expect("encoding should succeed");

        assert!(
            pairing_str.starts_with("sk-pair_v1."),
            "pairing string must start with 'sk-pair_v1.', got: {}",
            &pairing_str[..20.min(pairing_str.len())]
        );

        // The part after the prefix should be valid base64url
        let data_part = pairing_str.strip_prefix("sk-pair_v1.").unwrap();
        assert!(!data_part.is_empty(), "base64 payload must not be empty");
        assert!(
            URL_SAFE_NO_PAD.decode(data_part).is_ok(),
            "payload after prefix must be valid base64url"
        );
    }

    #[test]
    fn invalid_pairing_string_rejected() {
        // Wrong prefix
        let result = PairingPayload::from_pairing_string("wrong-prefix.abc123");
        assert!(result.is_err(), "wrong prefix should be rejected");

        // Empty string
        let result = PairingPayload::from_pairing_string("");
        assert!(result.is_err(), "empty string should be rejected");

        // Correct prefix but no payload
        let result = PairingPayload::from_pairing_string("sk-pair_v1.");
        assert!(result.is_err(), "empty payload should be rejected");

        // Correct prefix but invalid base64
        let result = PairingPayload::from_pairing_string("sk-pair_v1.!!!not-base64!!!");
        assert!(result.is_err(), "invalid base64 should be rejected");

        // Correct prefix, valid base64, but invalid JSON
        let garbage_b64 = URL_SAFE_NO_PAD.encode(b"not json at all");
        let result = PairingPayload::from_pairing_string(&format!("sk-pair_v1.{garbage_b64}"));
        assert!(
            result.is_err(),
            "valid base64 with invalid JSON should be rejected"
        );

        // Truncated base64 (valid prefix, truncated payload)
        let payload = sample_payload();
        let full_str = payload.to_pairing_string().unwrap();
        let truncated = &full_str[..full_str.len() - 10];
        let result = PairingPayload::from_pairing_string(truncated);
        assert!(result.is_err(), "truncated payload should be rejected");
    }

    #[test]
    fn special_characters_in_server_url() {
        let urls = [
            "https://relay.sakya.io:8443/v1",
            "http://localhost:3000/sync?token=abc&device=123",
            "https://example.com/path/to/relay#fragment",
            "wss://relay.example.org:9090/ws?room=test%20room",
            "https://user:pass@host.example.com:443/path", // pragma: allowlist secret
        ];

        for url in &urls {
            let payload = PairingPayload::new(Uuid::new_v4(), [1u8; 32], url.to_string());

            let pairing_str = payload
                .to_pairing_string()
                .unwrap_or_else(|e| panic!("encoding should succeed for URL '{url}': {e}"));
            let decoded = PairingPayload::from_pairing_string(&pairing_str)
                .unwrap_or_else(|e| panic!("decoding should succeed for URL '{url}': {e}"));

            assert_eq!(
                payload.server_url, decoded.server_url,
                "server URL should survive round-trip: {url}"
            );
        }
    }
}
