//! Key rotation on device removal.
//!
//! When a device is removed from a sync group, all document keys must be
//! rotated so that the removed device can no longer decrypt new data.
//! This module generates fresh [`DocumentKey`]s for each project and wraps
//! them into per-device [`EncryptedEnvelope`]s that only the remaining
//! devices can open.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use x25519_dalek::PublicKey;

use crate::encryptor::{Encryptor, XChaCha20Encryptor};
use crate::error::Result;
use crate::{DocumentKey, EncryptedEnvelope, EphemeralKeyPair};

/// Serialization-friendly payload containing the rotated keys.
///
/// Packed into JSON and encrypted for each remaining device.
#[derive(Debug, Serialize, Deserialize)]
struct RotatedKeyPayload {
    /// (project_id, key_bytes) pairs.
    keys: Vec<(Uuid, Vec<u8>)>,
}

/// The result of a key rotation: new per-project keys and encrypted
/// envelopes for every remaining device.
pub struct KeyRotationBundle {
    /// New document key per project (project_id, key).
    pub rotated_keys: Vec<(Uuid, DocumentKey)>,
    /// Encrypted key bundle per remaining device (device_id, envelope).
    pub device_envelopes: Vec<(Uuid, EncryptedEnvelope)>,
}

/// Generate new random document keys for the given project IDs.
///
/// Each project receives an independent 256-bit key sourced from the OS
/// CSPRNG.
pub fn rotate_document_keys(project_ids: &[Uuid]) -> Vec<(Uuid, DocumentKey)> {
    project_ids
        .iter()
        .map(|pid| (*pid, DocumentKey::generate()))
        .collect()
}

/// Create encrypted envelopes containing the rotated keys for each
/// remaining device.
///
/// For every device the function:
/// 1. Derives a shared secret via X25519 DH between `local_keypair` and
///    the device's public key.
/// 2. Constructs an [`XChaCha20Encryptor`] keyed with that shared secret.
/// 3. Serialises the rotated keys as JSON.
/// 4. Encrypts the JSON with the device's UUID bytes as AAD.
pub fn create_rotation_envelopes(
    rotated_keys: &[(Uuid, DocumentKey)],
    remaining_devices: &[(Uuid, [u8; 32])],
    local_keypair: &EphemeralKeyPair,
) -> Result<Vec<(Uuid, EncryptedEnvelope)>> {
    // Pre-serialise the key payload once -- it is the same plaintext for
    // every device (only the encryption key differs).
    let payload = RotatedKeyPayload {
        keys: rotated_keys
            .iter()
            .map(|(pid, dk)| (*pid, dk.as_bytes().to_vec()))
            .collect(),
    };
    let plaintext = serde_json::to_vec(&payload)
        .map_err(|e| crate::error::CryptoError::EncryptionFailed(e.to_string()))?;

    let mut envelopes = Vec::with_capacity(remaining_devices.len());

    for (device_id, pub_key_bytes) in remaining_devices {
        let remote_public = PublicKey::from(*pub_key_bytes);
        let shared_secret = local_keypair.derive_shared_secret(&remote_public);
        let encryptor = XChaCha20Encryptor::new(shared_secret);

        let aad = device_id.as_bytes();
        let envelope = encryptor.encrypt(&plaintext, aad)?;
        envelopes.push((*device_id, envelope));
    }

    Ok(envelopes)
}

