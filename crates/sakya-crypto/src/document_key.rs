//! Document encryption key management with secure memory handling.
//!
//! Provides [`DocumentKey`] -- a 256-bit symmetric key for encrypting
//! per-project CRDT updates. Implements [`Zeroize`] and [`ZeroizeOnDrop`]
//! for secure cleanup of key material from memory.

use hkdf::Hkdf;
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::Sha256;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A 256-bit symmetric key for document encryption.
///
/// Wraps raw key bytes with secure memory handling. Key material is
/// automatically zeroed on drop to prevent leakage via memory dumps.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct DocumentKey {
    bytes: [u8; 32],
}

impl DocumentKey {
    /// Generate a new random document key from the OS CSPRNG.
    pub fn generate() -> Self {
        let mut bytes = [0u8; 32];
        OsRng.fill_bytes(&mut bytes);
        Self { bytes }
    }

    /// Create a document key from raw bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }

    /// Get a reference to the raw key bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    /// Derive a document key from a master secret and context string
    /// using HKDF-SHA256.
    ///
    /// The same `(master, context)` inputs always produce the same key.
    /// Different contexts produce different keys, providing domain separation.
    pub fn derive(master: &[u8], context: &str) -> Self {
        let hkdf = Hkdf::<Sha256>::new(None, master);
        let mut bytes = [0u8; 32];
        hkdf.expand(context.as_bytes(), &mut bytes)
            .expect("32 bytes is a valid HKDF-SHA256 output length");
        Self { bytes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encryptor::{Encryptor, XChaCha20Encryptor};

    #[test]
    fn generate_produces_non_zero_key() {
        let key = DocumentKey::generate();
        let bytes = key.as_bytes();

        // A randomly generated key should not be all zeros
        // (probability 2^-256, effectively impossible)
        assert!(
            bytes.iter().any(|&b| b != 0),
            "generated key should not be all zeros"
        );
    }

    #[test]
    fn from_bytes_round_trip() {
        let original: [u8; 32] = [42u8; 32];
        let key = DocumentKey::from_bytes(original);

        assert_eq!(
            key.as_bytes(),
            &original,
            "from_bytes and as_bytes should round-trip"
        );
    }

    #[test]
    fn derive_is_deterministic() {
        let master = b"master secret key material";
        let context = "project-abc-123";

        let key1 = DocumentKey::derive(master, context);
        let key2 = DocumentKey::derive(master, context);

        assert_eq!(
            key1.as_bytes(),
            key2.as_bytes(),
            "derivation with same inputs should be deterministic"
        );
    }

    #[test]
    fn derive_different_contexts_different_keys() {
        let master = b"master secret key material";

        let key_a = DocumentKey::derive(master, "project-a");
        let key_b = DocumentKey::derive(master, "project-b");

        assert_ne!(
            key_a.as_bytes(),
            key_b.as_bytes(),
            "different contexts should produce different keys"
        );
    }

    #[test]
    fn derive_different_masters_different_keys() {
        let context = "same-context";

        let key_a = DocumentKey::derive(b"master A", context);
        let key_b = DocumentKey::derive(b"master B", context);

        assert_ne!(
            key_a.as_bytes(),
            key_b.as_bytes(),
            "different master secrets should produce different keys"
        );
    }

    #[test]
    fn key_works_with_encryptor() {
        let doc_key = DocumentKey::generate();
        let enc = XChaCha20Encryptor::new(*doc_key.as_bytes());

        let plaintext = b"chapter content";
        let aad = b"chapter-slug";

        let envelope = enc.encrypt(plaintext, aad).expect("encrypt should succeed");
        let decrypted = enc.decrypt(&envelope).expect("decrypt should succeed");

        assert_eq!(decrypted, plaintext);
    }
}
