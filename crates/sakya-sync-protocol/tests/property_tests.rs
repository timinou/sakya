//! Property-based tests for sakya-sync-protocol

use proptest::prelude::*;
use sakya_sync_protocol::{EncryptedEnvelope, ErrorCode, Fragmenter, Reassembler, SyncMessage};
use uuid::Uuid;

// Strategy for generating UUIDs
fn uuid_strategy() -> impl Strategy<Value = Uuid> {
    any::<u128>().prop_map(|v| Uuid::from_u128(v))
}

// Strategy for generating EncryptedEnvelope
fn encrypted_envelope_strategy() -> impl Strategy<Value = EncryptedEnvelope> {
    (
        prop::collection::vec(any::<u8>(), 0..50),  // nonce
        prop::collection::vec(any::<u8>(), 0..500), // ciphertext
        prop::collection::vec(any::<u8>(), 0..50),  // aad
    )
        .prop_map(|(nonce, ciphertext, aad)| EncryptedEnvelope {
            nonce,
            ciphertext,
            aad,
        })
}

// Strategy for generating ErrorCode
fn error_code_strategy() -> impl Strategy<Value = ErrorCode> {
    prop_oneof![
        Just(ErrorCode::Unauthorized),
        Just(ErrorCode::RoomNotFound),
        Just(ErrorCode::InvalidUpdate),
        Just(ErrorCode::SnapshotRequired),
        Just(ErrorCode::RateLimited),
        Just(ErrorCode::InternalError),
    ]
}

// Strategy for generating SyncMessage
fn sync_message_strategy() -> impl Strategy<Value = SyncMessage> {
    prop_oneof![
        // Auth
        any::<String>().prop_map(|token| SyncMessage::Auth { token }),
        // AuthOk
        any::<String>().prop_map(|server_version| SyncMessage::AuthOk { server_version }),
        // JoinRoom
        uuid_strategy().prop_map(|project_id| SyncMessage::JoinRoom { project_id }),
        // RoomJoined
        (uuid_strategy(), prop::collection::vec(any::<u8>(), 0..20)).prop_map(
            |(project_id, server_version)| SyncMessage::RoomJoined {
                project_id,
                server_version,
            }
        ),
        // LeaveRoom
        uuid_strategy().prop_map(|project_id| SyncMessage::LeaveRoom { project_id }),
        // EncryptedUpdate
        (
            uuid_strategy(),
            encrypted_envelope_strategy(),
            any::<u64>(),
            uuid_strategy(),
        )
            .prop_map(|(project_id, envelope, sequence, device_id)| {
                SyncMessage::EncryptedUpdate {
                    project_id,
                    envelope,
                    sequence,
                    device_id,
                }
            }),
        // EncryptedSnapshot
        (
            uuid_strategy(),
            encrypted_envelope_strategy(),
            uuid_strategy(),
        )
            .prop_map(|(project_id, envelope, snapshot_id)| {
                SyncMessage::EncryptedSnapshot {
                    project_id,
                    envelope,
                    snapshot_id,
                }
            }),
        // SyncRequest
        (uuid_strategy(), any::<u64>()).prop_map(|(project_id, since_sequence)| {
            SyncMessage::SyncRequest {
                project_id,
                since_sequence,
            }
        }),
        // Ephemeral
        (uuid_strategy(), prop::collection::vec(any::<u8>(), 0..100))
            .prop_map(|(project_id, data)| SyncMessage::Ephemeral { project_id, data }),
        // Error
        (error_code_strategy(), any::<String>())
            .prop_map(|(code, message)| SyncMessage::Error { code, message }),
        // Ping
        Just(SyncMessage::Ping),
        // Pong
        Just(SyncMessage::Pong),
    ]
}

// Strategy for generating SyncMessage suitable for use in SyncResponse updates
fn sync_update_message_strategy() -> impl Strategy<Value = SyncMessage> {
    prop_oneof![
        // Only include variants that make sense as updates
        (
            uuid_strategy(),
            encrypted_envelope_strategy(),
            any::<u64>(),
            uuid_strategy(),
        )
            .prop_map(|(project_id, envelope, sequence, device_id)| {
                SyncMessage::EncryptedUpdate {
                    project_id,
                    envelope,
                    sequence,
                    device_id,
                }
            }),
    ]
}

