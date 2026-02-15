//! AEAD encryption using XChaCha20-Poly1305.
//!
//! Provides the [`Encryptor`] trait and [`XChaCha20Encryptor`] implementation
//! for authenticated encryption with associated data.

use crate::error::{CryptoError, Result};
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, Payload},
    XChaCha20Poly1305, XNonce,
};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

/// An encrypted envelope containing ciphertext, nonce, and associated data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedEnvelope {
    /// 24-byte nonce used for XChaCha20-Poly1305.
    pub nonce: [u8; 24],
    /// The encrypted ciphertext with authentication tag.
    pub ciphertext: Vec<u8>,
    /// Associated authenticated data (not encrypted, but authenticated).
    pub aad: Vec<u8>,
}

/// Trait for symmetric AEAD encryption/decryption.
pub trait Encryptor {
    /// Encrypt plaintext with associated data, returning an [`EncryptedEnvelope`].
    fn encrypt(&self, plaintext: &[u8], aad: &[u8]) -> Result<EncryptedEnvelope>;

    /// Decrypt an [`EncryptedEnvelope`], verifying the associated data.
    fn decrypt(&self, envelope: &EncryptedEnvelope) -> Result<Vec<u8>>;
}

/// XChaCha20-Poly1305 AEAD encryptor.
///
/// Uses a 256-bit key and 192-bit nonce (randomly generated per encryption).
/// The large nonce size makes random nonce generation safe without risk of
/// nonce reuse even at high volumes.
pub struct XChaCha20Encryptor {
    cipher: XChaCha20Poly1305,
}

impl XChaCha20Encryptor {
    /// Create a new encryptor with the given 256-bit key.
    pub fn new(key: [u8; 32]) -> Self {
        let cipher = XChaCha20Poly1305::new_from_slice(&key)
            .expect("32-byte key is always valid for XChaCha20Poly1305");
        Self { cipher }
    }
}

impl Encryptor for XChaCha20Encryptor {
    fn encrypt(&self, plaintext: &[u8], aad: &[u8]) -> Result<EncryptedEnvelope> {
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);

        let payload = Payload {
            msg: plaintext,
            aad,
        };

        let ciphertext = self
            .cipher
            .encrypt(&nonce, payload)
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

        let mut nonce_bytes = [0u8; 24];
        nonce_bytes.copy_from_slice(&nonce);

        Ok(EncryptedEnvelope {
            nonce: nonce_bytes,
            ciphertext,
            aad: aad.to_vec(),
        })
    }

    fn decrypt(&self, envelope: &EncryptedEnvelope) -> Result<Vec<u8>> {
        let nonce = XNonce::from_slice(&envelope.nonce);

        let payload = Payload {
            msg: &envelope.ciphertext,
            aad: &envelope.aad,
        };

        self.cipher
            .decrypt(nonce, payload)
            .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn random_key() -> [u8; 32] {
        use rand::RngCore;
        let mut key = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut key);
        key
    }

    #[test]
    fn encrypt_decrypt_round_trip() {
        let key = random_key();
        let enc = XChaCha20Encryptor::new(key);
        let plaintext = b"hello world";
        let aad = b"context";

        let envelope = enc.encrypt(plaintext, aad).expect("encrypt should succeed");
        let decrypted = enc.decrypt(&envelope).expect("decrypt should succeed");

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn wrong_key_fails_decrypt() {
        let key_a = random_key();
        let key_b = random_key();
        let enc_a = XChaCha20Encryptor::new(key_a);
        let enc_b = XChaCha20Encryptor::new(key_b);

        let envelope = enc_a
            .encrypt(b"secret", b"aad")
            .expect("encrypt should succeed");
        let result = enc_b.decrypt(&envelope);

        assert!(result.is_err(), "decryption with wrong key should fail");
    }

    #[test]
    fn tampered_ciphertext_fails() {
        let key = random_key();
        let enc = XChaCha20Encryptor::new(key);

        let mut envelope = enc
            .encrypt(b"secret", b"aad")
            .expect("encrypt should succeed");
        // Tamper with a ciphertext byte
        if let Some(byte) = envelope.ciphertext.first_mut() {
            *byte ^= 0xFF;
        }

        let result = enc.decrypt(&envelope);
        assert!(
            result.is_err(),
            "decryption of tampered ciphertext should fail"
        );
    }

    #[test]
    fn tampered_aad_fails() {
        let key = random_key();
        let enc = XChaCha20Encryptor::new(key);

        let mut envelope = enc
            .encrypt(b"secret", b"ctx1")
            .expect("encrypt should succeed");
        // Replace AAD with different context
        envelope.aad = b"ctx2".to_vec();

        let result = enc.decrypt(&envelope);
        assert!(result.is_err(), "decryption with tampered AAD should fail");
    }

    #[test]
    fn empty_plaintext_round_trip() {
        let key = random_key();
        let enc = XChaCha20Encryptor::new(key);

        let envelope = enc.encrypt(b"", b"aad").expect("encrypt should succeed");
        let decrypted = enc.decrypt(&envelope).expect("decrypt should succeed");

        assert!(
            decrypted.is_empty(),
            "decrypted empty plaintext should be empty"
        );
    }

    #[test]
    fn large_plaintext_round_trip() {
        let key = random_key();
        let enc = XChaCha20Encryptor::new(key);

        // 1 MB of random data
        let mut plaintext = vec![0u8; 1_000_000];
        use rand::RngCore;
        rand::rngs::OsRng.fill_bytes(&mut plaintext);

        let envelope = enc
            .encrypt(&plaintext, b"large")
            .expect("encrypt should succeed");
        let decrypted = enc.decrypt(&envelope).expect("decrypt should succeed");

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn nonce_is_random() {
        let key = random_key();
        let enc = XChaCha20Encryptor::new(key);
        let plaintext = b"same plaintext";
        let aad = b"same aad";

        let envelope1 = enc
            .encrypt(plaintext, aad)
            .expect("encrypt 1 should succeed");
        let envelope2 = enc
            .encrypt(plaintext, aad)
            .expect("encrypt 2 should succeed");

        assert_ne!(
            envelope1.nonce, envelope2.nonce,
            "two encryptions should produce different nonces"
        );
    }

    #[test]
    fn aad_is_preserved() {
        let key = random_key();
        let enc = XChaCha20Encryptor::new(key);
        let aad = b"my-context-data";

        let envelope = enc
            .encrypt(b"payload", aad)
            .expect("encrypt should succeed");

        assert_eq!(envelope.aad, aad, "AAD in envelope should match input AAD");
    }
}
