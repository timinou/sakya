//! Wire protocol for Sakya sync.
//!
//! Defines [`SyncMessage`] enum, fragmentation/reassembly,
//! and error codes for client-server communication.
//!
//! # Architecture
//!
//! The protocol uses JSON-encoded messages over WebSocket.
//! Messages are internally tagged with a `"type"` field for
//! easy consumption by both Rust and TypeScript.
//!
//! Large messages (>256 KiB) are split into [`Fragment`]s
//! and reassembled by the [`Reassembler`].
//!
//! # Example
//!
//! ```
//! use sakya_sync_protocol::{SyncMessage, ErrorCode};
//!
//! let msg = SyncMessage::Error {
//!     code: ErrorCode::Unauthorized,
//!     message: "Invalid token".to_string(),
//! };
//!
//! let json = msg.to_json().unwrap();
//! let round_tripped = SyncMessage::from_json(&json).unwrap();
//! assert_eq!(msg, round_tripped);
//! ```

pub mod error;
pub mod fragment;
pub mod message;

pub use error::{ErrorCode, ProtocolError};
pub use fragment::{Fragment, Fragmenter, Reassembler};
pub use message::{EncryptedEnvelope, SyncMessage};
