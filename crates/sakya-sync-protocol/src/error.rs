use serde::{Deserialize, Serialize};

/// Standard error codes for the sync protocol.
///
/// These are serialized to snake_case for JSON wire format,
/// allowing TypeScript consumers to match on them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    Unauthorized,
    RoomNotFound,
    InvalidUpdate,
    SnapshotRequired,
    RateLimited,
    InternalError,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unauthorized => write!(f, "Unauthorized"),
            Self::RoomNotFound => write!(f, "Room not found"),
            Self::InvalidUpdate => write!(f, "Invalid update"),
            Self::SnapshotRequired => write!(f, "Snapshot required"),
            Self::RateLimited => write!(f, "Rate limited"),
            Self::InternalError => write!(f, "Internal server error"),
        }
    }
}

/// Protocol-level errors for serialization, fragmentation, and validation.
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Fragment error: {0}")]
    FragmentError(String),
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_display_unauthorized() {
        assert_eq!(ErrorCode::Unauthorized.to_string(), "Unauthorized");
    }

    #[test]
    fn error_code_display_room_not_found() {
        assert_eq!(ErrorCode::RoomNotFound.to_string(), "Room not found");
    }

    #[test]
    fn error_code_display_invalid_update() {
        assert_eq!(ErrorCode::InvalidUpdate.to_string(), "Invalid update");
    }

    #[test]
    fn error_code_display_snapshot_required() {
        assert_eq!(ErrorCode::SnapshotRequired.to_string(), "Snapshot required");
    }

    #[test]
    fn error_code_display_rate_limited() {
        assert_eq!(ErrorCode::RateLimited.to_string(), "Rate limited");
    }

    #[test]
    fn error_code_display_internal_error() {
        assert_eq!(
            ErrorCode::InternalError.to_string(),
            "Internal server error"
        );
    }

    #[test]
    fn error_code_serializes_to_snake_case() {
        let code = ErrorCode::RoomNotFound;
        let json = serde_json::to_string(&code).unwrap();
        assert_eq!(json, r#""room_not_found""#);
    }

    #[test]
    fn error_code_deserializes_from_snake_case() {
        let code: ErrorCode = serde_json::from_str(r#""snapshot_required""#).unwrap();
        assert_eq!(code, ErrorCode::SnapshotRequired);
    }

    #[test]
    fn error_code_serde_round_trip_all_variants() {
        let variants = vec![
            ErrorCode::Unauthorized,
            ErrorCode::RoomNotFound,
            ErrorCode::InvalidUpdate,
            ErrorCode::SnapshotRequired,
            ErrorCode::RateLimited,
            ErrorCode::InternalError,
        ];
        for variant in variants {
            let json = serde_json::to_string(&variant).unwrap();
            let deserialized: ErrorCode = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn protocol_error_display_serialization() {
        let err = ProtocolError::SerializationError("bad json".to_string());
        assert_eq!(err.to_string(), "Serialization error: bad json");
    }

    #[test]
    fn protocol_error_display_fragment() {
        let err = ProtocolError::FragmentError("missing fragment".to_string());
        assert_eq!(err.to_string(), "Fragment error: missing fragment");
    }

    #[test]
    fn protocol_error_display_invalid_message() {
        let err = ProtocolError::InvalidMessage("no type field".to_string());
        assert_eq!(err.to_string(), "Invalid message: no type field");
    }
}
