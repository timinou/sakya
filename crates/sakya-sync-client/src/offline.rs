//! Disk-based offline queue for encrypted sync updates.
//!
//! When the client is disconnected from the relay server, encrypted updates
//! are persisted to disk as individual JSON files. On reconnect, the queue
//! is drained in sequence order and sent to the server.
//!
//! If the queue exceeds [`SNAPSHOT_THRESHOLD`] entries, the caller should
//! create and send a full snapshot instead of replaying individual updates.

use crate::error::SyncClientError;
use sakya_sync_protocol::EncryptedEnvelope;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Maximum queue depth before triggering snapshot fallback.
const SNAPSHOT_THRESHOLD: usize = 1000;

/// Persists encrypted updates to disk when offline.
///
/// Each update is stored as a separate JSON file named `{sequence:010}.json`
/// (zero-padded to 10 digits) inside the queue directory. Lexicographic
/// ordering of filenames matches sequence ordering, so a simple sorted
/// directory listing yields updates in the correct replay order.
///
/// # Storage Layout
///
/// ```text
/// <queue_dir>/
///   0000000001.json
///   0000000002.json
///   0000000003.json
/// ```
pub struct OfflineQueue {
    queue_dir: PathBuf,
}

/// A queued update ready to be sent on reconnection.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct QueuedUpdate {
    pub project_id: Uuid,
    pub device_id: Uuid,
    pub sequence: u64,
    pub envelope: EncryptedEnvelope,
}

impl OfflineQueue {
    /// Create a new offline queue backed by the given directory.
    ///
    /// The directory is created (recursively) if it does not already exist.
    pub fn new(queue_dir: &Path) -> Result<Self, SyncClientError> {
        fs::create_dir_all(queue_dir).map_err(|e| {
            SyncClientError::OfflineQueueError(format!(
                "Failed to create queue directory {}: {}",
                queue_dir.display(),
                e
            ))
        })?;
        Ok(Self {
            queue_dir: queue_dir.to_path_buf(),
        })
    }

    /// Enqueue an update to disk.
    ///
    /// The update is serialized to JSON and written to a file named
    /// `{sequence:010}.json` inside the queue directory.
    pub fn enqueue(&self, update: &QueuedUpdate) -> Result<(), SyncClientError> {
        let filename = format!("{:010}.json", update.sequence);
        let path = self.queue_dir.join(filename);
        let json = serde_json::to_string_pretty(update).map_err(|e| {
            SyncClientError::SerializationError(format!("Failed to serialize queued update: {}", e))
        })?;
        fs::write(&path, json).map_err(|e| {
            SyncClientError::OfflineQueueError(format!(
                "Failed to write queue file {}: {}",
                path.display(),
                e
            ))
        })?;
        Ok(())
    }

    /// Get the number of queued updates.
    pub fn len(&self) -> Result<usize, SyncClientError> {
        Ok(self.list_entries()?.len())
    }

    /// Check if the queue is empty.
    pub fn is_empty(&self) -> Result<bool, SyncClientError> {
        Ok(self.list_entries()?.is_empty())
    }

    /// Check if the queue has exceeded the snapshot threshold.
    ///
    /// When this returns `true`, the caller should create a full snapshot
    /// and send that instead of draining individual updates. After the
    /// snapshot is sent, call [`clear`](Self::clear) to discard the queue.
    pub fn needs_snapshot(&self) -> Result<bool, SyncClientError> {
        Ok(self.list_entries()?.len() > SNAPSHOT_THRESHOLD)
    }

    /// Drain all queued updates in sequence order.
    ///
    /// Returns every update currently on disk, sorted by sequence number.
    /// The files are **not** removed by this method -- the caller should
    /// call [`remove`](Self::remove) or [`clear`](Self::clear) after
    /// successfully sending each update.
    pub fn drain(&self) -> Result<Vec<QueuedUpdate>, SyncClientError> {
        let entries = self.list_entries()?;
        let mut updates = Vec::with_capacity(entries.len());
        for entry in entries {
            let contents = fs::read_to_string(&entry).map_err(|e| {
                SyncClientError::OfflineQueueError(format!(
                    "Failed to read queue file {}: {}",
                    entry.display(),
                    e
                ))
            })?;
            let update: QueuedUpdate = serde_json::from_str(&contents).map_err(|e| {
                SyncClientError::SerializationError(format!(
                    "Failed to deserialize queue file {}: {}",
                    entry.display(),
                    e
                ))
            })?;
            updates.push(update);
        }
        Ok(updates)
    }

    /// Remove a single queued update by its sequence number.
    ///
    /// This is typically called after the update has been successfully
    /// sent to the server.
    pub fn remove(&self, sequence: u64) -> Result<(), SyncClientError> {
        let filename = format!("{:010}.json", sequence);
        let path = self.queue_dir.join(filename);
        if path.exists() {
            fs::remove_file(&path).map_err(|e| {
                SyncClientError::OfflineQueueError(format!(
                    "Failed to remove queue file {}: {}",
                    path.display(),
                    e
                ))
            })?;
        }
        Ok(())
    }

    /// Clear all queued updates.
    ///
    /// This is typically called after a snapshot has been sent, making
    /// the individual updates redundant.
    pub fn clear(&self) -> Result<(), SyncClientError> {
        let entries = self.list_entries()?;
        for entry in entries {
            fs::remove_file(&entry).map_err(|e| {
                SyncClientError::OfflineQueueError(format!(
                    "Failed to remove queue file {}: {}",
                    entry.display(),
                    e
                ))
            })?;
        }
        Ok(())
    }

