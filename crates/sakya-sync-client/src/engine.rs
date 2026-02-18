//! Core sync engine managing WebSocket connection and project sessions.
//!
//! The [`SyncEngine`] maintains a persistent WebSocket connection to the sync server,
//! handles authentication, room management, and message routing.

use crate::error::SyncClientError;
use crate::reconnect::ReconnectPolicy;
use futures_util::{SinkExt, StreamExt};
use sakya_crypto::Encryptor;
use sakya_sync_protocol::SyncMessage;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use uuid::Uuid;

/// Connection status for the sync engine.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "status")]
pub enum SyncStatus {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting { attempt: u32 },
    Error { message: String },
}

/// Commands that can be sent to the background engine task.
#[derive(Debug)]
pub enum EngineCommand {
    /// Send a raw SyncMessage over the WebSocket.
    SendMessage(SyncMessage),
    /// Enable sync for a project — join room + set up encryption.
    EnableProject {
        project_id: Uuid,
        doc_key: [u8; 32],
    },
    /// Disable sync for a project — leave room.
    DisableProject {
        project_id: Uuid,
    },
    /// Graceful shutdown.
    Shutdown,
}

/// Incoming events from the sync engine to the application layer.
#[derive(Debug, Clone)]
pub enum SyncEvent {
    /// Status changed.
    StatusChanged(SyncStatus),
    /// Decrypted CRDT update received from a remote peer.
    UpdateReceived {
        project_id: Uuid,
        update_bytes: Vec<u8>,
    },
    /// A project room was joined successfully.
    ProjectJoined { project_id: Uuid },
    /// Error related to a specific project.
    ProjectError {
        project_id: Uuid,
        message: String,
    },
}

/// Per-project sync session state.
struct ProjectSession {
    #[allow(dead_code)]
    project_id: Uuid,
    encryptor: sakya_crypto::XChaCha20Encryptor,
    local_sequence: u64,
}

/// The sync engine manages WebSocket connectivity and sync state.
pub struct SyncEngine {
    status: Arc<Mutex<SyncStatus>>,
    cmd_tx: mpsc::Sender<EngineCommand>,
    event_tx: broadcast::Sender<SyncEvent>,
    task_handle: tokio::task::JoinHandle<()>,
}

impl SyncEngine {
    /// Connect to the sync server and authenticate.
    ///
    /// Returns the engine handle. The connection runs in a background task.
    pub async fn connect(
        server_url: String,
        jwt_token: String,
        device_id: Uuid,
    ) -> Result<Self, SyncClientError> {
        let (cmd_tx, cmd_rx) = mpsc::channel(256);
        let (event_tx, _) = broadcast::channel(256);
        let status = Arc::new(Mutex::new(SyncStatus::Connecting));

        let task_status = status.clone();
        let task_event_tx = event_tx.clone();

        let task_handle = tokio::spawn(async move {
            engine_loop(
                server_url,
                jwt_token,
                device_id,
                cmd_rx,
                task_event_tx,
                task_status,
            )
            .await;
        });

        Ok(Self {
            status,
            cmd_tx,
            event_tx,
            task_handle,
        })
    }

    /// Get the current sync status.
    pub async fn status(&self) -> SyncStatus {
        self.status.lock().await.clone()
    }

    /// Subscribe to sync events.
    pub fn subscribe(&self) -> broadcast::Receiver<SyncEvent> {
        self.event_tx.subscribe()
    }

    /// Enable sync for a project with the given document encryption key.
    pub async fn enable_project(
        &self,
        project_id: Uuid,
        doc_key: [u8; 32],
    ) -> Result<(), SyncClientError> {
        self.cmd_tx
            .send(EngineCommand::EnableProject {
                project_id,
                doc_key,
            })
            .await
            .map_err(|_| SyncClientError::EngineStopped)
    }

    /// Disable sync for a project (leaves the room).
    pub async fn disable_project(&self, project_id: Uuid) -> Result<(), SyncClientError> {
        self.cmd_tx
            .send(EngineCommand::DisableProject { project_id })
            .await
            .map_err(|_| SyncClientError::EngineStopped)
    }

