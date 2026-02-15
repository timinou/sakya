//! WebSocket upgrade, authentication, and message handling.

use crate::room::BroadcastMsg;
use crate::state::AppState;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use sakya_sync_protocol::{ErrorCode, SyncMessage};
use std::collections::HashSet;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::Instant;
use uuid::Uuid;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(10);
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// GET /sync — WebSocket upgrade handler.
pub async fn ws_upgrade(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle an individual WebSocket connection.
async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let conn_id = Uuid::new_v4();

    // Phase 1: Authentication — first message must be Auth { token }
    let (account_id, device_id) = match authenticate(&mut socket, &state).await {
        Some(ids) => ids,
        None => return, // Auth failed, connection closed
    };

    tracing::info!(
        conn_id = %conn_id,
        account_id = %account_id,
        device_id = %device_id,
        "WebSocket authenticated"
    );

    // Phase 2: Message loop
    let mut joined_rooms: HashSet<Uuid> = HashSet::new();
    let mut room_rx: Option<broadcast::Receiver<BroadcastMsg>> = None;
    let mut heartbeat_timer =
        tokio::time::interval_at(Instant::now() + HEARTBEAT_INTERVAL, HEARTBEAT_INTERVAL);
    let mut last_pong = Instant::now();

    loop {
        tokio::select! {
            // Incoming WebSocket message
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Err(should_close) = handle_message(
                            &text,
                            &mut socket,
                            &state,
                            conn_id,
                            account_id,
                            device_id,
                            &mut joined_rooms,
                            &mut room_rx,
                        ).await {
                            if should_close {
                                break;
                            }
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        last_pong = Instant::now();
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    Some(Err(e)) => {
                        tracing::warn!(conn_id = %conn_id, "WebSocket error: {e}");
                        break;
                    }
                    _ => {} // Binary, Ping — ignore
                }
            }

            // Room broadcast messages
            msg = async {
                match room_rx.as_mut() {
                    Some(rx) => rx.recv().await,
                    None => std::future::pending().await,
                }
            } => {
                match msg {
                    Ok(broadcast_msg) => {
                        // Skip messages from ourselves
                        if broadcast_msg.sender_conn_id != conn_id
                            && socket.send(Message::Text(broadcast_msg.json.into())).await.is_err()
                        {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!(conn_id = %conn_id, "Broadcast lagged by {n} messages");
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        room_rx = None;
                    }
                }
            }

            // Heartbeat
            _ = heartbeat_timer.tick() => {
                if last_pong.elapsed() > HEARTBEAT_INTERVAL + HEARTBEAT_TIMEOUT {
                    tracing::warn!(conn_id = %conn_id, "Heartbeat timeout, closing connection");
                    break;
                }
                if socket.send(Message::Ping(vec![].into())).await.is_err() {
                    break;
                }
            }
        }
    }

    // Cleanup: leave all rooms
    for project_id in &joined_rooms {
        state.room_manager.leave(*project_id);
    }
    state.room_manager.cleanup_empty_rooms();

    // Update last_seen
    let dev_service = state.device_service.clone();
    let _ = tokio::task::spawn_blocking(move || {
        let svc = dev_service.lock().unwrap();
        let _ = svc.update_last_seen(device_id);
    })
    .await;

    tracing::info!(conn_id = %conn_id, "WebSocket disconnected");
}

/// Authenticate the first message on the WebSocket.
/// Returns (account_id, device_id) on success, or None if auth fails.
async fn authenticate(socket: &mut WebSocket, state: &AppState) -> Option<(Uuid, Uuid)> {
    // Wait for the first message (with timeout)
    let msg = tokio::time::timeout(Duration::from_secs(10), socket.recv()).await;

    let text = match msg {
        Ok(Some(Ok(Message::Text(text)))) => text,
        _ => {
            let _ = send_error(socket, ErrorCode::Unauthorized, "Expected Auth message").await;
            return None;
        }
    };

    let sync_msg = match SyncMessage::from_json(&text) {
        Ok(msg) => msg,
        Err(_) => {
            let _ = send_error(socket, ErrorCode::Unauthorized, "Invalid message format").await;
            return None;
        }
    };

    match sync_msg {
        SyncMessage::Auth { token } => match state.jwt_service.validate_token(&token) {
            Ok(claims) => {
                let auth_ok = SyncMessage::AuthOk {
                    server_version: SERVER_VERSION.to_string(),
                };
                if let Ok(json) = auth_ok.to_json() {
                    let _ = socket.send(Message::Text(json.into())).await;
                }
                Some((claims.sub, claims.device_id))
            }
            Err(_) => {
                let _ =
                    send_error(socket, ErrorCode::Unauthorized, "Invalid or expired token").await;
                None
            }
        },
        _ => {
            let _ = send_error(
                socket,
                ErrorCode::Unauthorized,
                "First message must be Auth",
            )
            .await;
            None
        }
    }
}