proptest! {
    #[test]
    fn sync_message_serde_round_trip(msg in sync_message_strategy()) {
        // Serialize to JSON
        let json = msg.to_json().expect("Serialization should succeed");

        // Deserialize back
        let deserialized = SyncMessage::from_json(&json)
            .expect("Deserialization should succeed");

        // Should be equal
        prop_assert_eq!(msg, deserialized);

        // JSON should contain the type field
        prop_assert!(json.contains("\"type\":"));
    }

    #[test]
    fn fragment_reassemble_identity(
        data in prop::collection::vec(any::<u8>(), 0..500000),
        max_fragment_size in 1000..100000usize,
    ) {
        let fragmenter = Fragmenter::new(max_fragment_size);
        let mut reassembler = Reassembler::new(60);

        // Fragment the data
        let fragments = fragmenter.fragment(&data);

        // Verify fragment count
        if data.is_empty() {
            prop_assert_eq!(fragments.len(), 1);
        } else {
            let expected_fragments = (data.len() + max_fragment_size - 1) / max_fragment_size;
            prop_assert_eq!(fragments.len(), expected_fragments);
        }

        // Reassemble
        let mut result = None;
        for fragment in fragments {
            result = reassembler.add_fragment(fragment)
                .expect("Fragment addition should succeed");
        }

        // Should have reassembled data
        let reassembled = result.expect("Should have reassembled data");
        prop_assert_eq!(data, reassembled);
    }

    #[test]
    fn fragment_ordering_independence(
        data in prop::collection::vec(any::<u8>(), 100..10000),
        max_fragment_size in 100..1000usize,
    ) {
        let fragmenter = Fragmenter::new(max_fragment_size);
        let mut reassembler = Reassembler::new(60);

        // Fragment the data
        let fragments = fragmenter.fragment(&data);

        // Only test if we have multiple fragments
        if fragments.len() > 1 {
            // Shuffle the fragments using a simple approach
            use std::collections::VecDeque;
            let mut shuffled: VecDeque<_> = fragments.into();

            // Simple shuffle: reverse order
            let mut fragments_reversed = Vec::new();
            while let Some(frag) = shuffled.pop_back() {
                fragments_reversed.push(frag);
            }

            // Reassemble in reversed order
            let mut result = None;
            for fragment in fragments_reversed {
                result = reassembler.add_fragment(fragment)
                    .expect("Fragment addition should succeed");
            }

            // Should still produce the same data
            let reassembled = result.expect("Should have reassembled data");
            prop_assert_eq!(data, reassembled);
        }
    }

    #[test]
    fn sync_response_with_nested_messages_round_trip(
        project_id in uuid_strategy(),
        updates in prop::collection::vec(sync_update_message_strategy(), 0..5),
        has_snapshot in any::<bool>(),
        snapshot_id in uuid_strategy(),
        envelope in encrypted_envelope_strategy(),
    ) {
        let latest_snapshot = if has_snapshot {
            Some(Box::new(SyncMessage::EncryptedSnapshot {
                project_id,
                envelope,
                snapshot_id,
            }))
        } else {
            None
        };

        let msg = SyncMessage::SyncResponse {
            project_id,
            updates,
            latest_snapshot,
        };

        // Serialize and deserialize
        let json = msg.to_json().expect("Serialization should succeed");
        let deserialized = SyncMessage::from_json(&json)
            .expect("Deserialization should succeed");

        prop_assert_eq!(msg, deserialized);
    }

    #[test]
    fn fragmenter_needs_fragmentation_consistency(
        data_len in 0..500000usize,
        max_fragment_size in 1..100000usize,
    ) {
        let data = vec![0u8; data_len];
        let fragmenter = Fragmenter::new(max_fragment_size);

        let needs_frag = fragmenter.needs_fragmentation(&data);
        let fragments = fragmenter.fragment(&data);

        if needs_frag {
            prop_assert!(fragments.len() > 1);
            prop_assert!(data.len() > max_fragment_size);
        } else {
            prop_assert_eq!(fragments.len(), 1);
            prop_assert!(data.len() <= max_fragment_size);
        }
    }

    #[test]
    fn fragment_total_data_preserved(
        data in prop::collection::vec(any::<u8>(), 0..100000),
        max_fragment_size in 100..10000usize,
    ) {
        let fragmenter = Fragmenter::new(max_fragment_size);
        let fragments = fragmenter.fragment(&data);

        // All fragments should have the same message_id
        if !fragments.is_empty() {
            let msg_id = fragments[0].message_id;
            for frag in &fragments {
                prop_assert_eq!(frag.message_id, msg_id);
            }
        }

        // Total data length should be preserved
        let total_len: usize = fragments.iter().map(|f| f.data.len()).sum();
        prop_assert_eq!(total_len, data.len());

        // Each fragment (except possibly the last) should be max_fragment_size
        for (i, frag) in fragments.iter().enumerate() {
            if i < fragments.len() - 1 {
                // Not the last fragment
                prop_assert_eq!(frag.data.len(), max_fragment_size);
            } else {
                // Last fragment
                if data.is_empty() {
                    prop_assert_eq!(frag.data.len(), 0);
                } else {
                    let expected_last_size = data.len() % max_fragment_size;
                    if expected_last_size == 0 {
                        prop_assert_eq!(frag.data.len(), max_fragment_size);
                    } else {
                        prop_assert_eq!(frag.data.len(), expected_last_size);
                    }
                }
            }
        }
    }

    #[test]
    fn duplicate_fragments_handled_correctly(
        data in prop::collection::vec(any::<u8>(), 100..1000),
        max_fragment_size in 50..200usize,
        duplicate_indices in prop::collection::vec(any::<usize>(), 0..10),
    ) {
        let fragmenter = Fragmenter::new(max_fragment_size);
        let mut reassembler = Reassembler::new(60);

        let fragments = fragmenter.fragment(&data);

        if fragments.len() > 1 {
            // Add each fragment, potentially multiple times based on duplicate_indices
            let mut added_all = false;
            let mut result = None;

            for (i, fragment) in fragments.iter().enumerate() {
                // Always add the fragment at least once
                result = reassembler.add_fragment(fragment.clone())
                    .expect("Fragment addition should succeed");

                if result.is_some() {
                    added_all = true;
                    break;
                }

                // Add duplicates
                for &dup_idx in &duplicate_indices {
                    if dup_idx % fragments.len() == i {
                        result = reassembler.add_fragment(fragment.clone())
                            .expect("Duplicate fragment addition should succeed");

                        if result.is_some() {
                            added_all = true;
                            break;
                        }
                    }
                }

                if added_all {
                    break;
                }
            }

            // If we haven't completed yet, add any remaining fragments
            if !added_all {
                for fragment in &fragments {
                    if let Some(r) = reassembler.add_fragment(fragment.clone())
                        .expect("Fragment addition should succeed") {
                        result = Some(r);
                        break;
                    }
                }
            }

            let reassembled = result.expect("Should have reassembled data");
            prop_assert_eq!(data, reassembled);
        }
    }
}
