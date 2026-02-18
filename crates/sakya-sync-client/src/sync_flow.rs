//! Higher-level sync flow helpers for the sync client.
//!
//! Provides envelope conversion between `sakya_crypto` and `sakya_sync_protocol`,
//! encrypt/decrypt utilities for CRDT updates, and helpers to build and process
//! sync request/response messages.

use crate::error::SyncClientError;
use sakya_crypto::Encryptor;
use sakya_sync_protocol::SyncMessage;
use uuid::Uuid;

/// Convert a `sakya_crypto::EncryptedEnvelope` (fixed `[u8; 24]` nonce) to
/// a `sakya_sync_protocol::EncryptedEnvelope` (`Vec<u8>` nonce).
pub fn crypto_to_protocol_envelope(
    crypto: &sakya_crypto::EncryptedEnvelope,
) -> sakya_sync_protocol::EncryptedEnvelope {
    sakya_sync_protocol::EncryptedEnvelope {
        nonce: crypto.nonce.to_vec(),
        ciphertext: crypto.ciphertext.clone(),
        aad: crypto.aad.clone(),
    }
}

/// Convert a `sakya_sync_protocol::EncryptedEnvelope` (`Vec<u8>` nonce) to
/// a `sakya_crypto::EncryptedEnvelope` (fixed `[u8; 24]` nonce).
///
/// Returns `SyncClientError::ProtocolError` if the nonce is not exactly 24 bytes.
pub fn protocol_to_crypto_envelope(
    proto: &sakya_sync_protocol::EncryptedEnvelope,
) -> Result<sakya_crypto::EncryptedEnvelope, SyncClientError> {
    let nonce: [u8; 24] = proto.nonce.as_slice().try_into().map_err(|_| {
        SyncClientError::ProtocolError(format!(
            "Invalid nonce length: expected 24 bytes, got {}",
            proto.nonce.len()
        ))
    })?;
    Ok(sakya_crypto::EncryptedEnvelope {
        nonce,
        ciphertext: proto.ciphertext.clone(),
        aad: proto.aad.clone(),
    })
}

/// Encrypt a plaintext CRDT update for transmission over the wire.
///
/// The project ID bytes are used as the AAD (associated authenticated data),
/// binding the ciphertext to a specific project.
pub fn encrypt_update(
    encryptor: &sakya_crypto::XChaCha20Encryptor,
    plaintext: &[u8],
    project_id: &Uuid,
) -> Result<sakya_sync_protocol::EncryptedEnvelope, SyncClientError> {
    let crypto_envelope = encryptor
        .encrypt(plaintext, project_id.as_bytes())
        .map_err(|e| SyncClientError::EncryptionError(e.to_string()))?;
    Ok(crypto_to_protocol_envelope(&crypto_envelope))
}

/// Decrypt a received encrypted update from the wire.
///
/// Converts the protocol envelope to a crypto envelope and decrypts it.
pub fn decrypt_update(
    encryptor: &sakya_crypto::XChaCha20Encryptor,
    envelope: &sakya_sync_protocol::EncryptedEnvelope,
) -> Result<Vec<u8>, SyncClientError> {
    let crypto_envelope = protocol_to_crypto_envelope(envelope)?;
    encryptor
        .decrypt(&crypto_envelope)
        .map_err(|e| SyncClientError::DecryptionError(e.to_string()))
}

/// Build a `SyncRequest` message asking for updates since the given sequence number.
pub fn build_sync_request(project_id: Uuid, since_sequence: u64) -> SyncMessage {
    SyncMessage::SyncRequest {
        project_id,
        since_sequence,
    }
}

