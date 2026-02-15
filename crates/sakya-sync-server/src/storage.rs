//! SQLite storage for encrypted CRDT updates and snapshots.
//!
//! The server stores encrypted blobs opaquely — it never decrypts content.
//! Updates are keyed by (project_id, device_id, sequence) for deduplication.

use crate::error::ServerError;
use rusqlite::Connection;

/// A stored encrypted update.
#[derive(Debug, Clone, serde::Serialize)]
pub struct StoredUpdate {
    pub id: i64,
    pub project_id: String,
    pub device_id: String,
    pub sequence: u64,
    pub envelope_json: String,
    pub created_at: String,
}

/// A stored encrypted snapshot.
#[derive(Debug, Clone, serde::Serialize)]
pub struct StoredSnapshot {
    pub id: i64,
    pub project_id: String,
    pub snapshot_id: String,
    pub envelope_json: String,
    pub created_at: String,
}

/// SQLite-backed storage for sync updates and snapshots.
pub struct SyncStorage {
    conn: Connection,
}

impl SyncStorage {
    /// Open or create a sync database at the given path.
    pub fn new(path: &str) -> Result<Self, ServerError> {
        let conn = Connection::open(path)
            .map_err(|e| ServerError::Internal(format!("Failed to open sync DB: {e}")))?;
        let storage = Self { conn };
        storage.migrate()?;
        Ok(storage)
    }

    /// Create an in-memory sync database (for testing).
    pub fn new_in_memory() -> Result<Self, ServerError> {
        let conn = Connection::open_in_memory()
            .map_err(|e| ServerError::Internal(format!("Failed to open in-memory DB: {e}")))?;
        let storage = Self { conn };
        storage.migrate()?;
        Ok(storage)
    }

    /// Run schema migrations.
    fn migrate(&self) -> Result<(), ServerError> {
        self.conn
            .execute_batch(
                "
                CREATE TABLE IF NOT EXISTS encrypted_updates (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    project_id TEXT NOT NULL,
                    device_id TEXT NOT NULL,
                    sequence INTEGER NOT NULL,
                    envelope_json TEXT NOT NULL,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    UNIQUE(project_id, device_id, sequence)
                );
                CREATE INDEX IF NOT EXISTS idx_updates_project_seq
                    ON encrypted_updates(project_id, sequence);

                CREATE TABLE IF NOT EXISTS encrypted_snapshots (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    project_id TEXT NOT NULL,
                    snapshot_id TEXT NOT NULL UNIQUE,
                    envelope_json TEXT NOT NULL,
                    created_at TEXT NOT NULL DEFAULT (datetime('now'))
                );
                CREATE INDEX IF NOT EXISTS idx_snapshots_project
                    ON encrypted_snapshots(project_id);
                ",
            )
            .map_err(|e| ServerError::Internal(format!("Migration failed: {e}")))?;
        Ok(())
    }