    /// List all `.json` files in the queue directory, sorted lexicographically.
    ///
    /// Because filenames are zero-padded sequence numbers, lexicographic
    /// order equals sequence order.
    fn list_entries(&self) -> Result<Vec<PathBuf>, SyncClientError> {
        let mut entries: Vec<PathBuf> = fs::read_dir(&self.queue_dir)
            .map_err(|e| {
                SyncClientError::OfflineQueueError(format!(
                    "Failed to read queue directory {}: {}",
                    self.queue_dir.display(),
                    e
                ))
            })?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();
        entries.sort();
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sakya_sync_protocol::EncryptedEnvelope;
    use tempfile::tempdir;

    fn dummy_envelope() -> EncryptedEnvelope {
        EncryptedEnvelope {
            nonce: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            ciphertext: vec![0xDE, 0xAD, 0xBE, 0xEF],
            aad: vec![0xAA, 0xBB],
        }
    }

    fn fixed_project_id() -> Uuid {
        Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
    }

    fn fixed_device_id() -> Uuid {
        Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap()
    }

    fn make_update(sequence: u64) -> QueuedUpdate {
        QueuedUpdate {
            project_id: fixed_project_id(),
            device_id: fixed_device_id(),
            sequence,
            envelope: dummy_envelope(),
        }
    }

    #[test]
    fn enqueue_persists_to_disk() {
        let dir = tempdir().unwrap();
        let queue = OfflineQueue::new(dir.path()).unwrap();

        let update = make_update(1);
        queue.enqueue(&update).unwrap();

        let file_path = dir.path().join("0000000001.json");
        assert!(file_path.exists(), "Queue file should exist on disk");

        // Verify contents deserialize correctly
        let contents = fs::read_to_string(&file_path).unwrap();
        let deserialized: QueuedUpdate = serde_json::from_str(&contents).unwrap();
        assert_eq!(deserialized, update);
    }

    #[test]
    fn drain_returns_in_order() {
        let dir = tempdir().unwrap();
        let queue = OfflineQueue::new(dir.path()).unwrap();

        // Enqueue out of order to verify sorting
        queue.enqueue(&make_update(3)).unwrap();
        queue.enqueue(&make_update(1)).unwrap();
        queue.enqueue(&make_update(2)).unwrap();

        let updates = queue.drain().unwrap();
        assert_eq!(updates.len(), 3);
        assert_eq!(updates[0].sequence, 1);
        assert_eq!(updates[1].sequence, 2);
        assert_eq!(updates[2].sequence, 3);
    }

    #[test]
    fn snapshot_threshold() {
        let dir = tempdir().unwrap();
        let queue = OfflineQueue::new(dir.path()).unwrap();

        // At exactly SNAPSHOT_THRESHOLD, needs_snapshot should be false
        for i in 1..=SNAPSHOT_THRESHOLD as u64 {
            queue.enqueue(&make_update(i)).unwrap();
        }
        assert!(!queue.needs_snapshot().unwrap());

        // One more tips it over
        queue
            .enqueue(&make_update(SNAPSHOT_THRESHOLD as u64 + 1))
            .unwrap();
        assert!(queue.needs_snapshot().unwrap());
    }

    #[test]
    fn clear_removes_all() {
        let dir = tempdir().unwrap();
        let queue = OfflineQueue::new(dir.path()).unwrap();

        queue.enqueue(&make_update(1)).unwrap();
        queue.enqueue(&make_update(2)).unwrap();
        queue.enqueue(&make_update(3)).unwrap();
        assert_eq!(queue.len().unwrap(), 3);

        queue.clear().unwrap();
        assert_eq!(queue.len().unwrap(), 0);
        assert!(queue.is_empty().unwrap());
    }

    #[test]
    fn empty_queue_drain() {
        let dir = tempdir().unwrap();
        let queue = OfflineQueue::new(dir.path()).unwrap();

        let updates = queue.drain().unwrap();
        assert!(updates.is_empty());
        assert!(queue.is_empty().unwrap());
    }

    #[test]
    fn remove_single() {
        let dir = tempdir().unwrap();
        let queue = OfflineQueue::new(dir.path()).unwrap();

        queue.enqueue(&make_update(10)).unwrap();
        queue.enqueue(&make_update(20)).unwrap();
        queue.enqueue(&make_update(30)).unwrap();
        assert_eq!(queue.len().unwrap(), 3);

        // Remove the middle one
        queue.remove(20).unwrap();
        assert_eq!(queue.len().unwrap(), 2);

        let updates = queue.drain().unwrap();
        assert_eq!(updates.len(), 2);
        assert_eq!(updates[0].sequence, 10);
        assert_eq!(updates[1].sequence, 30);
    }

    #[test]
    fn remove_nonexistent_is_ok() {
        let dir = tempdir().unwrap();
        let queue = OfflineQueue::new(dir.path()).unwrap();

        // Removing a sequence that doesn't exist should not error
        queue.remove(999).unwrap();
    }

    #[test]
    fn new_creates_directory_recursively() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("a").join("b").join("c");
        assert!(!nested.exists());

        let _queue = OfflineQueue::new(&nested).unwrap();
        assert!(nested.exists());
    }

    #[test]
    fn non_json_files_ignored() {
        let dir = tempdir().unwrap();
        let queue = OfflineQueue::new(dir.path()).unwrap();

        queue.enqueue(&make_update(1)).unwrap();

        // Write a non-json file that should be ignored
        fs::write(dir.path().join("readme.txt"), "ignore me").unwrap();
        fs::write(dir.path().join(".lock"), "").unwrap();

        assert_eq!(queue.len().unwrap(), 1);
        let updates = queue.drain().unwrap();
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].sequence, 1);
    }
}
