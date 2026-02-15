use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{ErrorCode, ProtocolError};

/// Temporary local type -- will be replaced with sakya_crypto::EncryptedEnvelope
/// once the sakya-crypto crate implementation is complete.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EncryptedEnvelope {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub aad: Vec<u8>,
}

/// Wire protocol messages for Sakya sync.
///
/// Uses serde's internally-tagged enum representation (`#[serde(tag = "type")]`)
/// so each JSON message contains a `"type"` field discriminator.
///
/// # Examples
///
/// ```
/// use sakya_sync_protocol::SyncMessage;
///
/// let msg = SyncMessage::Ping;
/// let json = msg.to_json().unwrap();
/// assert!(json.contains(r#""type":"Ping""#));
///
/// let round_tripped = SyncMessage::from_json(&json).unwrap();
/// assert_eq!(msg, round_tripped);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum SyncMessage {
    Auth {
        token: String,
    },
    AuthOk {
        server_version: String,
    },
    JoinRoom {
        project_id: Uuid,
    },
    RoomJoined {
        project_id: Uuid,
        server_version: Vec<u8>,
    },
    LeaveRoom {
        project_id: Uuid,
    },
    EncryptedUpdate {
        project_id: Uuid,
        envelope: EncryptedEnvelope,
        sequence: u64,
        device_id: Uuid,
    },
    EncryptedSnapshot {
        project_id: Uuid,
        envelope: EncryptedEnvelope,
        snapshot_id: Uuid,
    },
    SyncRequest {
        project_id: Uuid,
        since_sequence: u64,
    },
    SyncResponse {
        project_id: Uuid,
        updates: Vec<SyncMessage>,
        latest_snapshot: Option<Box<SyncMessage>>,
    },
    Ephemeral {
        project_id: Uuid,
        data: Vec<u8>,
    },
    Error {
        code: ErrorCode,
        message: String,
    },
    Ping,
    Pong,
}

impl SyncMessage {
    /// Serialize this message to JSON.
    pub fn to_json(&self) -> Result<String, ProtocolError> {
        serde_json::to_string(self).map_err(|e| ProtocolError::SerializationError(e.to_string()))
    }

