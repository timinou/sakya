//! X25519 Diffie-Hellman key exchange for device pairing.
//!
//! Provides [`EphemeralKeyPair`] for generating temporary X25519 keypairs
//! and deriving shared secrets via ECDH + HKDF-SHA256.

use hkdf::Hkdf;
use rand::rngs::OsRng;
use sha2::Sha256;
use x25519_dalek::{PublicKey, StaticSecret};

/// An ephemeral X25519 keypair for Diffie-Hellman key exchange.
///
/// Used during device pairing to derive a shared secret that encrypts
/// the provisioning payload.
pub struct EphemeralKeyPair {
    secret: StaticSecret,
    public: PublicKey,
}

impl EphemeralKeyPair {
    /// Generate a new random X25519 keypair.
    pub fn generate() -> Self {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);
        Self { secret, public }
    }

    /// Get the public key for sharing with the remote party.
    pub fn public_key(&self) -> PublicKey {
        self.public
    }

    /// Get the raw public key bytes (32 bytes) for encoding/transmission.
    pub fn public_key_bytes(&self) -> [u8; 32] {
        *self.public.as_bytes()
    }

    /// Perform X25519 Diffie-Hellman key agreement and derive a 256-bit
    /// encryption key via HKDF-SHA256.
    ///
    /// Both parties calling this with each other's public key will derive
    /// the same 32-byte key.
    pub fn derive_shared_secret(&self, remote_public: &PublicKey) -> [u8; 32] {
        let shared_secret = self.secret.diffie_hellman(remote_public);

        // Derive a proper encryption key from the raw DH output using HKDF-SHA256
        let hkdf = Hkdf::<Sha256>::new(None, shared_secret.as_bytes());
        let mut okm = [0u8; 32];
        hkdf.expand(b"sakya-key-exchange-v1", &mut okm)
            .expect("32 bytes is a valid HKDF-SHA256 output length");
        okm
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encryptor::{Encryptor, XChaCha20Encryptor};

    #[test]
    fn two_parties_derive_same_secret() {
        let alice = EphemeralKeyPair::generate();
        let bob = EphemeralKeyPair::generate();

        let secret_a = alice.derive_shared_secret(&bob.public_key());
        let secret_b = bob.derive_shared_secret(&alice.public_key());

        assert_eq!(
            secret_a, secret_b,
            "both parties should derive the same shared secret"
        );
    }

    #[test]
    fn different_pairs_different_secrets() {
        let alice = EphemeralKeyPair::generate();
        let bob = EphemeralKeyPair::generate();
        let charlie = EphemeralKeyPair::generate();

        let secret_ab = alice.derive_shared_secret(&bob.public_key());
        let secret_ac = alice.derive_shared_secret(&charlie.public_key());

        assert_ne!(
            secret_ab, secret_ac,
            "different key pairs should produce different shared secrets"
        );
    }

    #[test]
    fn public_key_bytes_round_trip() {
        let kp = EphemeralKeyPair::generate();
        let bytes = kp.public_key_bytes();
        let reconstructed = PublicKey::from(bytes);

        assert_eq!(
            kp.public_key().as_bytes(),
            reconstructed.as_bytes(),
            "public key bytes should round-trip correctly"
        );
    }

    #[test]
    fn derived_key_is_32_bytes() {
        let alice = EphemeralKeyPair::generate();
        let bob = EphemeralKeyPair::generate();

        let secret = alice.derive_shared_secret(&bob.public_key());

        assert_eq!(secret.len(), 32, "derived key should be 32 bytes");
    }

    #[test]
    fn shared_secret_works_with_encryptor() {
        let alice = EphemeralKeyPair::generate();
        let bob = EphemeralKeyPair::generate();

        let shared_key = alice.derive_shared_secret(&bob.public_key());
        let enc = XChaCha20Encryptor::new(shared_key);

        let plaintext = b"provisioning payload";
        let aad = b"pairing-v1";

        let envelope = enc.encrypt(plaintext, aad).expect("encrypt should succeed");
        let decrypted = enc.decrypt(&envelope).expect("decrypt should succeed");

        assert_eq!(decrypted, plaintext);
    }
}
