//! Client-side sync engine for Sakya.
//!
//! Manages WebSocket connections, offline queuing,
//! and CRDT state synchronization with the relay server.

pub mod engine;
pub mod error;
pub mod offline;
pub mod reconnect;
pub mod sync_flow;

pub use engine::{EngineCommand, SyncEngine, SyncEvent, SyncStatus};
pub use error::SyncClientError;
pub use offline::OfflineQueue;