/// Generate new document keys **and** wrap them for every remaining device
/// in a single call.
pub fn perform_key_rotation(
    project_ids: &[Uuid],
    remaining_devices: &[(Uuid, [u8; 32])],
    local_keypair: &EphemeralKeyPair,
) -> Result<KeyRotationBundle> {
    let rotated_keys = rotate_document_keys(project_ids);
    let device_envelopes =
        create_rotation_envelopes(&rotated_keys, remaining_devices, local_keypair)?;
    Ok(KeyRotationBundle {
        rotated_keys,
        device_envelopes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encryptor::Encryptor;
    use std::collections::HashSet;

    /// Helper: create an EphemeralKeyPair for a simulated remote device and
    /// return `(device_id, public_key_bytes, keypair)`.
    fn make_device() -> (Uuid, [u8; 32], EphemeralKeyPair) {
        let kp = EphemeralKeyPair::generate();
        (Uuid::new_v4(), kp.public_key_bytes(), kp)
    }

    /// Decrypt an envelope using the device's own keypair and the sender's
    /// public key, returning the deserialised payload.
    fn decrypt_envelope(
        envelope: &EncryptedEnvelope,
        device_id: &Uuid,
        device_kp: &EphemeralKeyPair,
        sender_public: &PublicKey,
    ) -> RotatedKeyPayload {
        let shared = device_kp.derive_shared_secret(sender_public);
        let dec = XChaCha20Encryptor::new(shared);

        // Verify AAD matches device_id
        assert_eq!(
            envelope.aad,
            device_id.as_bytes(),
            "AAD should be the device UUID bytes"
        );

        let plaintext = dec.decrypt(envelope).expect("decryption should succeed");
        serde_json::from_slice(&plaintext).expect("payload should deserialise")
    }

    // ------------------------------------------------------------------
    // Test 1: Each project gets a unique, non-zero key
    // ------------------------------------------------------------------
    #[test]
    fn rotate_generates_new_keys_per_project() {
        let ids: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();
        let rotated = rotate_document_keys(&ids);

        assert_eq!(rotated.len(), ids.len(), "one key per project");

        // All project IDs present and in order
        for (i, (pid, _)) in rotated.iter().enumerate() {
            assert_eq!(pid, &ids[i]);
        }

        // All keys are unique (compare raw bytes)
        let key_set: HashSet<[u8; 32]> = rotated.iter().map(|(_, dk)| *dk.as_bytes()).collect();
        assert_eq!(
            key_set.len(),
            ids.len(),
            "every project should receive a distinct key"
        );

        // No key is all zeros
        for (_, dk) in &rotated {
            assert!(
                dk.as_bytes().iter().any(|&b| b != 0),
                "key should not be all zeros"
            );
        }
    }

    // ------------------------------------------------------------------
    // Test 2: A remaining device can decrypt its envelope and recover keys
    // ------------------------------------------------------------------
    #[test]
    fn rotation_envelopes_decrypt_correctly() {
        let sender = EphemeralKeyPair::generate();
        let (dev_id, dev_pub, dev_kp) = make_device();

        let project_ids: Vec<Uuid> = (0..3).map(|_| Uuid::new_v4()).collect();
        let rotated = rotate_document_keys(&project_ids);

        let envelopes = create_rotation_envelopes(&rotated, &[(dev_id, dev_pub)], &sender)
            .expect("envelope creation should succeed");

        assert_eq!(envelopes.len(), 1);
        let (eid, ref envelope) = envelopes[0];
        assert_eq!(eid, dev_id);

        let payload = decrypt_envelope(envelope, &dev_id, &dev_kp, &sender.public_key());

        assert_eq!(payload.keys.len(), rotated.len());
        for ((pid, dk), (payload_pid, payload_bytes)) in rotated.iter().zip(payload.keys.iter()) {
            assert_eq!(pid, payload_pid, "project IDs must match");
            assert_eq!(
                dk.as_bytes().as_slice(),
                payload_bytes.as_slice(),
                "key bytes must match"
            );
        }
    }

    // ------------------------------------------------------------------
    // Test 3: Different devices get different envelopes (different ciphertexts)
    // ------------------------------------------------------------------
    #[test]
    fn each_device_gets_unique_envelope() {
        let sender = EphemeralKeyPair::generate();
        let (id_a, pub_a, _kp_a) = make_device();
        let (id_b, pub_b, _kp_b) = make_device();

        let project_ids = vec![Uuid::new_v4()];
        let rotated = rotate_document_keys(&project_ids);

        let envelopes =
            create_rotation_envelopes(&rotated, &[(id_a, pub_a), (id_b, pub_b)], &sender)
                .expect("envelope creation should succeed");

        assert_eq!(envelopes.len(), 2);

        let (eid_a, ref env_a) = envelopes[0];
        let (eid_b, ref env_b) = envelopes[1];
        assert_eq!(eid_a, id_a);
        assert_eq!(eid_b, id_b);

        // Ciphertext must differ because the encryption keys are different
        assert_ne!(
            env_a.ciphertext, env_b.ciphertext,
            "envelopes for different devices should have different ciphertexts"
        );
    }

    // ------------------------------------------------------------------
    // Test 4: A device whose key was NOT used cannot decrypt the envelope
    // ------------------------------------------------------------------
    #[test]
    fn wrong_key_fails_decryption() {
        let sender = EphemeralKeyPair::generate();
        let (dev_id, dev_pub, _dev_kp) = make_device();
        let (_rogue_id, _rogue_pub, rogue_kp) = make_device();

        let project_ids = vec![Uuid::new_v4()];
        let rotated = rotate_document_keys(&project_ids);

        let envelopes = create_rotation_envelopes(&rotated, &[(dev_id, dev_pub)], &sender)
            .expect("envelope creation should succeed");

        let (_, ref envelope) = envelopes[0];

        // The rogue device derives a *different* shared secret
        let rogue_shared = rogue_kp.derive_shared_secret(&sender.public_key());
        let rogue_dec = XChaCha20Encryptor::new(rogue_shared);

        let result = rogue_dec.decrypt(envelope);
        assert!(
            result.is_err(),
            "a device not in remaining_devices must not be able to decrypt"
        );
    }

    // ------------------------------------------------------------------
    // Test 5: Empty device list yields empty envelopes; keys still generated
    // ------------------------------------------------------------------
    #[test]
    fn empty_device_list_returns_empty() {
        let sender = EphemeralKeyPair::generate();
        let project_ids: Vec<Uuid> = (0..3).map(|_| Uuid::new_v4()).collect();

        let bundle = perform_key_rotation(&project_ids, &[], &sender)
            .expect("rotation with no devices should succeed");

        assert_eq!(
            bundle.rotated_keys.len(),
            3,
            "keys should still be generated for every project"
        );
        assert!(
            bundle.device_envelopes.is_empty(),
            "no envelopes when there are no remaining devices"
        );
    }

    // ------------------------------------------------------------------
    // Test 6: Old key cannot decrypt data encrypted with the new key
    // ------------------------------------------------------------------
    #[test]
    fn old_key_doesnt_decrypt_new_data() {
        let old_key = DocumentKey::generate();
        let new_key = DocumentKey::generate();

        let new_enc = XChaCha20Encryptor::new(*new_key.as_bytes());
        let old_dec = XChaCha20Encryptor::new(*old_key.as_bytes());

        let envelope = new_enc
            .encrypt(b"post-rotation content", b"chapter-1")
            .expect("encrypt should succeed");

        let result = old_dec.decrypt(&envelope);
        assert!(
            result.is_err(),
            "old key must not decrypt data encrypted with a new key"
        );
    }
}
