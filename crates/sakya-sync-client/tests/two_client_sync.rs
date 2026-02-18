//! Two-client sync E2E integration tests.
//!
//! Each test spins up an in-process sync server, connects two SyncEngine
//! instances, and verifies end-to-end encrypted sync flows.

use sakya_sync_client::{SyncEngine, SyncEvent};
use sakya_sync_server::routes::build_router;
use sakya_sync_server::state::AppState;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use uuid::Uuid;

/// Start the sync server on a random port and return its address + state.
async fn start_server() -> (SocketAddr, AppState) {
    let state = AppState::new_test("two-client-test-secret");
    let app = build_router(state.clone()).layer(tower_http::cors::CorsLayer::permissive());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(10)).await;
    (addr, state)
}

/// Create an account and return (account_id, device_id, jwt).
/// Uses unique email addresses per call to avoid conflicts.
fn create_test_account(state: &AppState, email: &str, device_name: &str) -> (Uuid, Uuid, String) {
    let token = {
        let svc = state.magic_link_service.lock().unwrap();
        svc.create_magic_link(email).unwrap()
    };
    let (account_id, _) = {
        let svc = state.magic_link_service.lock().unwrap();
        svc.verify_magic_link(&token).unwrap()
    };
    let device_id = {
        let svc = state.device_service.lock().unwrap();
        svc.register_device(account_id, device_name, &[0u8; 32])
            .unwrap()
    };
    let jwt = state
        .jwt_service
        .generate_token(account_id, device_id)
        .unwrap();
    (account_id, device_id, jwt)
}

/// Helper: wait for a specific SyncEvent on a receiver, with timeout.
async fn wait_for_event<F>(
    rx: &mut tokio::sync::broadcast::Receiver<SyncEvent>,
    timeout_secs: u64,
    mut predicate: F,
) -> SyncEvent
where
    F: FnMut(&SyncEvent) -> bool,
{
    tokio::time::timeout(Duration::from_secs(timeout_secs), async {
        loop {
            match rx.recv().await {
                Ok(event) if predicate(&event) => return event,
                Ok(_) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("Lagged by {n} messages");
                    continue;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    panic!("Event channel closed unexpectedly");
                }
            }
        }
    })
    .await
    .expect("Timed out waiting for sync event")
}