    /// Send a CRDT update for a project. The engine encrypts and sends it.
    pub async fn send_update(
        &self,
        project_id: Uuid,
        update_bytes: Vec<u8>,
    ) -> Result<(), SyncClientError> {
        // We send a special SendMessage that the engine loop will encrypt
        // before transmission. Use EncryptedUpdate with a dummy envelope —
        // the engine loop handles actual encryption from the project session.
        self.cmd_tx
            .send(EngineCommand::SendMessage(SyncMessage::EncryptedUpdate {
                project_id,
                envelope: sakya_sync_protocol::EncryptedEnvelope {
                    // Placeholder — the engine loop replaces this with the real encrypted data
                    nonce: update_bytes,
                    ciphertext: vec![],
                    aad: vec![],
                },
                sequence: 0, // Engine assigns the real sequence
                device_id: Uuid::nil(),
            }))
            .await
            .map_err(|_| SyncClientError::EngineStopped)
    }

    /// Send a raw SyncMessage (for advanced use).
    pub async fn send_message(&self, msg: SyncMessage) -> Result<(), SyncClientError> {
        self.cmd_tx
            .send(EngineCommand::SendMessage(msg))
            .await
            .map_err(|_| SyncClientError::EngineStopped)
    }

    /// Gracefully disconnect.
    pub async fn disconnect(self) -> Result<(), SyncClientError> {
        let _ = self.cmd_tx.send(EngineCommand::Shutdown).await;
        let _ = self.task_handle.await;
        Ok(())
    }
}

/// Set the status and emit event.
async fn set_status(
    status: &Mutex<SyncStatus>,
    event_tx: &broadcast::Sender<SyncEvent>,
    new_status: SyncStatus,
) {
    let mut s = status.lock().await;
    *s = new_status.clone();
    let _ = event_tx.send(SyncEvent::StatusChanged(new_status));
}

