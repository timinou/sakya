//! Provisioning payload encryption for secure device-to-device key transfer.
//!
//! During device provisioning, an existing device transfers account credentials
//! and document encryption keys to a newly paired device. The
//! [`ProvisioningPayload`] is encrypted end-to-end using a shared secret
//! derived via X25519 Diffie-Hellman key exchange, then sealed with
//! XChaCha20-Poly1305 AEAD.
//!
//! # Protocol
//!
//! 1. Both devices generate [`EphemeralKeyPair`]s and exchange public keys
//!    (e.g., via QR code / [`PairingPayload`]).
//! 2. The existing device constructs a [`ProvisioningPayload`] containing the
//!    account ID, per-project document keys, and a JWT auth token.
//! 3. It calls [`ProvisioningPayload::encrypt`] with its local keypair and
//!    the new device's public key.
//! 4. The encrypted envelope is transmitted over the relay.
//! 5. The new device calls [`ProvisioningPayload::decrypt`] with its own
//!    keypair and the existing device's public key, recovering the payload.
//!
//! The account ID is bound into the AEAD as Additional Authenticated Data
//! (AAD), preventing payload substitution attacks across accounts.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use x25519_dalek::PublicKey;

use crate::encryptor::{EncryptedEnvelope, Encryptor, XChaCha20Encryptor};
use crate::error::{CryptoError, Result};
use crate::key_exchange::EphemeralKeyPair;

/// Payload exchanged during device provisioning.
///
/// Sent by an existing (already authenticated) device to a newly paired device.
/// Contains everything the new device needs to decrypt project data and
/// authenticate with the sync server.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ProvisioningPayload {
    /// The user's account identifier.
    pub account_id: Uuid,
    /// Per-project document encryption keys: `(project_id, key_bytes)`.
    ///
    /// Each entry maps a project UUID to its 256-bit document key.
    pub document_keys: Vec<(Uuid, Vec<u8>)>,
    /// JWT authentication token for the sync server.
    pub jwt_token: String,
}

impl ProvisioningPayload {
    /// Create a new provisioning payload.
    pub fn new(account_id: Uuid, document_keys: Vec<(Uuid, Vec<u8>)>, jwt_token: String) -> Self {
        Self {
            account_id,
            document_keys,
            jwt_token,
        }
    }

    /// Encrypt this payload for transmission to a remote device.
    ///
    /// Derives a shared secret from the local ephemeral keypair and the
    /// remote device's X25519 public key, then encrypts the JSON-serialized
    /// payload using XChaCha20-Poly1305. The `account_id` bytes are used as
    /// Additional Authenticated Data (AAD), binding the ciphertext to this
    /// specific account.
    pub fn encrypt(
        &self,
        local_keypair: &EphemeralKeyPair,
        remote_public_key: &[u8; 32],
    ) -> Result<EncryptedEnvelope> {
        let remote_pk = PublicKey::from(*remote_public_key);
        let shared_secret = local_keypair.derive_shared_secret(&remote_pk);
        let encryptor = XChaCha20Encryptor::new(shared_secret);

        let plaintext = serde_json::to_vec(self).map_err(|e| {
            CryptoError::EncryptionFailed(format!("failed to serialize provisioning payload: {e}"))
        })?;

        let aad = self.account_id.as_bytes();

        encryptor.encrypt(&plaintext, aad)
    }

