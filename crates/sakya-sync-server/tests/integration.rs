//! Integration tests for sakya-sync-server.
//!
//! Each test spins up an in-process Axum server on a random port
//! and tests the full HTTP + WebSocket flows.

use futures_util::{SinkExt, StreamExt};
use sakya_sync_protocol::{ErrorCode, SyncMessage};
use sakya_sync_server::routes::build_router;
use sakya_sync_server::state::AppState;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use uuid::Uuid;

/// Start the server on a random port and return its address.
async fn start_server() -> (SocketAddr, AppState) {
    let state = AppState::new_test("integration-test-secret");
    let app = build_router(state.clone()).layer(tower_http::cors::CorsLayer::permissive());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    (addr, state)
}

/// Create an account via magic link and return (account_id, device_id, jwt).
fn create_test_account(state: &AppState) -> (Uuid, Uuid, String) {
    let token = {
        let svc = state.magic_link_service.lock().unwrap();
        svc.create_magic_link("integration@example.com").unwrap()
    };
    let (account_id, _) = {
        let svc = state.magic_link_service.lock().unwrap();
        svc.verify_magic_link(&token).unwrap()
    };
    let device_id = {
        let svc = state.device_service.lock().unwrap();
        svc.register_device(account_id, "Test Device", &[0u8; 32])
            .unwrap()
    };
    let jwt = state
        .jwt_service
        .generate_token(account_id, device_id)
        .unwrap();
    (account_id, device_id, jwt)
}

/// Connect a WebSocket client and authenticate.
async fn connect_and_auth(
    addr: SocketAddr,
    jwt: &str,
) -> tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>> {
    let url = format!("ws://{addr}/sync");
    let (mut ws, _) = tokio_tungstenite::connect_async(&url).await.unwrap();

    // Send Auth message
    let auth = SyncMessage::Auth {
        token: jwt.to_string(),
    };
    ws.send(WsMessage::Text(auth.to_json().unwrap().into()))
        .await
        .unwrap();

    // Read AuthOk response
    let msg = ws.next().await.unwrap().unwrap();
    let text = msg.into_text().unwrap();
    let parsed = SyncMessage::from_json(&text).unwrap();
    match parsed {
        SyncMessage::AuthOk { server_version } => {
            assert!(!server_version.is_empty());
        }
        other => panic!("Expected AuthOk, got: {other:?}"),
    }

    ws
}

// ─── Test 1: Health endpoint ─────────────────────────────────

#[tokio::test]
async fn health_endpoint() {
    let (addr, _state) = start_server().await;

    let resp = reqwest::get(format!("http://{addr}/health")).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
    assert!(body["version"].is_string());
}

// ─── Test 2: Auth flow end-to-end ────────────────────────────