/// The main engine loop: connect, authenticate, run message loop, reconnect on failure.
async fn engine_loop(
    server_url: String,
    jwt_token: String,
    device_id: Uuid,
    mut cmd_rx: mpsc::Receiver<EngineCommand>,
    event_tx: broadcast::Sender<SyncEvent>,
    status: Arc<Mutex<SyncStatus>>,
) {
    let mut reconnect = ReconnectPolicy::new();
    let mut sessions: HashMap<Uuid, ProjectSession> = HashMap::new();
    // Track projects that should be re-joined on reconnect
    let mut active_projects: HashMap<Uuid, [u8; 32]> = HashMap::new();

    'outer: loop {
        set_status(&status, &event_tx, SyncStatus::Connecting).await;

        // Attempt WebSocket connection
        let ws_url = format!("{}/sync", server_url.trim_end_matches('/'));
        let ws_result = tokio_tungstenite::connect_async(&ws_url).await;

        let ws_stream = match ws_result {
            Ok((stream, _)) => stream,
            Err(e) => {
                tracing::warn!("WebSocket connection failed: {e}");
                let attempt = reconnect.attempt_count();
                set_status(
                    &status,
                    &event_tx,
                    SyncStatus::Reconnecting { attempt },
                )
                .await;
                let delay = reconnect.next_delay();

                // Wait for reconnect delay, but check for shutdown commands
                tokio::select! {
                    _ = tokio::time::sleep(delay) => continue 'outer,
                    cmd = cmd_rx.recv() => {
                        match cmd {
                            Some(EngineCommand::Shutdown) | None => break 'outer,
                            Some(EngineCommand::EnableProject { project_id, doc_key }) => {
                                active_projects.insert(project_id, doc_key);
                                continue 'outer;
                            }
                            Some(EngineCommand::DisableProject { project_id }) => {
                                active_projects.remove(&project_id);
                                sessions.remove(&project_id);
                                continue 'outer;
                            }
                            _ => continue 'outer,
                        }
                    }
                }
            }
        };

        let (mut ws_write, mut ws_read) = ws_stream.split();

        // Authenticate
        let auth_msg = SyncMessage::Auth {
            token: jwt_token.clone(),
        };
        let auth_json = match auth_msg.to_json() {
            Ok(j) => j,
            Err(_) => continue 'outer,
        };
        if ws_write
            .send(WsMessage::Text(auth_json.into()))
            .await
            .is_err()
        {
            continue 'outer;
        }

        // Wait for AuthOk
        let auth_response = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            ws_read.next(),
        )
        .await;

        match auth_response {
            Ok(Some(Ok(WsMessage::Text(text)))) => {
                match SyncMessage::from_json(&text) {
                    Ok(SyncMessage::AuthOk { .. }) => {
                        // Authentication succeeded
                        reconnect.reset();
                        set_status(&status, &event_tx, SyncStatus::Connected).await;
                    }
                    Ok(SyncMessage::Error { message, .. }) => {
                        set_status(
                            &status,
                            &event_tx,
                            SyncStatus::Error {
                                message: format!("Auth failed: {message}"),
                            },
                        )
                        .await;
                        break 'outer;
                    }
                    _ => continue 'outer,
                }
            }
            _ => continue 'outer,
        }

        // Re-join rooms for active projects
        for (&project_id, &doc_key) in &active_projects {
            let join_msg = SyncMessage::JoinRoom { project_id };
            if let Ok(json) = join_msg.to_json() {
                let _ = ws_write.send(WsMessage::Text(json.into())).await;
            }
            sessions.entry(project_id).or_insert_with(|| ProjectSession {
                project_id,
                encryptor: sakya_crypto::XChaCha20Encryptor::new(doc_key),
                local_sequence: 0,
            });
        }

        // Main message loop
        let mut heartbeat = tokio::time::interval(std::time::Duration::from_secs(25));

        loop {
            tokio::select! {
                // Incoming WebSocket message
                msg = ws_read.next() => {
                    match msg {
                        Some(Ok(WsMessage::Text(text))) => {
                            handle_incoming_message(
                                &text,
                                &sessions,
                                &event_tx,
                            ).await;
                        }
                        Some(Ok(WsMessage::Close(_))) | None => {
                            // Connection closed, trigger reconnect
                            tracing::info!("WebSocket connection closed");
                            break;
                        }
                        Some(Err(e)) => {
                            tracing::warn!("WebSocket error: {e}");
                            break;
                        }
                        _ => {} // Ping/Pong/Binary
                    }
                }

                // Outgoing commands
                cmd = cmd_rx.recv() => {
                    match cmd {
                        Some(EngineCommand::Shutdown) | None => {
                            let _ = ws_write.close().await;
                            break 'outer;
                        }
                        Some(EngineCommand::SendMessage(msg)) => {
                            if let Some(json) = prepare_outgoing_message(
                                msg,
                                device_id,
                                &mut sessions,
                            ) {
                                if ws_write.send(WsMessage::Text(json.into())).await.is_err() {
                                    break; // Connection lost, reconnect
                                }
                            }
                        }
                        Some(EngineCommand::EnableProject { project_id, doc_key }) => {
                            active_projects.insert(project_id, doc_key);
                            sessions.entry(project_id).or_insert_with(|| ProjectSession {
                                project_id,
                                encryptor: sakya_crypto::XChaCha20Encryptor::new(doc_key),
                                local_sequence: 0,
                            });
                            let join = SyncMessage::JoinRoom { project_id };
                            if let Ok(json) = join.to_json() {
                                if ws_write.send(WsMessage::Text(json.into())).await.is_err() {
                                    break;
                                }
                            }
                        }
                        Some(EngineCommand::DisableProject { project_id }) => {
                            active_projects.remove(&project_id);
                            sessions.remove(&project_id);
                            let leave = SyncMessage::LeaveRoom { project_id };
                            if let Ok(json) = leave.to_json() {
                                let _ = ws_write.send(WsMessage::Text(json.into())).await;
                            }
                        }
                    }
                }

                // Heartbeat
                _ = heartbeat.tick() => {
                    let ping = SyncMessage::Ping;
                    if let Ok(json) = ping.to_json() {
                        if ws_write.send(WsMessage::Text(json.into())).await.is_err() {
                            break; // Connection lost
                        }
                    }
                }
            }
        }

        // Connection lost — loop back to reconnect
        let attempt = reconnect.attempt_count();
        set_status(
            &status,
            &event_tx,
            SyncStatus::Reconnecting { attempt },
        )
        .await;
        let delay = reconnect.next_delay();

        tokio::select! {
            _ = tokio::time::sleep(delay) => {},
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(EngineCommand::Shutdown) | None => break 'outer,
                    Some(EngineCommand::EnableProject { project_id, doc_key }) => {
                        active_projects.insert(project_id, doc_key);
                    }
                    Some(EngineCommand::DisableProject { project_id }) => {
                        active_projects.remove(&project_id);
                        sessions.remove(&project_id);
                    }
                    _ => {}
                }
            }
        }
    }

    set_status(&status, &event_tx, SyncStatus::Disconnected).await;
}