    /// Decrypt a provisioning payload received from a remote device.
    ///
    /// Derives the same shared secret from the local ephemeral keypair and
    /// the remote device's X25519 public key, then decrypts and deserializes
    /// the payload. The AAD (account ID) is verified as part of AEAD
    /// decryption -- any tampering causes an error.
    pub fn decrypt(
        envelope: &EncryptedEnvelope,
        local_keypair: &EphemeralKeyPair,
        remote_public_key: &[u8; 32],
    ) -> Result<Self> {
        let remote_pk = PublicKey::from(*remote_public_key);
        let shared_secret = local_keypair.derive_shared_secret(&remote_pk);
        let encryptor = XChaCha20Encryptor::new(shared_secret);

        let plaintext = encryptor.decrypt(envelope)?;

        serde_json::from_slice(&plaintext).map_err(|e| {
            CryptoError::DecryptionFailed(format!(
                "failed to deserialize provisioning payload: {e}"
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key_exchange::EphemeralKeyPair;

    /// Helper: build a sample provisioning payload with 3 projects.
    fn sample_payload() -> ProvisioningPayload {
        let account_id = Uuid::parse_str("a1a2a3a4-b1b2-c1c2-d1d2-e1e2e3e4e5e6").unwrap();
        let document_keys = vec![
            (Uuid::new_v4(), vec![1u8; 32]),
            (Uuid::new_v4(), vec![2u8; 32]),
            (Uuid::new_v4(), vec![3u8; 32]),
        ];
        let jwt_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test.signature".to_string();

        ProvisioningPayload::new(account_id, document_keys, jwt_token)
    }

    #[test]
    fn encrypt_decrypt_round_trip() {
        let alice = EphemeralKeyPair::generate();
        let bob = EphemeralKeyPair::generate();

        let payload = sample_payload();

        // Alice encrypts with her keypair + Bob's public key
        let envelope = payload
            .encrypt(&alice, &bob.public_key_bytes())
            .expect("encryption should succeed");

        // Bob decrypts with his keypair + Alice's public key
        let decrypted = ProvisioningPayload::decrypt(&envelope, &bob, &alice.public_key_bytes())
            .expect("decryption should succeed");

        assert_eq!(payload.account_id, decrypted.account_id);
        assert_eq!(payload.document_keys, decrypted.document_keys);
        assert_eq!(payload.jwt_token, decrypted.jwt_token);
    }

    #[test]
    fn wrong_key_fails() {
        let alice = EphemeralKeyPair::generate();
        let bob = EphemeralKeyPair::generate();
        let eve = EphemeralKeyPair::generate();

        let payload = sample_payload();

        // Alice encrypts for Bob
        let envelope = payload
            .encrypt(&alice, &bob.public_key_bytes())
            .expect("encryption should succeed");

        // Eve (third party) tries to decrypt with her own keypair
        let result = ProvisioningPayload::decrypt(&envelope, &eve, &alice.public_key_bytes());

        assert!(
            result.is_err(),
            "decryption with a third-party keypair should fail"
        );
    }

    #[test]
    fn payload_serialization() {
        let account_id = Uuid::parse_str("b2b3b4b5-c2c3-d2d3-e2e3-f2f3f4f5f6f7").unwrap();
        let project_ids: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();
        let document_keys: Vec<(Uuid, Vec<u8>)> = project_ids
            .iter()
            .enumerate()
            .map(|(i, &id)| (id, vec![(i + 10) as u8; 32]))
            .collect();
        let jwt_token = "tok.en.value".to_string();

        let payload =
            ProvisioningPayload::new(account_id, document_keys.clone(), jwt_token.clone());

        // Serialize to JSON
        let json = serde_json::to_vec(&payload).expect("serialization should succeed");

        // Deserialize back
        let restored: ProvisioningPayload =
            serde_json::from_slice(&json).expect("deserialization should succeed");

        assert_eq!(
            payload, restored,
            "JSON round-trip should preserve all fields"
        );
        assert_eq!(restored.account_id, account_id);
        assert_eq!(restored.document_keys.len(), 5);
        assert_eq!(restored.jwt_token, jwt_token);

        // Verify each key entry
        for (i, (id, key_bytes)) in restored.document_keys.iter().enumerate() {
            assert_eq!(*id, project_ids[i]);
            assert_eq!(key_bytes.len(), 32);
            assert!(key_bytes.iter().all(|&b| b == (i + 10) as u8));
        }
    }

    #[test]
    fn large_payload() {
        let alice = EphemeralKeyPair::generate();
        let bob = EphemeralKeyPair::generate();

        let account_id = Uuid::new_v4();
        let document_keys: Vec<(Uuid, Vec<u8>)> = (0..150)
            .map(|i| {
                let mut key_bytes = vec![0u8; 32];
                // Fill with a recognizable pattern per project
                for (j, byte) in key_bytes.iter_mut().enumerate() {
                    *byte = ((i * 7 + j) % 256) as u8;
                }
                (Uuid::new_v4(), key_bytes)
            })
            .collect();
        let jwt_token = "a]".repeat(500); // Reasonably long token

        let payload = ProvisioningPayload::new(account_id, document_keys, jwt_token);

        // Encrypt
        let envelope = payload
            .encrypt(&alice, &bob.public_key_bytes())
            .expect("encrypting large payload should succeed");

        // Ciphertext should be non-trivial size
        assert!(
            envelope.ciphertext.len() > 1000,
            "ciphertext for 150 keys should be substantial, got {} bytes",
            envelope.ciphertext.len()
        );

        // Decrypt and verify
        let decrypted = ProvisioningPayload::decrypt(&envelope, &bob, &alice.public_key_bytes())
            .expect("decrypting large payload should succeed");

        assert_eq!(decrypted.account_id, payload.account_id);
        assert_eq!(decrypted.document_keys.len(), 150);
        assert_eq!(decrypted.jwt_token, payload.jwt_token);

        // Spot-check a few key entries
        for i in [0, 49, 99, 149] {
            assert_eq!(
                decrypted.document_keys[i], payload.document_keys[i],
                "document key at index {i} should match"
            );
        }
    }

    #[test]
    fn tampered_aad_fails_decrypt() {
        let alice = EphemeralKeyPair::generate();
        let bob = EphemeralKeyPair::generate();

        let payload = sample_payload();

        let mut envelope = payload
            .encrypt(&alice, &bob.public_key_bytes())
            .expect("encryption should succeed");

        // Tamper with the AAD (account_id bytes) -- simulate a payload
        // substitution attack where an attacker replaces the account_id.
        let fake_account = Uuid::new_v4();
        envelope.aad = fake_account.as_bytes().to_vec();

        let result = ProvisioningPayload::decrypt(&envelope, &bob, &alice.public_key_bytes());

        assert!(
            result.is_err(),
            "decryption with tampered AAD (wrong account_id) should fail"
        );
    }
}