/// Extract and decrypt all updates from a `SyncResponse` message.
///
/// If the response contains a snapshot, it is decrypted first, followed by
/// all individual updates. The returned `Vec<Vec<u8>>` preserves this ordering:
/// snapshot (if any) first, then updates in the order they appear.
///
/// Returns `SyncClientError::ProtocolError` if the message is not a `SyncResponse`.
pub fn process_sync_response(
    response: &SyncMessage,
    encryptor: &sakya_crypto::XChaCha20Encryptor,
) -> Result<Vec<Vec<u8>>, SyncClientError> {
    let (updates, latest_snapshot) = match response {
        SyncMessage::SyncResponse {
            updates,
            latest_snapshot,
            ..
        } => (updates, latest_snapshot),
        _ => {
            return Err(SyncClientError::ProtocolError(
                "Expected SyncResponse message".to_string(),
            ))
        }
    };

    let mut result = Vec::new();

    // Process snapshot first if available
    if let Some(snapshot_msg) = latest_snapshot {
        if let SyncMessage::EncryptedSnapshot { envelope, .. } = snapshot_msg.as_ref() {
            let plaintext = decrypt_update(encryptor, envelope)?;
            result.push(plaintext);
        }
    }

    // Then process individual updates
    for update in updates {
        if let SyncMessage::EncryptedUpdate { envelope, .. } = update {
            let plaintext = decrypt_update(encryptor, envelope)?;
            result.push(plaintext);
        }
    }

    Ok(result)
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

    // ---------------------------------------------------------------
    // 1. crypto_to_protocol_envelope round trip
    // ---------------------------------------------------------------
    #[test]
    fn crypto_to_protocol_round_trip() {
        let crypto = sakya_crypto::EncryptedEnvelope {
            nonce: [42u8; 24],
            ciphertext: vec![0xDE, 0xAD, 0xBE, 0xEF],
            aad: vec![0xAA, 0xBB],
        };

        let proto = crypto_to_protocol_envelope(&crypto);
        assert_eq!(proto.nonce, crypto.nonce.to_vec());
        assert_eq!(proto.ciphertext, crypto.ciphertext);
        assert_eq!(proto.aad, crypto.aad);

        // Convert back
        let round_tripped = protocol_to_crypto_envelope(&proto).unwrap();
        assert_eq!(round_tripped.nonce, crypto.nonce);
        assert_eq!(round_tripped.ciphertext, crypto.ciphertext);
        assert_eq!(round_tripped.aad, crypto.aad);
    }

    // ---------------------------------------------------------------
    // 2. protocol_to_crypto_envelope with invalid nonce length
    // ---------------------------------------------------------------
    #[test]
    fn protocol_to_crypto_invalid_nonce_length() {
        let proto = sakya_sync_protocol::EncryptedEnvelope {
            nonce: vec![1, 2, 3], // Only 3 bytes, need 24
            ciphertext: vec![0xDE, 0xAD],
            aad: vec![],
        };

        let err = protocol_to_crypto_envelope(&proto).unwrap_err();
        match err {
            SyncClientError::ProtocolError(msg) => {
                assert!(
                    msg.contains("24"),
                    "Error should mention expected length 24, got: {msg}"
                );
                assert!(
                    msg.contains("3"),
                    "Error should mention actual length 3, got: {msg}"
                );
            }
            other => panic!("Expected ProtocolError, got: {other:?}"),
        }
    }

    // ---------------------------------------------------------------
    // 3. encrypt_update produces valid envelope
    // ---------------------------------------------------------------
    #[test]
    fn encrypt_update_produces_valid_envelope() {
        let key = random_key();
        let encryptor = sakya_crypto::XChaCha20Encryptor::new(key);
        let project_id = Uuid::new_v4();
        let plaintext = b"crdt update data";

        let proto_envelope = encrypt_update(&encryptor, plaintext, &project_id).unwrap();

        // Nonce should be 24 bytes
        assert_eq!(proto_envelope.nonce.len(), 24);
        // Ciphertext should be non-empty (plaintext + auth tag)
        assert!(!proto_envelope.ciphertext.is_empty());
        // AAD should be the project ID bytes
        assert_eq!(proto_envelope.aad, project_id.as_bytes().to_vec());
    }

    // ---------------------------------------------------------------
    // 4. decrypt_update reverses encrypt_update
    // ---------------------------------------------------------------
    #[test]
    fn decrypt_update_reverses_encrypt_update() {
        let key = random_key();
        let encryptor = sakya_crypto::XChaCha20Encryptor::new(key);
        let project_id = Uuid::new_v4();
        let plaintext = b"hello sync world";

        let proto_envelope = encrypt_update(&encryptor, plaintext, &project_id).unwrap();
        let decrypted = decrypt_update(&encryptor, &proto_envelope).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    // ---------------------------------------------------------------
    // 5. encrypt/decrypt with different data payloads
    // ---------------------------------------------------------------
    #[test]
    fn encrypt_decrypt_different_data() {
        let key = random_key();
        let encryptor = sakya_crypto::XChaCha20Encryptor::new(key);
        let project_id = Uuid::new_v4();

        let payloads: &[&[u8]] = &[
            b"",                // empty
            b"a",               // single byte
            b"short",           // short
            &[0xFF; 1024],      // 1KB
            &[0xAB; 64 * 1024], // 64KB
        ];

        for payload in payloads {
            let envelope = encrypt_update(&encryptor, payload, &project_id).unwrap();
            let decrypted = decrypt_update(&encryptor, &envelope).unwrap();
            assert_eq!(
                &decrypted,
                payload,
                "Round-trip failed for payload of length {}",
                payload.len()
            );
        }
    }

    // ---------------------------------------------------------------
    // 6. build_sync_request produces correct message
    // ---------------------------------------------------------------
    #[test]
    fn build_sync_request_correct_message() {
        let project_id = Uuid::new_v4();
        let since_seq = 42;

        let msg = build_sync_request(project_id, since_seq);
        match msg {
            SyncMessage::SyncRequest {
                project_id: pid,
                since_sequence,
            } => {
                assert_eq!(pid, project_id);
                assert_eq!(since_sequence, since_seq);
            }
            other => panic!("Expected SyncRequest, got: {other:?}"),
        }

        // Should serialize to valid JSON
        let json = msg.to_json().unwrap();
        assert!(json.contains("SyncRequest"));
        let round_tripped = SyncMessage::from_json(&json).unwrap();
        assert_eq!(round_tripped, msg);
    }

    // ---------------------------------------------------------------
    // 7. process_sync_response with updates only
    // ---------------------------------------------------------------
    #[test]
    fn process_sync_response_with_updates() {
        let key = random_key();
        let encryptor = sakya_crypto::XChaCha20Encryptor::new(key);
        let project_id = Uuid::new_v4();
        let device_id = Uuid::new_v4();

        let plaintext_a = b"update alpha";
        let plaintext_b = b"update beta";

        let env_a = encrypt_update(&encryptor, plaintext_a, &project_id).unwrap();
        let env_b = encrypt_update(&encryptor, plaintext_b, &project_id).unwrap();

        let response = SyncMessage::SyncResponse {
            project_id,
            updates: vec![
                SyncMessage::EncryptedUpdate {
                    project_id,
                    envelope: env_a,
                    sequence: 1,
                    device_id,
                },
                SyncMessage::EncryptedUpdate {
                    project_id,
                    envelope: env_b,
                    sequence: 2,
                    device_id,
                },
            ],
            latest_snapshot: None,
        };

        let decrypted = process_sync_response(&response, &encryptor).unwrap();
        assert_eq!(decrypted.len(), 2);
        assert_eq!(decrypted[0], plaintext_a);
        assert_eq!(decrypted[1], plaintext_b);
    }

    // ---------------------------------------------------------------
    // 8. process_sync_response with snapshot + updates
    // ---------------------------------------------------------------
    #[test]
    fn process_sync_response_with_snapshot_and_updates() {
        let key = random_key();
        let encryptor = sakya_crypto::XChaCha20Encryptor::new(key);
        let project_id = Uuid::new_v4();
        let device_id = Uuid::new_v4();

        let snapshot_data = b"full document snapshot";
        let update_data = b"incremental update";

        let snap_env = encrypt_update(&encryptor, snapshot_data, &project_id).unwrap();
        let upd_env = encrypt_update(&encryptor, update_data, &project_id).unwrap();

        let response = SyncMessage::SyncResponse {
            project_id,
            updates: vec![SyncMessage::EncryptedUpdate {
                project_id,
                envelope: upd_env,
                sequence: 10,
                device_id,
            }],
            latest_snapshot: Some(Box::new(SyncMessage::EncryptedSnapshot {
                project_id,
                envelope: snap_env,
                snapshot_id: Uuid::new_v4(),
            })),
        };

        let decrypted = process_sync_response(&response, &encryptor).unwrap();
        // Snapshot comes first, then updates
        assert_eq!(decrypted.len(), 2);
        assert_eq!(decrypted[0], snapshot_data);
        assert_eq!(decrypted[1], update_data);
    }

    // ---------------------------------------------------------------
    // Extra: process_sync_response rejects non-SyncResponse
    // ---------------------------------------------------------------
    #[test]
    fn process_sync_response_rejects_non_sync_response() {
        let key = random_key();
        let encryptor = sakya_crypto::XChaCha20Encryptor::new(key);

        let msg = SyncMessage::Ping;
        let err = process_sync_response(&msg, &encryptor).unwrap_err();
        match err {
            SyncClientError::ProtocolError(msg) => {
                assert!(msg.contains("SyncResponse"));
            }
            other => panic!("Expected ProtocolError, got: {other:?}"),
        }
    }

    // ---------------------------------------------------------------
    // Extra: decrypt with wrong key fails
    // ---------------------------------------------------------------
    #[test]
    fn decrypt_with_wrong_key_fails() {
        let key_a = random_key();
        let key_b = random_key();
        let enc_a = sakya_crypto::XChaCha20Encryptor::new(key_a);
        let enc_b = sakya_crypto::XChaCha20Encryptor::new(key_b);
        let project_id = Uuid::new_v4();

        let envelope = encrypt_update(&enc_a, b"secret data", &project_id).unwrap();
        let err = decrypt_update(&enc_b, &envelope).unwrap_err();
        assert!(
            matches!(err, SyncClientError::DecryptionError(_)),
            "Expected DecryptionError, got: {err:?}"
        );
    }
}