/// Handle an incoming WS message: decrypt and emit events.
async fn handle_incoming_message(
    text: &str,
    sessions: &HashMap<Uuid, ProjectSession>,
    event_tx: &broadcast::Sender<SyncEvent>,
) {
    let msg = match SyncMessage::from_json(text) {
        Ok(m) => m,
        Err(_) => return,
    };

    match msg {
        SyncMessage::RoomJoined { project_id, .. } => {
            let _ = event_tx.send(SyncEvent::ProjectJoined { project_id });
        }
        SyncMessage::EncryptedUpdate {
            project_id,
            envelope,
            ..
        } => {
            if let Some(session) = sessions.get(&project_id) {
                // Convert protocol envelope to crypto envelope
                let crypto_envelope = sakya_crypto::EncryptedEnvelope {
                    nonce: envelope
                        .nonce
                        .try_into()
                        .unwrap_or([0u8; 24]),
                    ciphertext: envelope.ciphertext,
                    aad: envelope.aad,
                };
                match session.encryptor.decrypt(&crypto_envelope) {
                    Ok(plaintext) => {
                        let _ = event_tx.send(SyncEvent::UpdateReceived {
                            project_id,
                            update_bytes: plaintext,
                        });
                    }
                    Err(e) => {
                        tracing::warn!(
                            project_id = %project_id,
                            "Failed to decrypt update: {e}"
                        );
                    }
                }
            }
        }
        SyncMessage::SyncResponse {
            project_id,
            updates,
            latest_snapshot,
        } => {
            if let Some(session) = sessions.get(&project_id) {
                // Process snapshot first if available
                if let Some(snapshot_msg) = latest_snapshot {
                    if let SyncMessage::EncryptedSnapshot { envelope, .. } = *snapshot_msg {
                        let crypto_envelope = sakya_crypto::EncryptedEnvelope {
                            nonce: envelope
                                .nonce
                                .try_into()
                                .unwrap_or([0u8; 24]),
                            ciphertext: envelope.ciphertext,
                            aad: envelope.aad,
                        };
                        if let Ok(plaintext) = session.encryptor.decrypt(&crypto_envelope) {
                            let _ = event_tx.send(SyncEvent::UpdateReceived {
                                project_id,
                                update_bytes: plaintext,
                            });
                        }
                    }
                }
                // Then process individual updates
                for update in updates {
                    if let SyncMessage::EncryptedUpdate { envelope, .. } = update {
                        let crypto_envelope = sakya_crypto::EncryptedEnvelope {
                            nonce: envelope
                                .nonce
                                .try_into()
                                .unwrap_or([0u8; 24]),
                            ciphertext: envelope.ciphertext,
                            aad: envelope.aad,
                        };
                        if let Ok(plaintext) = session.encryptor.decrypt(&crypto_envelope) {
                            let _ = event_tx.send(SyncEvent::UpdateReceived {
                                project_id,
                                update_bytes: plaintext,
                            });
                        }
                    }
                }
            }
        }
        SyncMessage::Error { code, message } => {
            tracing::warn!("Server error: {code:?} — {message}");
        }
        SyncMessage::Pong => {} // Expected heartbeat response
        _ => {}
    }
}