/// Helper: wait until engine reaches a target status, with timeout.
async fn wait_for_status(engine: &SyncEngine, target: &str, timeout_secs: u64) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(timeout_secs);
    loop {
        let status = engine.status().await;
        let status_str = format!("{status:?}");
        if status_str.contains(target) {
            return;
        }
        if tokio::time::Instant::now() > deadline {
            panic!("Timed out waiting for status '{target}', current: {status:?}");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

// ─── Test 1: Online sync — Client A sends, Client B receives ─────

#[tokio::test]
async fn online_sync_a_to_b() {
    let (addr, state) = start_server().await;

    let (_acct1, dev1, jwt1) = create_test_account(&state, "client1@test.com", "Device A");
    let (_acct2, _dev2, jwt2) = create_test_account(&state, "client2@test.com", "Device B");

    let engine_a = SyncEngine::connect(format!("ws://{addr}"), jwt1, dev1)
        .await
        .unwrap();
    let engine_b = SyncEngine::connect(format!("ws://{addr}"), jwt2, Uuid::new_v4())
        .await
        .unwrap();

    // Wait for both to connect
    wait_for_status(&engine_a, "Connected", 5).await;
    wait_for_status(&engine_b, "Connected", 5).await;

    // Both join the same project with the same encryption key
    let project_id = Uuid::new_v4();
    let shared_key = [42u8; 32];

    engine_a
        .enable_project(project_id, shared_key)
        .await
        .unwrap();
    engine_b
        .enable_project(project_id, shared_key)
        .await
        .unwrap();

    // Wait for ProjectJoined on both
    let mut rx_a = engine_a.subscribe();
    let mut rx_b = engine_b.subscribe();

    wait_for_event(
        &mut rx_a,
        5,
        |e| matches!(e, SyncEvent::ProjectJoined { project_id: pid } if *pid == project_id),
    )
    .await;
    wait_for_event(
        &mut rx_b,
        5,
        |e| matches!(e, SyncEvent::ProjectJoined { project_id: pid } if *pid == project_id),
    )
    .await;

    // Small delay for room membership to propagate
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Client A sends an update
    let update_data = b"chapter 1 CRDT update from A".to_vec();
    engine_a
        .send_update(project_id, update_data.clone())
        .await
        .unwrap();

    // Client B should receive the decrypted update
    let event = wait_for_event(
        &mut rx_b,
        5,
        |e| matches!(e, SyncEvent::UpdateReceived { project_id: pid, .. } if *pid == project_id),
    )
    .await;

    match event {
        SyncEvent::UpdateReceived { update_bytes, .. } => {
            assert_eq!(
                update_bytes, update_data,
                "Client B should receive the exact plaintext"
            );
        }
        _ => unreachable!(),
    }

    engine_a.disconnect().await.unwrap();
    engine_b.disconnect().await.unwrap();
}

// ─── Test 2: Bidirectional sync — both clients edit ──────────────

#[tokio::test]
async fn bidirectional_sync() {
    let (addr, state) = start_server().await;

    let (_acct1, dev1, jwt1) = create_test_account(&state, "bidir1@test.com", "Device A");
    let (_acct2, dev2, jwt2) = create_test_account(&state, "bidir2@test.com", "Device B");

    let engine_a = SyncEngine::connect(format!("ws://{addr}"), jwt1, dev1)
        .await
        .unwrap();
    let engine_b = SyncEngine::connect(format!("ws://{addr}"), jwt2, dev2)
        .await
        .unwrap();

    wait_for_status(&engine_a, "Connected", 5).await;
    wait_for_status(&engine_b, "Connected", 5).await;

    let project_id = Uuid::new_v4();
    let shared_key = [99u8; 32];

    engine_a
        .enable_project(project_id, shared_key)
        .await
        .unwrap();
    engine_b
        .enable_project(project_id, shared_key)
        .await
        .unwrap();

    let mut rx_a = engine_a.subscribe();
    let mut rx_b = engine_b.subscribe();

    wait_for_event(&mut rx_a, 5, |e| {
        matches!(e, SyncEvent::ProjectJoined { .. })
    })
    .await;
    wait_for_event(&mut rx_b, 5, |e| {
        matches!(e, SyncEvent::ProjectJoined { .. })
    })
    .await;

    tokio::time::sleep(Duration::from_millis(100)).await;

    // A sends
    let update_a = b"edit from A".to_vec();
    engine_a
        .send_update(project_id, update_a.clone())
        .await
        .unwrap();

    // B receives A's update
    let event = wait_for_event(&mut rx_b, 5, |e| {
        matches!(e, SyncEvent::UpdateReceived { .. })
    })
    .await;
    match event {
        SyncEvent::UpdateReceived { update_bytes, .. } => {
            assert_eq!(update_bytes, update_a);
        }
        _ => unreachable!(),
    }

    // B sends
    let update_b = b"edit from B".to_vec();
    engine_b
        .send_update(project_id, update_b.clone())
        .await
        .unwrap();

    // A receives B's update
    let event = wait_for_event(&mut rx_a, 5, |e| {
        matches!(e, SyncEvent::UpdateReceived { .. })
    })
    .await;
    match event {
        SyncEvent::UpdateReceived { update_bytes, .. } => {
            assert_eq!(update_bytes, update_b);
        }
        _ => unreachable!(),
    }

    engine_a.disconnect().await.unwrap();
    engine_b.disconnect().await.unwrap();
}

// ─── Test 3: Sync request catch-up — late joiner gets missed updates ─

#[tokio::test]
async fn sync_request_catchup() {
    let (addr, state) = start_server().await;

    let (_acct1, dev1, jwt1) = create_test_account(&state, "catchup1@test.com", "Device A");
    let (_acct2, dev2, jwt2) = create_test_account(&state, "catchup2@test.com", "Device B");

    // Client A connects, joins, sends updates
    let engine_a = SyncEngine::connect(format!("ws://{addr}"), jwt1, dev1)
        .await
        .unwrap();
    wait_for_status(&engine_a, "Connected", 5).await;

    let project_id = Uuid::new_v4();
    let shared_key = [77u8; 32];

    engine_a
        .enable_project(project_id, shared_key)
        .await
        .unwrap();

    let mut rx_a = engine_a.subscribe();
    wait_for_event(&mut rx_a, 5, |e| {
        matches!(e, SyncEvent::ProjectJoined { .. })
    })
    .await;

    // A sends 3 updates while B is not connected
    for i in 0..3 {
        let data = format!("update-{i}").into_bytes();
        engine_a.send_update(project_id, data).await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Small delay to let server persist updates
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Now Client B connects and joins the same project
    let engine_b = SyncEngine::connect(format!("ws://{addr}"), jwt2, dev2)
        .await
        .unwrap();
    wait_for_status(&engine_b, "Connected", 5).await;

    let mut rx_b = engine_b.subscribe();
    engine_b
        .enable_project(project_id, shared_key)
        .await
        .unwrap();

    wait_for_event(&mut rx_b, 5, |e| {
        matches!(e, SyncEvent::ProjectJoined { .. })
    })
    .await;

    // B sends a SyncRequest to catch up (via the raw message API)
    use sakya_sync_protocol::SyncMessage;
    engine_b
        .send_message(SyncMessage::SyncRequest {
            project_id,
            since_sequence: 0,
        })
        .await
        .unwrap();

    // B should receive the 3 missed updates (from SyncResponse)
    let mut received_updates = Vec::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);

    while received_updates.len() < 3 && tokio::time::Instant::now() < deadline {
        tokio::select! {
            event = rx_b.recv() => {
                match event {
                    Ok(SyncEvent::UpdateReceived { update_bytes, project_id: pid }) => {
                        if pid == project_id {
                            received_updates.push(update_bytes);
                        }
                    }
                    _ => {}
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(50)) => {}
        }
    }

    assert_eq!(
        received_updates.len(),
        3,
        "B should receive all 3 missed updates, got {}",
        received_updates.len()
    );

    // Verify the content
    for (i, update) in received_updates.iter().enumerate() {
        let expected = format!("update-{i}").into_bytes();
        assert_eq!(update, &expected, "Update {i} content mismatch");
    }

    engine_a.disconnect().await.unwrap();
    engine_b.disconnect().await.unwrap();
}

// ─── Test 4: Multiple concurrent projects on same engines ────────

#[tokio::test]
async fn concurrent_projects() {
    let (addr, state) = start_server().await;

    let (_acct1, dev1, jwt1) = create_test_account(&state, "multi1@test.com", "Device A");
    let (_acct2, dev2, jwt2) = create_test_account(&state, "multi2@test.com", "Device B");

    let engine_a = SyncEngine::connect(format!("ws://{addr}"), jwt1, dev1)
        .await
        .unwrap();
    let engine_b = SyncEngine::connect(format!("ws://{addr}"), jwt2, dev2)
        .await
        .unwrap();

    wait_for_status(&engine_a, "Connected", 5).await;
    wait_for_status(&engine_b, "Connected", 5).await;

    let project_1 = Uuid::new_v4();
    let project_2 = Uuid::new_v4();
    let key_1 = [11u8; 32];
    let key_2 = [22u8; 32];

    // Subscribe BEFORE enabling projects so we don't miss ProjectJoined events
    let mut rx_a = engine_a.subscribe();
    let mut rx_b = engine_b.subscribe();

    // Both engines join both projects
    engine_a.enable_project(project_1, key_1).await.unwrap();
    engine_a.enable_project(project_2, key_2).await.unwrap();
    engine_b.enable_project(project_1, key_1).await.unwrap();
    engine_b.enable_project(project_2, key_2).await.unwrap();

    // Wait for both joins on A
    let mut a_joined = 0;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while a_joined < 2 && tokio::time::Instant::now() < deadline {
        tokio::select! {
            event = rx_a.recv() => {
                if let Ok(SyncEvent::ProjectJoined { .. }) = event {
                    a_joined += 1;
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(50)) => {}
        }
    }

    // Wait for both joins on B
    let mut b_joined = 0;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while b_joined < 2 && tokio::time::Instant::now() < deadline {
        tokio::select! {
            event = rx_b.recv() => {
                if let Ok(SyncEvent::ProjectJoined { .. }) = event {
                    b_joined += 1;
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(50)) => {}
        }
    }

    tokio::time::sleep(Duration::from_millis(200)).await;

    // A sends to project 1
    let data_p1 = b"project-1-update".to_vec();
    engine_a
        .send_update(project_1, data_p1.clone())
        .await
        .unwrap();

    // Wait for the first update to arrive before sending the second
    tokio::time::sleep(Duration::from_millis(500)).await;

    // A sends to project 2
    let data_p2 = b"project-2-update".to_vec();
    engine_a
        .send_update(project_2, data_p2.clone())
        .await
        .unwrap();

    // B should receive both, with correct project IDs
    use std::collections::HashMap;
    let mut received = HashMap::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);

    while received.len() < 2 && tokio::time::Instant::now() < deadline {
        tokio::select! {
            event = rx_b.recv() => {
                match event {
                    Ok(SyncEvent::UpdateReceived { project_id, update_bytes }) => {
                        received.insert(project_id, update_bytes);
                    }
                    _ => {}
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(50)) => {}
        }
    }

    assert_eq!(
        received.len(),
        2,
        "Should receive updates for both projects"
    );
    assert_eq!(received.get(&project_1).unwrap(), &data_p1);
    assert_eq!(received.get(&project_2).unwrap(), &data_p2);

    engine_a.disconnect().await.unwrap();
    engine_b.disconnect().await.unwrap();
}

// ─── Test 5: Wrong key fails decryption (isolation) ──────────────

#[tokio::test]
async fn wrong_key_no_decryption() {
    let (addr, state) = start_server().await;

    let (_acct1, dev1, jwt1) = create_test_account(&state, "wrongkey1@test.com", "Device A");
    let (_acct2, dev2, jwt2) = create_test_account(&state, "wrongkey2@test.com", "Device B");

    let engine_a = SyncEngine::connect(format!("ws://{addr}"), jwt1, dev1)
        .await
        .unwrap();
    let engine_b = SyncEngine::connect(format!("ws://{addr}"), jwt2, dev2)
        .await
        .unwrap();

    wait_for_status(&engine_a, "Connected", 5).await;
    wait_for_status(&engine_b, "Connected", 5).await;

    let project_id = Uuid::new_v4();
    let key_a = [42u8; 32]; // A's key
    let key_b = [99u8; 32]; // B has a DIFFERENT key — should not decrypt

    engine_a.enable_project(project_id, key_a).await.unwrap();
    engine_b.enable_project(project_id, key_b).await.unwrap();

    let mut rx_a = engine_a.subscribe();
    let mut rx_b = engine_b.subscribe();

    wait_for_event(&mut rx_a, 5, |e| {
        matches!(e, SyncEvent::ProjectJoined { .. })
    })
    .await;
    wait_for_event(&mut rx_b, 5, |e| {
        matches!(e, SyncEvent::ProjectJoined { .. })
    })
    .await;

    tokio::time::sleep(Duration::from_millis(100)).await;

    // A sends an update
    engine_a
        .send_update(project_id, b"secret data".to_vec())
        .await
        .unwrap();

    // B should NOT receive an UpdateReceived event (decryption fails silently)
    let result = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            match rx_b.recv().await {
                Ok(SyncEvent::UpdateReceived { .. }) => return true,
                Ok(_) => continue,
                Err(_) => return false,
            }
        }
    })
    .await;

    // Either timeout (Ok but no result) or no UpdateReceived
    match result {
        Err(_) => {} // Timeout — expected: B never got UpdateReceived
        Ok(got_update) => {
            assert!(
                !got_update,
                "B should NOT have received a decrypted update with wrong key"
            );
        }
    }

    engine_a.disconnect().await.unwrap();
    engine_b.disconnect().await.unwrap();
}

// ─── Test 6: Disable project stops receiving updates ─────────────

#[tokio::test]
async fn disable_project_stops_updates() {
    let (addr, state) = start_server().await;

    let (_acct1, dev1, jwt1) = create_test_account(&state, "disable1@test.com", "Device A");
    let (_acct2, dev2, jwt2) = create_test_account(&state, "disable2@test.com", "Device B");

    let engine_a = SyncEngine::connect(format!("ws://{addr}"), jwt1, dev1)
        .await
        .unwrap();
    let engine_b = SyncEngine::connect(format!("ws://{addr}"), jwt2, dev2)
        .await
        .unwrap();

    wait_for_status(&engine_a, "Connected", 5).await;
    wait_for_status(&engine_b, "Connected", 5).await;

    let project_id = Uuid::new_v4();
    let shared_key = [55u8; 32];

    engine_a
        .enable_project(project_id, shared_key)
        .await
        .unwrap();
    engine_b
        .enable_project(project_id, shared_key)
        .await
        .unwrap();

    let mut rx_b = engine_b.subscribe();
    wait_for_event(&mut rx_b, 5, |e| {
        matches!(e, SyncEvent::ProjectJoined { .. })
    })
    .await;

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify B receives an update first
    engine_a
        .send_update(project_id, b"should arrive".to_vec())
        .await
        .unwrap();

    let event = wait_for_event(&mut rx_b, 5, |e| {
        matches!(e, SyncEvent::UpdateReceived { .. })
    })
    .await;
    match event {
        SyncEvent::UpdateReceived { update_bytes, .. } => {
            assert_eq!(update_bytes, b"should arrive");
        }
        _ => unreachable!(),
    }

    // B disables the project
    engine_b.disable_project(project_id).await.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;

    // A sends another update
    engine_a
        .send_update(project_id, b"should not arrive".to_vec())
        .await
        .unwrap();

    // B should NOT receive it (timed out waiting)
    let result = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            match rx_b.recv().await {
                Ok(SyncEvent::UpdateReceived {
                    project_id: pid, ..
                }) if pid == project_id => return true,
                Ok(_) => continue,
                Err(_) => return false,
            }
        }
    })
    .await;

    match result {
        Err(_) => {} // Timeout — expected
        Ok(got) => assert!(!got, "B should not receive updates after disabling project"),
    }

    engine_a.disconnect().await.unwrap();
    engine_b.disconnect().await.unwrap();
}

