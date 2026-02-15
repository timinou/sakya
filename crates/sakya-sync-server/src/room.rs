//! Room manager for project-based WebSocket broadcasting.
//!
//! Each project has a "room" backed by a tokio broadcast channel.
//! Clients join/leave rooms, and messages are broadcast to all
//! room members except the sender.

use dashmap::DashMap;
use tokio::sync::broadcast;
use uuid::Uuid;

/// A message broadcast within a room.
#[derive(Debug, Clone)]
pub struct BroadcastMsg {
    /// The connection that sent this message (used to skip echo).
    pub sender_conn_id: Uuid,
    /// The JSON-encoded SyncMessage.
    pub json: String,
}

/// Manages project rooms with lock-free concurrent access.
pub struct RoomManager {
    rooms: DashMap<Uuid, broadcast::Sender<BroadcastMsg>>,
}

impl RoomManager {
    /// Create a new empty room manager.
    pub fn new() -> Self {
        Self {
            rooms: DashMap::new(),
        }
    }

    /// Join a room for the given project, creating it if it doesn't exist.
    ///
    /// Returns a broadcast receiver to listen for messages.
    pub fn join(&self, project_id: Uuid) -> broadcast::Receiver<BroadcastMsg> {
        let entry = self.rooms.entry(project_id).or_insert_with(|| {
            let (tx, _) = broadcast::channel(256);
            tx
        });
        entry.value().subscribe()
    }

    /// Leave a room. If the room is empty after this, it will be cleaned up
    /// on the next `cleanup_empty_rooms` call.
    pub fn leave(&self, project_id: Uuid) {
        // Just unsubscribe by dropping the receiver.
        // Cleanup is handled separately.
        let _ = project_id;
    }

    /// Broadcast a message to all members of a room.
    ///
    /// Returns the number of receivers that received the message,
    /// or 0 if the room doesn't exist.
    pub fn broadcast(&self, project_id: Uuid, msg: BroadcastMsg) -> usize {
        if let Some(tx) = self.rooms.get(&project_id) {
            tx.send(msg).unwrap_or(0)
        } else {
            0
        }
    }

    /// Remove rooms with no active subscribers.
    pub fn cleanup_empty_rooms(&self) {
        self.rooms.retain(|_, tx| tx.receiver_count() > 0);
    }

    /// Get the number of active rooms.
    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    /// Check if a room exists for the given project.
    pub fn has_room(&self, project_id: Uuid) -> bool {
        self.rooms.contains_key(&project_id)
    }
}

impl Default for RoomManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn join_creates_room() {
        let mgr = RoomManager::new();
        let project_id = Uuid::new_v4();

        assert!(!mgr.has_room(project_id));
        let _rx = mgr.join(project_id);
        assert!(mgr.has_room(project_id));
    }

    #[tokio::test]
    async fn broadcast_reaches_subscriber() {
        let mgr = RoomManager::new();
        let project_id = Uuid::new_v4();
        let sender_id = Uuid::new_v4();

        let mut rx = mgr.join(project_id);

        let msg = BroadcastMsg {
            sender_conn_id: sender_id,
            json: r#"{"type":"Ping"}"#.to_string(),
        };

        let count = mgr.broadcast(project_id, msg.clone());
        assert_eq!(count, 1);

        let received = rx.recv().await.unwrap();
        assert_eq!(received.sender_conn_id, sender_id);
        assert_eq!(received.json, r#"{"type":"Ping"}"#);
    }

    #[tokio::test]
    async fn broadcast_to_nonexistent_room_returns_zero() {
        let mgr = RoomManager::new();
        let msg = BroadcastMsg {
            sender_conn_id: Uuid::new_v4(),
            json: "test".to_string(),
        };
        let count = mgr.broadcast(Uuid::new_v4(), msg);
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn multiple_subscribers_receive_broadcast() {
        let mgr = RoomManager::new();
        let project_id = Uuid::new_v4();
        let sender_id = Uuid::new_v4();

        let mut rx1 = mgr.join(project_id);
        let mut rx2 = mgr.join(project_id);

        let msg = BroadcastMsg {
            sender_conn_id: sender_id,
            json: "hello".to_string(),
        };

        let count = mgr.broadcast(project_id, msg);
        assert_eq!(count, 2);

        let m1 = rx1.recv().await.unwrap();
        let m2 = rx2.recv().await.unwrap();
        assert_eq!(m1.json, "hello");
        assert_eq!(m2.json, "hello");
    }

    #[tokio::test]
    async fn cleanup_removes_empty_rooms() {
        let mgr = RoomManager::new();
        let project_id = Uuid::new_v4();

        {
            let _rx = mgr.join(project_id);
            assert_eq!(mgr.room_count(), 1);
        }
        // rx is dropped, no more subscribers

        mgr.cleanup_empty_rooms();
        assert_eq!(mgr.room_count(), 0);
    }

    #[tokio::test]
    async fn cleanup_keeps_active_rooms() {
        let mgr = RoomManager::new();
        let p1 = Uuid::new_v4();
        let p2 = Uuid::new_v4();

        let _rx1 = mgr.join(p1);
        {
            let _rx2 = mgr.join(p2);
        }
        // p2's subscriber is dropped

        mgr.cleanup_empty_rooms();
        assert_eq!(mgr.room_count(), 1);
        assert!(mgr.has_room(p1));
        assert!(!mgr.has_room(p2));
    }
}