    /// Store an encrypted update. Duplicates (same project+device+sequence) are ignored.
    pub fn store_update(
        &self,
        project_id: &str,
        device_id: &str,
        sequence: u64,
        envelope_json: &str,
    ) -> Result<(), ServerError> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO encrypted_updates (project_id, device_id, sequence, envelope_json)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![project_id, device_id, sequence, envelope_json],
            )
            .map_err(|e| ServerError::Internal(format!("Failed to store update: {e}")))?;
        Ok(())
    }

    /// Store an encrypted snapshot. Replaces any existing snapshot with the same snapshot_id.
    pub fn store_snapshot(
        &self,
        project_id: &str,
        snapshot_id: &str,
        envelope_json: &str,
    ) -> Result<(), ServerError> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO encrypted_snapshots (project_id, snapshot_id, envelope_json)
                 VALUES (?1, ?2, ?3)",
                rusqlite::params![project_id, snapshot_id, envelope_json],
            )
            .map_err(|e| ServerError::Internal(format!("Failed to store snapshot: {e}")))?;
        Ok(())
    }

    /// Get updates for a project since the given sequence number.
    pub fn get_updates_since(
        &self,
        project_id: &str,
        since_sequence: u64,
        limit: u32,
    ) -> Result<Vec<StoredUpdate>, ServerError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, project_id, device_id, sequence, envelope_json, created_at
                 FROM encrypted_updates
                 WHERE project_id = ?1 AND sequence > ?2
                 ORDER BY sequence ASC
                 LIMIT ?3",
            )
            .map_err(|e| ServerError::Internal(format!("Failed to prepare query: {e}")))?;

        let rows = stmt
            .query_map(
                rusqlite::params![project_id, since_sequence, limit],
                |row| {
                    Ok(StoredUpdate {
                        id: row.get(0)?,
                        project_id: row.get(1)?,
                        device_id: row.get(2)?,
                        sequence: row.get(3)?,
                        envelope_json: row.get(4)?,
                        created_at: row.get(5)?,
                    })
                },
            )
            .map_err(|e| ServerError::Internal(format!("Failed to query updates: {e}")))?;

        let mut updates = Vec::new();
        for row in rows {
            updates.push(row.map_err(|e| ServerError::Internal(format!("Row error: {e}")))?);
        }
        Ok(updates)
    }

    /// Get the latest snapshot for a project.
    pub fn get_latest_snapshot(
        &self,
        project_id: &str,
    ) -> Result<Option<StoredSnapshot>, ServerError> {
        let result = self.conn.query_row(
            "SELECT id, project_id, snapshot_id, envelope_json, created_at
             FROM encrypted_snapshots
             WHERE project_id = ?1
             ORDER BY id DESC
             LIMIT 1",
            rusqlite::params![project_id],
            |row| {
                Ok(StoredSnapshot {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    snapshot_id: row.get(2)?,
                    envelope_json: row.get(3)?,
                    created_at: row.get(4)?,
                })
            },
        );

        match result {
            Ok(snapshot) => Ok(Some(snapshot)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(ServerError::Internal(format!(
                "Failed to query snapshot: {e}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> SyncStorage {
        SyncStorage::new_in_memory().unwrap()
    }

    #[test]
    fn store_and_retrieve_update() {
        let storage = setup();
        let project_id = "proj-1";
        let device_id = "dev-1";

        storage
            .store_update(project_id, device_id, 1, r#"{"encrypted":"data1"}"#)
            .unwrap();
        storage
            .store_update(project_id, device_id, 2, r#"{"encrypted":"data2"}"#)
            .unwrap();

        let updates = storage.get_updates_since(project_id, 0, 100).unwrap();
        assert_eq!(updates.len(), 2);
        assert_eq!(updates[0].sequence, 1);
        assert_eq!(updates[1].sequence, 2);
    }

    #[test]
    fn get_updates_since_filters_correctly() {
        let storage = setup();
        let project_id = "proj-1";

        for seq in 1..=5 {
            storage
                .store_update(project_id, "dev-1", seq, &format!("data-{seq}"))
                .unwrap();
        }

        let updates = storage.get_updates_since(project_id, 3, 100).unwrap();
        assert_eq!(updates.len(), 2);
        assert_eq!(updates[0].sequence, 4);
        assert_eq!(updates[1].sequence, 5);
    }

    #[test]
    fn duplicate_update_ignored() {
        let storage = setup();

        storage
            .store_update("proj-1", "dev-1", 1, "original")
            .unwrap();
        // Same project+device+sequence → INSERT OR IGNORE
        storage
            .store_update("proj-1", "dev-1", 1, "duplicate")
            .unwrap();

        let updates = storage.get_updates_since("proj-1", 0, 100).unwrap();
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].envelope_json, "original");
    }

    #[test]
    fn store_and_retrieve_snapshot() {
        let storage = setup();

        storage
            .store_snapshot("proj-1", "snap-1", r#"{"snapshot":"data1"}"#)
            .unwrap();

        let snap = storage.get_latest_snapshot("proj-1").unwrap();
        assert!(snap.is_some());
        let snap = snap.unwrap();
        assert_eq!(snap.snapshot_id, "snap-1");
        assert_eq!(snap.envelope_json, r#"{"snapshot":"data1"}"#);
    }

    #[test]
    fn latest_snapshot_returns_most_recent() {
        let storage = setup();

        storage.store_snapshot("proj-1", "snap-1", "old").unwrap();
        storage.store_snapshot("proj-1", "snap-2", "new").unwrap();

        let snap = storage.get_latest_snapshot("proj-1").unwrap().unwrap();
        assert_eq!(snap.snapshot_id, "snap-2");
        assert_eq!(snap.envelope_json, "new");
    }

    #[test]
    fn no_snapshot_returns_none() {
        let storage = setup();
        let snap = storage.get_latest_snapshot("nonexistent").unwrap();
        assert!(snap.is_none());
    }

    #[test]
    fn updates_ordered_by_sequence() {
        let storage = setup();

        // Insert out of order
        storage.store_update("proj-1", "dev-1", 3, "third").unwrap();
        storage.store_update("proj-1", "dev-1", 1, "first").unwrap();
        storage
            .store_update("proj-1", "dev-1", 2, "second")
            .unwrap();

        let updates = storage.get_updates_since("proj-1", 0, 100).unwrap();
        assert_eq!(updates[0].sequence, 1);
        assert_eq!(updates[1].sequence, 2);
        assert_eq!(updates[2].sequence, 3);
    }

    #[test]
    fn get_updates_respects_limit() {
        let storage = setup();

        for seq in 1..=10 {
            storage
                .store_update("proj-1", "dev-1", seq, &format!("data-{seq}"))
                .unwrap();
        }

        let updates = storage.get_updates_since("proj-1", 0, 3).unwrap();
        assert_eq!(updates.len(), 3);
    }
}