/// Prepare an outgoing message: encrypt if it's an update.
fn prepare_outgoing_message(
    msg: SyncMessage,
    device_id: Uuid,
    sessions: &mut HashMap<Uuid, ProjectSession>,
) -> Option<String> {
    match msg {
        SyncMessage::EncryptedUpdate {
            project_id,
            envelope,
            ..
        } => {
            // The `nonce` field holds the raw plaintext bytes to encrypt
            let plaintext = envelope.nonce;
            let session = sessions.get_mut(&project_id)?;
            session.local_sequence += 1;

            use sakya_crypto::Encryptor;
            let crypto_envelope = session
                .encryptor
                .encrypt(&plaintext, project_id.as_bytes())
                .ok()?;

            let protocol_envelope = sakya_sync_protocol::EncryptedEnvelope {
                nonce: crypto_envelope.nonce.to_vec(),
                ciphertext: crypto_envelope.ciphertext,
                aad: crypto_envelope.aad,
            };

            let wire_msg = SyncMessage::EncryptedUpdate {
                project_id,
                envelope: protocol_envelope,
                sequence: session.local_sequence,
                device_id,
            };
            wire_msg.to_json().ok()
        }
        other => other.to_json().ok(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn sync_status_serialization() {
        let status = SyncStatus::Connected;
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("Connected"));

        let status = SyncStatus::Reconnecting { attempt: 3 };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("Reconnecting"));
        assert!(json.contains("3"));
    }

    #[test]
    fn sync_status_variants() {
        let variants = vec![
            SyncStatus::Disconnected,
            SyncStatus::Connecting,
            SyncStatus::Connected,
            SyncStatus::Reconnecting { attempt: 1 },
            SyncStatus::Error {
                message: "test".to_string(),
            },
        ];
        for v in &variants {
            let json = serde_json::to_string(v).unwrap();
            let parsed: SyncStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(&parsed, v);
        }
    }

    #[test]
    fn prepare_outgoing_non_update() {
        let mut sessions = HashMap::new();
        let device_id = Uuid::new_v4();

        let msg = SyncMessage::Ping;
        let json = prepare_outgoing_message(msg, device_id, &mut sessions);
        assert!(json.is_some());
        assert!(json.unwrap().contains("Ping"));
    }

    #[test]
    fn prepare_outgoing_update_encrypts() {
        let mut sessions = HashMap::new();
        let device_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();
        let key = [42u8; 32];

        sessions.insert(
            project_id,
            ProjectSession {
                project_id,
                encryptor: sakya_crypto::XChaCha20Encryptor::new(key),
                local_sequence: 0,
            },
        );

        let plaintext = b"hello crdt update".to_vec();
        let msg = SyncMessage::EncryptedUpdate {
            project_id,
            envelope: sakya_sync_protocol::EncryptedEnvelope {
                nonce: plaintext.clone(),
                ciphertext: vec![],
                aad: vec![],
            },
            sequence: 0,
            device_id: Uuid::nil(),
        };

        let json = prepare_outgoing_message(msg, device_id, &mut sessions);
        assert!(json.is_some());
        let json = json.unwrap();

        // Verify it's an EncryptedUpdate with a real envelope
        let parsed = SyncMessage::from_json(&json).unwrap();
        match parsed {
            SyncMessage::EncryptedUpdate {
                sequence,
                device_id: did,
                envelope,
                ..
            } => {
                assert_eq!(sequence, 1);
                assert_eq!(did, device_id);
                // Nonce should be 24 bytes (base64 encoded in JSON)
                assert_eq!(envelope.nonce.len(), 24);
                // Ciphertext should be non-empty
                assert!(!envelope.ciphertext.is_empty());
            }
            _ => panic!("Expected EncryptedUpdate"),
        }
    }

    #[test]
    fn prepare_outgoing_update_no_session() {
        let mut sessions = HashMap::new();
        let device_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();

        let msg = SyncMessage::EncryptedUpdate {
            project_id,
            envelope: sakya_sync_protocol::EncryptedEnvelope {
                nonce: vec![1, 2, 3],
                ciphertext: vec![],
                aad: vec![],
            },
            sequence: 0,
            device_id: Uuid::nil(),
        };

        let json = prepare_outgoing_message(msg, device_id, &mut sessions);
        assert!(json.is_none(), "Should return None when no session exists");
    }

    #[test]
    fn sequence_increments() {
        let mut sessions = HashMap::new();
        let device_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();
        let key = [42u8; 32];

        sessions.insert(
            project_id,
            ProjectSession {
                project_id,
                encryptor: sakya_crypto::XChaCha20Encryptor::new(key),
                local_sequence: 0,
            },
        );

        for expected_seq in 1..=3 {
            let msg = SyncMessage::EncryptedUpdate {
                project_id,
                envelope: sakya_sync_protocol::EncryptedEnvelope {
                    nonce: vec![1],
                    ciphertext: vec![],
                    aad: vec![],
                },
                sequence: 0,
                device_id: Uuid::nil(),
            };
            let json = prepare_outgoing_message(msg, device_id, &mut sessions).unwrap();
            let parsed = SyncMessage::from_json(&json).unwrap();
            match parsed {
                SyncMessage::EncryptedUpdate { sequence, .. } => {
                    assert_eq!(sequence, expected_seq);
                }
                _ => panic!("Expected EncryptedUpdate"),
            }
        }
    }

    #[tokio::test]
    async fn engine_connect_to_nonexistent_server_enters_reconnecting() {
        let engine = SyncEngine::connect(
            "ws://127.0.0.1:1".to_string(), // Port 1 won't have a server
            "fake-jwt".to_string(),
            Uuid::new_v4(),
        )
        .await
        .unwrap();

        // Give the engine a moment to attempt connection and fail
        tokio::time::sleep(Duration::from_millis(200)).await;

        let status = engine.status().await;
        match status {
            SyncStatus::Reconnecting { .. } | SyncStatus::Connecting => {}
            other => panic!("Expected Reconnecting or Connecting, got: {other:?}"),
        }

        engine.disconnect().await.unwrap();
    }
}