#[tokio::test]
async fn auth_flow_end_to_end() {
    let (addr, state) = start_server().await;

    // Request magic link
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{addr}/auth/magic-link"))
        .json(&serde_json::json!({"email": "e2e@example.com"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Get the actual token from the magic link service directly
    let token = {
        let svc = state.magic_link_service.lock().unwrap();
        svc.create_magic_link("e2e-verify@example.com").unwrap()
    };

    // Verify magic link
    let resp = client
        .post(format!("http://{addr}/auth/verify"))
        .json(&serde_json::json!({
            "token": token,
            "device_name": "E2E Device",
            "public_key": vec![42u8; 32]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["jwt"].is_string());
    assert!(body["account_id"].is_string());
    assert!(body["device_id"].is_string());
}

// ─── Test 3: Device CRUD ─────────────────────────────────────

#[tokio::test]
async fn device_crud() {
    let (addr, state) = start_server().await;
    let (_account_id, _device_id, jwt) = create_test_account(&state);

    let client = reqwest::Client::new();

    // List devices
    let resp = client
        .get(format!("http://{addr}/devices"))
        .bearer_auth(&jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let devices: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(devices.len(), 1);

    // Register new device
    let resp = client
        .post(format!("http://{addr}/devices"))
        .bearer_auth(&jwt)
        .json(&serde_json::json!({
            "name": "Second Device",
            "public_key": vec![1u8; 32]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let new_device_id = body["device_id"].as_str().unwrap();

    // List again — should be 2
    let resp = client
        .get(format!("http://{addr}/devices"))
        .bearer_auth(&jwt)
        .send()
        .await
        .unwrap();
    let devices: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(devices.len(), 2);

    // Remove the new device
    let resp = client
        .delete(format!("http://{addr}/devices/{new_device_id}"))
        .bearer_auth(&jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    // List again — should be 1
    let resp = client
        .get(format!("http://{addr}/devices"))
        .bearer_auth(&jwt)
        .send()
        .await
        .unwrap();
    let devices: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(devices.len(), 1);
}

// ─── Test 4: WS auth required ────────────────────────────────

#[tokio::test]
async fn ws_auth_required() {
    let (addr, _state) = start_server().await;

    let url = format!("ws://{addr}/sync");
    let (mut ws, _) = tokio_tungstenite::connect_async(&url).await.unwrap();

    // Send a non-Auth message
    let msg = SyncMessage::Ping;
    ws.send(WsMessage::Text(msg.to_json().unwrap().into()))
        .await
        .unwrap();

    // Should get an error
    let response = ws.next().await.unwrap().unwrap();
    let text = response.into_text().unwrap();
    let parsed = SyncMessage::from_json(&text).unwrap();
    match parsed {
        SyncMessage::Error { code, .. } => {
            assert_eq!(code, ErrorCode::Unauthorized);
        }
        other => panic!("Expected Error, got: {other:?}"),
    }
}

// ─── Test 5: WS auth success ─────────────────────────────────

#[tokio::test]
async fn ws_auth_success() {
    let (addr, state) = start_server().await;
    let (_account_id, _device_id, jwt) = create_test_account(&state);

    let _ws = connect_and_auth(addr, &jwt).await;
    // If we got here without panic, auth succeeded
}

// ─── Test 6: WS join room ────────────────────────────────────

#[tokio::test]
async fn ws_join_room() {
    let (addr, state) = start_server().await;
    let (_account_id, _device_id, jwt) = create_test_account(&state);
    let mut ws = connect_and_auth(addr, &jwt).await;

    let project_id = Uuid::new_v4();
    let join = SyncMessage::JoinRoom { project_id };
    ws.send(WsMessage::Text(join.to_json().unwrap().into()))
        .await
        .unwrap();

    let msg = ws.next().await.unwrap().unwrap();
    let text = msg.into_text().unwrap();
    let parsed = SyncMessage::from_json(&text).unwrap();
    match parsed {
        SyncMessage::RoomJoined {
            project_id: pid, ..
        } => {
            assert_eq!(pid, project_id);
        }
        other => panic!("Expected RoomJoined, got: {other:?}"),
    }
}

// ─── Test 7: Two clients relay ───────────────────────────────

#[tokio::test]
async fn two_clients_relay() {
    let (addr, state) = start_server().await;

    // Create two accounts with separate JWTs
    let token1 = {
        let svc = state.magic_link_service.lock().unwrap();
        svc.create_magic_link("client1@example.com").unwrap()
    };
    let (account_id1, _) = {
        let svc = state.magic_link_service.lock().unwrap();
        svc.verify_magic_link(&token1).unwrap()
    };
    let device_id1 = {
        let svc = state.device_service.lock().unwrap();
        svc.register_device(account_id1, "Client1", &[1u8; 32])
            .unwrap()
    };
    let jwt1 = state
        .jwt_service
        .generate_token(account_id1, device_id1)
        .unwrap();

    let token2 = {
        let svc = state.magic_link_service.lock().unwrap();
        svc.create_magic_link("client2@example.com").unwrap()
    };
    let (account_id2, _) = {
        let svc = state.magic_link_service.lock().unwrap();
        svc.verify_magic_link(&token2).unwrap()
    };
    let device_id2 = {
        let svc = state.device_service.lock().unwrap();
        svc.register_device(account_id2, "Client2", &[2u8; 32])
            .unwrap()
    };
    let jwt2 = state
        .jwt_service
        .generate_token(account_id2, device_id2)
        .unwrap();

    let mut ws1 = connect_and_auth(addr, &jwt1).await;
    let mut ws2 = connect_and_auth(addr, &jwt2).await;

    let project_id = Uuid::new_v4();

    // Both join the same room
    let join = SyncMessage::JoinRoom { project_id };
    ws1.send(WsMessage::Text(join.to_json().unwrap().into()))
        .await
        .unwrap();
    ws2.send(WsMessage::Text(join.to_json().unwrap().into()))
        .await
        .unwrap();

    // Read RoomJoined for both
    let _ = ws1.next().await.unwrap().unwrap();
    let _ = ws2.next().await.unwrap().unwrap();

    // Client 1 sends an encrypted update
    let update = SyncMessage::EncryptedUpdate {
        project_id,
        envelope: sakya_sync_protocol::EncryptedEnvelope {
            nonce: vec![1; 12],
            ciphertext: vec![0xDE, 0xAD],
            aad: vec![0xAA],
        },
        sequence: 1,
        device_id: device_id1,
    };
    ws1.send(WsMessage::Text(update.to_json().unwrap().into()))
        .await
        .unwrap();

    // Client 2 should receive the update
    let msg = tokio::time::timeout(std::time::Duration::from_secs(5), ws2.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let text = msg.into_text().unwrap();
    let parsed = SyncMessage::from_json(&text).unwrap();
    match parsed {
        SyncMessage::EncryptedUpdate {
            sequence,
            device_id,
            ..
        } => {
            assert_eq!(sequence, 1);
            assert_eq!(device_id, device_id1);
        }
        other => panic!("Expected EncryptedUpdate, got: {other:?}"),
    }
}

// ─── Test 8: Sync request returns stored updates ─────────────

#[tokio::test]
async fn sync_request_returns_stored() {
    let (addr, state) = start_server().await;
    let (_account_id, device_id, jwt) = create_test_account(&state);

    let mut ws = connect_and_auth(addr, &jwt).await;

    let project_id = Uuid::new_v4();

    // Join room
    let join = SyncMessage::JoinRoom { project_id };
    ws.send(WsMessage::Text(join.to_json().unwrap().into()))
        .await
        .unwrap();
    let _ = ws.next().await.unwrap().unwrap(); // RoomJoined

    // Send some updates
    for seq in 1..=3 {
        let update = SyncMessage::EncryptedUpdate {
            project_id,
            envelope: sakya_sync_protocol::EncryptedEnvelope {
                nonce: vec![seq as u8; 12],
                ciphertext: vec![seq as u8],
                aad: vec![],
            },
            sequence: seq,
            device_id,
        };
        ws.send(WsMessage::Text(update.to_json().unwrap().into()))
            .await
            .unwrap();
    }

    // Small delay to let storage complete
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Request sync from sequence 1 (should get 2 and 3)
    let request = SyncMessage::SyncRequest {
        project_id,
        since_sequence: 1,
    };
    ws.send(WsMessage::Text(request.to_json().unwrap().into()))
        .await
        .unwrap();

    let msg = tokio::time::timeout(std::time::Duration::from_secs(5), ws.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let text = msg.into_text().unwrap();
    let parsed = SyncMessage::from_json(&text).unwrap();
    match parsed {
        SyncMessage::SyncResponse { updates, .. } => {
            assert_eq!(updates.len(), 2, "should get updates with seq 2 and 3");
        }
        other => panic!("Expected SyncResponse, got: {other:?}"),
    }
}

// ─── Test 9: WS heartbeat ping-pong ─────────────────────────

#[tokio::test]
async fn ws_heartbeat_ping_pong() {
    let (addr, state) = start_server().await;
    let (_account_id, _device_id, jwt) = create_test_account(&state);
    let mut ws = connect_and_auth(addr, &jwt).await;

    // Send application-level Ping
    let ping = SyncMessage::Ping;
    ws.send(WsMessage::Text(ping.to_json().unwrap().into()))
        .await
        .unwrap();

    // Should get Pong back
    let msg = tokio::time::timeout(std::time::Duration::from_secs(5), ws.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let text = msg.into_text().unwrap();
    let parsed = SyncMessage::from_json(&text).unwrap();
    assert_eq!(parsed, SyncMessage::Pong);
}

// ─── Test 10: WS disconnect cleanup ─────────────────────────

#[tokio::test]
async fn ws_disconnect_cleanup() {
    let (addr, state) = start_server().await;
    let (_account_id, _device_id, jwt) = create_test_account(&state);

    let project_id = Uuid::new_v4();

    {
        let mut ws = connect_and_auth(addr, &jwt).await;

        // Join room
        let join = SyncMessage::JoinRoom { project_id };
        ws.send(WsMessage::Text(join.to_json().unwrap().into()))
            .await
            .unwrap();
        let _ = ws.next().await.unwrap().unwrap(); // RoomJoined

        assert!(state.room_manager.has_room(project_id));

        // Close connection
        ws.close(None).await.unwrap();
    }

    // Give server time to process disconnect
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Room should be cleaned up
    state.room_manager.cleanup_empty_rooms();
    assert!(
        !state.room_manager.has_room(project_id),
        "Room should be cleaned up after disconnect"
    );
}

// ─── Test 11: Invalid JSON returns error ─────────────────────

#[tokio::test]
async fn invalid_json_returns_error() {
    let (addr, state) = start_server().await;
    let (_account_id, _device_id, jwt) = create_test_account(&state);
    let mut ws = connect_and_auth(addr, &jwt).await;

    // Send garbage
    ws.send(WsMessage::Text("not valid json {{{".into()))
        .await
        .unwrap();

    let msg = tokio::time::timeout(std::time::Duration::from_secs(5), ws.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let text = msg.into_text().unwrap();
    let parsed = SyncMessage::from_json(&text).unwrap();
    match parsed {
        SyncMessage::Error { code, message } => {
            assert_eq!(code, ErrorCode::InternalError);
            assert!(message.contains("Invalid message"));
        }
        other => panic!("Expected Error, got: {other:?}"),
    }
}

// ─── Test 12: Update for unjoined room ───────────────────────

#[tokio::test]
async fn update_for_unjoined_room() {
    let (addr, state) = start_server().await;
    let (_account_id, device_id, jwt) = create_test_account(&state);
    let mut ws = connect_and_auth(addr, &jwt).await;

    // Send update without joining
    let update = SyncMessage::EncryptedUpdate {
        project_id: Uuid::new_v4(),
        envelope: sakya_sync_protocol::EncryptedEnvelope {
            nonce: vec![0; 12],
            ciphertext: vec![1],
            aad: vec![],
        },
        sequence: 1,
        device_id,
    };
    ws.send(WsMessage::Text(update.to_json().unwrap().into()))
        .await
        .unwrap();

    // Should get a RoomNotFound error
    let msg = tokio::time::timeout(std::time::Duration::from_secs(5), ws.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let text = msg.into_text().unwrap();
    let parsed = SyncMessage::from_json(&text).unwrap();
    match parsed {
        SyncMessage::Error { code, .. } => {
            assert_eq!(code, ErrorCode::RoomNotFound);
        }
        other => panic!("Expected Error with RoomNotFound, got: {other:?}"),
    }
}

// ─── Test 13: Ephemeral not stored ───────────────────────────

#[tokio::test]
async fn ephemeral_not_stored() {
    let (addr, state) = start_server().await;
    let (_account_id, _device_id, jwt) = create_test_account(&state);
    let mut ws = connect_and_auth(addr, &jwt).await;

    let project_id = Uuid::new_v4();

    // Join room
    let join = SyncMessage::JoinRoom { project_id };
    ws.send(WsMessage::Text(join.to_json().unwrap().into()))
        .await
        .unwrap();
    let _ = ws.next().await.unwrap().unwrap(); // RoomJoined

    // Send ephemeral message
    let eph = SyncMessage::Ephemeral {
        project_id,
        data: vec![1, 2, 3],
    };
    ws.send(WsMessage::Text(eph.to_json().unwrap().into()))
        .await
        .unwrap();

    // Small delay
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Verify nothing stored
    let updates = {
        let store = state.storage.lock().unwrap();
        store
            .get_updates_since(&project_id.to_string(), 0, 100)
            .unwrap()
    };
    assert!(
        updates.is_empty(),
        "Ephemeral messages should not be stored"
    );
}

// ─── Test 14: Snapshot replaces old ──────────────────────────

#[tokio::test]
async fn snapshot_replaces_old() {
    let (addr, state) = start_server().await;
    let (_account_id, _device_id, jwt) = create_test_account(&state);
    let mut ws = connect_and_auth(addr, &jwt).await;

    let project_id = Uuid::new_v4();

    // Join room
    let join = SyncMessage::JoinRoom { project_id };
    ws.send(WsMessage::Text(join.to_json().unwrap().into()))
        .await
        .unwrap();
    let _ = ws.next().await.unwrap().unwrap(); // RoomJoined

    // Send two snapshots
    for i in 1..=2 {
        let snap = SyncMessage::EncryptedSnapshot {
            project_id,
            envelope: sakya_sync_protocol::EncryptedEnvelope {
                nonce: vec![i; 12],
                ciphertext: vec![i],
                aad: vec![],
            },
            snapshot_id: Uuid::new_v4(),
        };
        ws.send(WsMessage::Text(snap.to_json().unwrap().into()))
            .await
            .unwrap();
    }

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Request sync — should get latest snapshot
    let request = SyncMessage::SyncRequest {
        project_id,
        since_sequence: 0,
    };
    ws.send(WsMessage::Text(request.to_json().unwrap().into()))
        .await
        .unwrap();

    let msg = tokio::time::timeout(std::time::Duration::from_secs(5), ws.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let text = msg.into_text().unwrap();
    let parsed = SyncMessage::from_json(&text).unwrap();
    match parsed {
        SyncMessage::SyncResponse {
            latest_snapshot, ..
        } => {
            assert!(latest_snapshot.is_some(), "Should have a latest snapshot");
            match *latest_snapshot.unwrap() {
                SyncMessage::EncryptedSnapshot { envelope, .. } => {
                    assert_eq!(
                        envelope.ciphertext,
                        vec![2],
                        "Should be the second (latest) snapshot"
                    );
                }
                other => panic!("Expected EncryptedSnapshot, got: {other:?}"),
            }
        }
        other => panic!("Expected SyncResponse, got: {other:?}"),
    }
}

// ─── Test 15: Concurrent two rooms ───────────────────────────

#[tokio::test]
async fn concurrent_two_rooms() {
    let (addr, state) = start_server().await;
    let (_account_id, device_id, jwt) = create_test_account(&state);

    let mut ws = connect_and_auth(addr, &jwt).await;

    let room_a = Uuid::new_v4();
    let room_b = Uuid::new_v4();

    // Join both rooms
    ws.send(WsMessage::Text(
        SyncMessage::JoinRoom { project_id: room_a }
            .to_json()
            .unwrap()
            .into(),
    ))
    .await
    .unwrap();
    let _ = ws.next().await.unwrap().unwrap(); // RoomJoined A

    ws.send(WsMessage::Text(
        SyncMessage::JoinRoom { project_id: room_b }
            .to_json()
            .unwrap()
            .into(),
    ))
    .await
    .unwrap();
    let _ = ws.next().await.unwrap().unwrap(); // RoomJoined B

    // Send updates to both rooms
    let update_a = SyncMessage::EncryptedUpdate {
        project_id: room_a,
        envelope: sakya_sync_protocol::EncryptedEnvelope {
            nonce: vec![1; 12],
            ciphertext: vec![0xAA],
            aad: vec![],
        },
        sequence: 1,
        device_id,
    };
    ws.send(WsMessage::Text(update_a.to_json().unwrap().into()))
        .await
        .unwrap();

    let update_b = SyncMessage::EncryptedUpdate {
        project_id: room_b,
        envelope: sakya_sync_protocol::EncryptedEnvelope {
            nonce: vec![2; 12],
            ciphertext: vec![0xBB],
            aad: vec![],
        },
        sequence: 1,
        device_id,
    };
    ws.send(WsMessage::Text(update_b.to_json().unwrap().into()))
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Verify both rooms have their updates stored independently
    {
        let store = state.storage.lock().unwrap();
        let updates_a = store
            .get_updates_since(&room_a.to_string(), 0, 100)
            .unwrap();
        let updates_b = store
            .get_updates_since(&room_b.to_string(), 0, 100)
            .unwrap();
        assert_eq!(updates_a.len(), 1);
        assert_eq!(updates_b.len(), 1);
    }
}