    /// Deserialize a message from JSON.
    pub fn from_json(json: &str) -> Result<Self, ProtocolError> {
        serde_json::from_str(json).map_err(|e| ProtocolError::SerializationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_envelope() -> EncryptedEnvelope {
        EncryptedEnvelope {
            nonce: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            ciphertext: vec![0xDE, 0xAD, 0xBE, 0xEF],
            aad: vec![0xAA, 0xBB],
        }
    }

    fn fixed_uuid() -> Uuid {
        Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
    }

    fn fixed_device_id() -> Uuid {
        Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap()
    }

    fn fixed_snapshot_id() -> Uuid {
        Uuid::parse_str("7ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap()
    }

    #[test]
    fn auth_round_trip() {
        let msg = SyncMessage::Auth {
            token: "jwt-token-123".to_string(),
        };
        let json = msg.to_json().unwrap();
        let deserialized = SyncMessage::from_json(&json).unwrap();
        assert_eq!(msg, deserialized);

        // Verify the type tag is present
        assert!(json.contains(r#""type":"Auth""#));
    }

    #[test]
    fn auth_ok_round_trip() {
        let msg = SyncMessage::AuthOk {
            server_version: "1.0.0".to_string(),
        };
        let json = msg.to_json().unwrap();
        let deserialized = SyncMessage::from_json(&json).unwrap();
        assert_eq!(msg, deserialized);
        assert!(json.contains(r#""type":"AuthOk""#));
    }

    #[test]
    fn join_room_round_trip() {
        let msg = SyncMessage::JoinRoom {
            project_id: fixed_uuid(),
        };
        let json = msg.to_json().unwrap();
        let deserialized = SyncMessage::from_json(&json).unwrap();
        assert_eq!(msg, deserialized);
        assert!(json.contains(r#""type":"JoinRoom""#));
    }

    #[test]
    fn room_joined_round_trip() {
        let msg = SyncMessage::RoomJoined {
            project_id: fixed_uuid(),
            server_version: vec![1, 0, 0],
        };
        let json = msg.to_json().unwrap();
        let deserialized = SyncMessage::from_json(&json).unwrap();
        assert_eq!(msg, deserialized);
        assert!(json.contains(r#""type":"RoomJoined""#));
    }

    #[test]
    fn leave_room_round_trip() {
        let msg = SyncMessage::LeaveRoom {
            project_id: fixed_uuid(),
        };
        let json = msg.to_json().unwrap();
        let deserialized = SyncMessage::from_json(&json).unwrap();
        assert_eq!(msg, deserialized);
        assert!(json.contains(r#""type":"LeaveRoom""#));
    }

    #[test]
    fn encrypted_update_round_trip() {
        let msg = SyncMessage::EncryptedUpdate {
            project_id: fixed_uuid(),
            envelope: dummy_envelope(),
            sequence: 42,
            device_id: fixed_device_id(),
        };
        let json = msg.to_json().unwrap();
        let deserialized = SyncMessage::from_json(&json).unwrap();
        assert_eq!(msg, deserialized);
        assert!(json.contains(r#""type":"EncryptedUpdate""#));
    }

    #[test]
    fn encrypted_snapshot_round_trip() {
        let msg = SyncMessage::EncryptedSnapshot {
            project_id: fixed_uuid(),
            envelope: dummy_envelope(),
            snapshot_id: fixed_snapshot_id(),
        };
        let json = msg.to_json().unwrap();
        let deserialized = SyncMessage::from_json(&json).unwrap();
        assert_eq!(msg, deserialized);
        assert!(json.contains(r#""type":"EncryptedSnapshot""#));
    }

    #[test]
    fn sync_request_round_trip() {
        let msg = SyncMessage::SyncRequest {
            project_id: fixed_uuid(),
            since_sequence: 100,
        };
        let json = msg.to_json().unwrap();
        let deserialized = SyncMessage::from_json(&json).unwrap();
        assert_eq!(msg, deserialized);
        assert!(json.contains(r#""type":"SyncRequest""#));
    }

    #[test]
    fn sync_response_round_trip() {
        let update1 = SyncMessage::EncryptedUpdate {
            project_id: fixed_uuid(),
            envelope: dummy_envelope(),
            sequence: 101,
            device_id: fixed_device_id(),
        };
        let update2 = SyncMessage::EncryptedUpdate {
            project_id: fixed_uuid(),
            envelope: dummy_envelope(),
            sequence: 102,
            device_id: fixed_device_id(),
        };
        let snapshot = SyncMessage::EncryptedSnapshot {
            project_id: fixed_uuid(),
            envelope: dummy_envelope(),
            snapshot_id: fixed_snapshot_id(),
        };

        let msg = SyncMessage::SyncResponse {
            project_id: fixed_uuid(),
            updates: vec![update1, update2],
            latest_snapshot: Some(Box::new(snapshot)),
        };
        let json = msg.to_json().unwrap();
        let deserialized = SyncMessage::from_json(&json).unwrap();
        assert_eq!(msg, deserialized);
        assert!(json.contains(r#""type":"SyncResponse""#));
    }

    #[test]
    fn sync_response_without_snapshot_round_trip() {
        let msg = SyncMessage::SyncResponse {
            project_id: fixed_uuid(),
            updates: vec![],
            latest_snapshot: None,
        };
        let json = msg.to_json().unwrap();
        let deserialized = SyncMessage::from_json(&json).unwrap();
        assert_eq!(msg, deserialized);
    }

    #[test]
    fn ephemeral_round_trip() {
        let msg = SyncMessage::Ephemeral {
            project_id: fixed_uuid(),
            data: vec![1, 2, 3, 4, 5],
        };
        let json = msg.to_json().unwrap();
        let deserialized = SyncMessage::from_json(&json).unwrap();
        assert_eq!(msg, deserialized);
        assert!(json.contains(r#""type":"Ephemeral""#));
    }

    #[test]
    fn error_round_trip() {
        let msg = SyncMessage::Error {
            code: ErrorCode::RoomNotFound,
            message: "Project does not exist".to_string(),
        };
        let json = msg.to_json().unwrap();
        let deserialized = SyncMessage::from_json(&json).unwrap();
        assert_eq!(msg, deserialized);
        assert!(json.contains(r#""type":"Error""#));
    }

    #[test]
    fn ping_pong_round_trip() {
        let ping = SyncMessage::Ping;
        let pong = SyncMessage::Pong;

        let ping_json = ping.to_json().unwrap();
        let pong_json = pong.to_json().unwrap();

        assert_eq!(SyncMessage::from_json(&ping_json).unwrap(), ping);
        assert_eq!(SyncMessage::from_json(&pong_json).unwrap(), pong);

        assert!(ping_json.contains(r#""type":"Ping""#));
        assert!(pong_json.contains(r#""type":"Pong""#));
    }

    #[test]
    fn invalid_json_returns_error() {
        let result = SyncMessage::from_json("not valid json at all {{{");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, ProtocolError::SerializationError(_)),
            "Expected SerializationError, got: {err:?}"
        );
    }

    #[test]
    fn missing_type_field_returns_error() {
        let json = r#"{"token": "abc"}"#;
        let result = SyncMessage::from_json(json);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, ProtocolError::SerializationError(_)),
            "Expected SerializationError for missing type field, got: {err:?}"
        );
    }

    #[test]
    fn unknown_type_field_returns_error() {
        let json = r#"{"type": "UnknownVariant", "data": 123}"#;
        let result = SyncMessage::from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn encrypted_envelope_fields_preserved() {
        let envelope = EncryptedEnvelope {
            nonce: vec![0; 24],
            ciphertext: vec![0xFF; 100],
            aad: vec![0x01, 0x02],
        };
        let json = serde_json::to_string(&envelope).unwrap();
        let deserialized: EncryptedEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(envelope, deserialized);
    }
}