// ─── Test 7: Rapid-fire updates all arrive ───────────────────────

#[tokio::test]
async fn rapid_fire_updates() {
    let (addr, state) = start_server().await;

    let (_acct1, dev1, jwt1) = create_test_account(&state, "rapid1@test.com", "Device A");
    let (_acct2, dev2, jwt2) = create_test_account(&state, "rapid2@test.com", "Device B");

    let engine_a = SyncEngine::connect(format!("ws://{addr}"), jwt1, dev1)
        .await
        .unwrap();
    let engine_b = SyncEngine::connect(format!("ws://{addr}"), jwt2, dev2)
        .await
        .unwrap();

    wait_for_status(&engine_a, "Connected", 5).await;
    wait_for_status(&engine_b, "Connected", 5).await;

    let project_id = Uuid::new_v4();
    let shared_key = [88u8; 32];

    engine_a
        .enable_project(project_id, shared_key)
        .await
        .unwrap();
    engine_b
        .enable_project(project_id, shared_key)
        .await
        .unwrap();

    let mut rx_a = engine_a.subscribe();
    let mut rx_b = engine_b.subscribe();

    wait_for_event(&mut rx_a, 5, |e| {
        matches!(e, SyncEvent::ProjectJoined { .. })
    })
    .await;
    wait_for_event(&mut rx_b, 5, |e| {
        matches!(e, SyncEvent::ProjectJoined { .. })
    })
    .await;

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Send 10 updates in rapid succession
    let num_updates = 10;
    for i in 0..num_updates {
        let data = format!("rapid-{i}").into_bytes();
        engine_a.send_update(project_id, data).await.unwrap();
    }

    // B should receive all 10
    let mut received = Vec::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);

    while received.len() < num_updates && tokio::time::Instant::now() < deadline {
        tokio::select! {
            event = rx_b.recv() => {
                if let Ok(SyncEvent::UpdateReceived { update_bytes, .. }) = event {
                    received.push(String::from_utf8(update_bytes).unwrap());
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(50)) => {}
        }
    }

    assert_eq!(
        received.len(),
        num_updates,
        "All {num_updates} updates should arrive, got {}",
        received.len()
    );

    // Verify all content (order may vary due to async)
    for i in 0..num_updates {
        let expected = format!("rapid-{i}");
        assert!(
            received.contains(&expected),
            "Missing update: {expected}. Received: {received:?}"
        );
    }

    engine_a.disconnect().await.unwrap();
    engine_b.disconnect().await.unwrap();
}
