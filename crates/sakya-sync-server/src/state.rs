//! Application state shared across all handlers.

use crate::room::RoomManager;
use crate::storage::SyncStorage;
use sakya_auth::{AuthDb, DeviceService, JwtService, MagicLinkService};
use std::sync::{Arc, Mutex};

/// Shared application state passed to all route handlers.
#[derive(Clone)]
pub struct AppState {
    /// JWT token validation (stateless, no DB access needed).
    pub jwt_service: Arc<JwtService>,
    /// Magic link creation/verification (owns its own AuthDb).
    pub magic_link_service: Arc<Mutex<MagicLinkService>>,
    /// Device registration/listing (owns its own AuthDb).
    pub device_service: Arc<Mutex<DeviceService>>,
    /// WebSocket room broadcasting.
    pub room_manager: Arc<RoomManager>,
    /// Encrypted update/snapshot persistence.
    pub storage: Arc<Mutex<SyncStorage>>,
}

impl AppState {
    /// Create a new AppState with the given configuration.
    ///
    /// Uses separate AuthDb instances for MagicLinkService and DeviceService
    /// since rusqlite::Connection is not Sync.
    pub fn new(
        jwt_secret: &str,
        auth_db_path: &str,
        sync_db_path: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let jwt_service = Arc::new(JwtService::new(jwt_secret));

        // Separate AuthDb per service (SQLite supports concurrent readers)
        let magic_link_db = AuthDb::new(auth_db_path)?;
        let device_db = AuthDb::new(auth_db_path)?;

        let magic_link_service = Arc::new(Mutex::new(MagicLinkService::new(magic_link_db)));
        let device_service = Arc::new(Mutex::new(DeviceService::new(device_db)));

        let room_manager = Arc::new(RoomManager::new());
        let storage = Arc::new(Mutex::new(SyncStorage::new(sync_db_path)?));

        Ok(Self {
            jwt_service,
            magic_link_service,
            device_service,
            room_manager,
            storage,
        })
    }
}

/// Test utilities (available in test builds and when test-utils feature is enabled).
#[cfg(any(test, feature = "test-utils"))]
impl AppState {
    /// Create an AppState backed by temp files (for testing).
    ///
    /// Uses temp files rather than in-memory DBs so that both the
    /// MagicLinkService and DeviceService share the same auth database
    /// (they need to see each other's accounts for FK constraints).
    pub fn new_test(jwt_secret: &str) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let auth_path = dir.path().join("test-auth.db");
        let sync_path = dir.path().join("test-sync.db");

        let jwt_service = Arc::new(JwtService::new(jwt_secret));

        let magic_link_db = AuthDb::new(auth_path.to_str().unwrap()).unwrap();
        let device_db = AuthDb::new(auth_path.to_str().unwrap()).unwrap();

        let magic_link_service = Arc::new(Mutex::new(MagicLinkService::new(magic_link_db)));
        let device_service = Arc::new(Mutex::new(DeviceService::new(device_db)));

        let room_manager = Arc::new(RoomManager::new());
        let storage = Arc::new(Mutex::new(
            SyncStorage::new(sync_path.to_str().unwrap()).unwrap(),
        ));

        // Leak the tempdir so it persists for the lifetime of tests
        std::mem::forget(dir);

        Self {
            jwt_service,
            magic_link_service,
            device_service,
            room_manager,
            storage,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_test_state() {
        let _state = AppState::new_test("test-secret");
    }

    #[test]
    fn create_file_based_state() {
        let dir = tempfile::tempdir().unwrap();
        let auth_path = dir.path().join("auth.db");
        let sync_path = dir.path().join("sync.db");
        let state = AppState::new(
            "test-secret",
            auth_path.to_str().unwrap(),
            sync_path.to_str().unwrap(),
        );
        assert!(state.is_ok(), "should create file-based app state");
    }
}