/// Handle a parsed SyncMessage. Returns Err(true) if connection should close.
#[allow(clippy::too_many_arguments)]
async fn handle_message(
    text: &str,
    socket: &mut WebSocket,
    state: &AppState,
    conn_id: Uuid,
    _account_id: Uuid,
    _device_id: Uuid,
    joined_rooms: &mut HashSet<Uuid>,
    room_rx: &mut Option<broadcast::Receiver<BroadcastMsg>>,
) -> Result<(), bool> {
    let msg = match SyncMessage::from_json(text) {
        Ok(m) => m,
        Err(e) => {
            let _ = send_error(
                socket,
                ErrorCode::InternalError,
                &format!("Invalid message: {e}"),
            )
            .await;
            return Ok(());
        }
    };

    match msg {
        SyncMessage::JoinRoom { project_id } => {
            let rx = state.room_manager.join(project_id);
            *room_rx = Some(rx);
            joined_rooms.insert(project_id);

            let response = SyncMessage::RoomJoined {
                project_id,
                server_version: vec![],
            };
            if let Ok(json) = response.to_json() {
                let _ = socket.send(Message::Text(json.into())).await;
            }
        }

        SyncMessage::LeaveRoom { project_id } => {
            joined_rooms.remove(&project_id);
            state.room_manager.leave(project_id);
            if joined_rooms.is_empty() {
                *room_rx = None;
            }
        }

        SyncMessage::EncryptedUpdate {
            project_id,
            envelope,
            sequence,
            device_id: update_device_id,
        } => {
            if !joined_rooms.contains(&project_id) {
                let _ =
                    send_error(socket, ErrorCode::RoomNotFound, "Not joined to this room").await;
                return Ok(());
            }

            // Store the update
            let storage = state.storage.clone();
            let envelope_json = serde_json::to_string(&envelope).unwrap_or_default();
            let pid = project_id.to_string();
            let did = update_device_id.to_string();

            let _ = tokio::task::spawn_blocking(move || {
                let store = storage.lock().unwrap();
                store.store_update(&pid, &did, sequence, &envelope_json)
            })
            .await;

            // Broadcast to room
            let broadcast_json = text.to_string();
            state.room_manager.broadcast(
                project_id,
                BroadcastMsg {
                    sender_conn_id: conn_id,
                    json: broadcast_json,
                },
            );
        }

        SyncMessage::EncryptedSnapshot {
            project_id,
            envelope,
            snapshot_id,
        } => {
            if !joined_rooms.contains(&project_id) {
                let _ =
                    send_error(socket, ErrorCode::RoomNotFound, "Not joined to this room").await;
                return Ok(());
            }

            let storage = state.storage.clone();
            let envelope_json = serde_json::to_string(&envelope).unwrap_or_default();
            let pid = project_id.to_string();
            let sid = snapshot_id.to_string();

            let _ = tokio::task::spawn_blocking(move || {
                let store = storage.lock().unwrap();
                store.store_snapshot(&pid, &sid, &envelope_json)
            })
            .await;
        }

        SyncMessage::SyncRequest {
            project_id,
            since_sequence,
        } => {
            let storage = state.storage.clone();
            let pid = project_id.to_string();

            let result = tokio::task::spawn_blocking(move || {
                let store = storage.lock().unwrap();
                let updates = store.get_updates_since(&pid, since_sequence, 1000)?;
                let snapshot = store.get_latest_snapshot(&pid)?;
                Ok::<_, crate::error::ServerError>((updates, snapshot))
            })
            .await;

            match result {
                Ok(Ok((stored_updates, snapshot))) => {
                    // Convert stored updates to SyncMessage::EncryptedUpdate
                    let updates: Vec<SyncMessage> = stored_updates
                        .into_iter()
                        .filter_map(|u| {
                            let envelope: sakya_sync_protocol::EncryptedEnvelope =
                                serde_json::from_str(&u.envelope_json).ok()?;
                            let dev_id = Uuid::parse_str(&u.device_id).ok()?;
                            Some(SyncMessage::EncryptedUpdate {
                                project_id,
                                envelope,
                                sequence: u.sequence,
                                device_id: dev_id,
                            })
                        })
                        .collect();

                    let latest_snapshot = snapshot.and_then(|s| {
                        let envelope: sakya_sync_protocol::EncryptedEnvelope =
                            serde_json::from_str(&s.envelope_json).ok()?;
                        let snap_id = Uuid::parse_str(&s.snapshot_id).ok()?;
                        Some(Box::new(SyncMessage::EncryptedSnapshot {
                            project_id,
                            envelope,
                            snapshot_id: snap_id,
                        }))
                    });

                    let response = SyncMessage::SyncResponse {
                        project_id,
                        updates,
                        latest_snapshot,
                    };
                    if let Ok(json) = response.to_json() {
                        let _ = socket.send(Message::Text(json.into())).await;
                    }
                }
                _ => {
                    let _ = send_error(socket, ErrorCode::InternalError, "Failed to fetch updates")
                        .await;
                }
            }
        }

        SyncMessage::Ephemeral {
            project_id,
            data: _,
        } => {
            if !joined_rooms.contains(&project_id) {
                return Ok(());
            }

            // Broadcast without storing
            state.room_manager.broadcast(
                project_id,
                BroadcastMsg {
                    sender_conn_id: conn_id,
                    json: text.to_string(),
                },
            );
        }

        SyncMessage::Ping => {
            let pong = SyncMessage::Pong;
            if let Ok(json) = pong.to_json() {
                let _ = socket.send(Message::Text(json.into())).await;
            }
        }

        _ => {
            // Ignore unexpected messages
        }
    }

    Ok(())
}

/// Send an error message over the WebSocket.
async fn send_error(
    socket: &mut WebSocket,
    code: ErrorCode,
    message: &str,
) -> Result<(), axum::Error> {
    let error = SyncMessage::Error {
        code,
        message: message.to_string(),
    };
    if let Ok(json) = error.to_json() {
        socket.send(Message::Text(json.into())).await
    } else {
        Ok(())
    }
}
