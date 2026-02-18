//! Tauri commands for CRDT sync operations.
//!
//! These commands bridge the frontend UI to the Rust sync engine,
//! providing connect/disconnect, project enable/disable, and update sending.

use crate::error::AppError;
use sakya_sync_client::{SyncEngine, SyncStatus};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;
use uuid::Uuid;

/// Managed state holding the optional SyncEngine.
pub struct SyncState(pub Arc<Mutex<Option<SyncEngine>>>);

impl Default for SyncState {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }
}

/// Connect to the sync server and start the sync engine.
#[tauri::command]
pub async fn sync_connect(
    app: AppHandle,
    state: State<'_, SyncState>,
    server_url: String,
    jwt_token: String,
    device_id: String,
) -> Result<(), AppError> {
    let device_uuid = Uuid::parse_str(&device_id)
        .map_err(|e| AppError::Sync(format!("Invalid device_id: {e}")))?;

    let engine = SyncEngine::connect(server_url, jwt_token, device_uuid)
        .await
        .map_err(|e| AppError::Sync(e.to_string()))?;

    // Subscribe to events and forward to frontend
    let mut event_rx = engine.subscribe();
    let app_handle = app.clone();
    tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            match event {
                sakya_sync_client::SyncEvent::StatusChanged(status) => {
                    let _ = app_handle.emit("sakya://sync-status-changed", &status);
                }
                sakya_sync_client::SyncEvent::UpdateReceived {
                    project_id,
                    update_bytes,
                } => {
                    let payload = serde_json::json!({
                        "project_id": project_id.to_string(),
                        "update_bytes": update_bytes,
                    });
                    let _ = app_handle.emit("sakya://sync-update-received", payload);
                }
                sakya_sync_client::SyncEvent::ProjectJoined { project_id } => {
                    let payload = serde_json::json!({
                        "project_id": project_id.to_string(),
                    });
                    let _ = app_handle.emit("sakya://sync-project-joined", payload);
                }
                sakya_sync_client::SyncEvent::ProjectError {
                    project_id,
                    message,
                } => {
                    let payload = serde_json::json!({
                        "project_id": project_id.to_string(),
                        "message": message,
                    });
                    let _ = app_handle.emit("sakya://sync-project-error", payload);
                }
            }
        }
    });

    let mut guard = state.0.lock().await;
    *guard = Some(engine);

    Ok(())
}

/// Disconnect from the sync server.
#[tauri::command]
pub async fn sync_disconnect(state: State<'_, SyncState>) -> Result<(), AppError> {
    let mut guard = state.0.lock().await;
    if let Some(engine) = guard.take() {
        engine
            .disconnect()
            .await
            .map_err(|e| AppError::Sync(e.to_string()))?;
    }
    Ok(())
}

/// Get the current sync status.
#[tauri::command]
pub async fn sync_status(state: State<'_, SyncState>) -> Result<SyncStatus, AppError> {
    let guard = state.0.lock().await;
    match guard.as_ref() {
        Some(engine) => Ok(engine.status().await),
        None => Ok(SyncStatus::Disconnected),
    }
}

/// Enable sync for a project (join room with document encryption key).
#[tauri::command]
pub async fn sync_enable_project(
    state: State<'_, SyncState>,
    project_id: String,
    doc_key_bytes: Vec<u8>,
) -> Result<(), AppError> {
    let project_uuid = Uuid::parse_str(&project_id)
        .map_err(|e| AppError::Sync(format!("Invalid project_id: {e}")))?;

    let doc_key: [u8; 32] = doc_key_bytes
        .try_into()
        .map_err(|_| AppError::Sync("Document key must be exactly 32 bytes".to_string()))?;

    let guard = state.0.lock().await;
    let engine = guard
        .as_ref()
        .ok_or_else(|| AppError::Sync("Not connected".to_string()))?;

    engine
        .enable_project(project_uuid, doc_key)
        .await
        .map_err(|e| AppError::Sync(e.to_string()))
}

/// Disable sync for a project (leave room).
#[tauri::command]
pub async fn sync_disable_project(
    state: State<'_, SyncState>,
    project_id: String,
) -> Result<(), AppError> {
    let project_uuid = Uuid::parse_str(&project_id)
        .map_err(|e| AppError::Sync(format!("Invalid project_id: {e}")))?;

    let guard = state.0.lock().await;
    let engine = guard
        .as_ref()
        .ok_or_else(|| AppError::Sync("Not connected".to_string()))?;

    engine
        .disable_project(project_uuid)
        .await
        .map_err(|e| AppError::Sync(e.to_string()))
}

/// Send a CRDT update for a project. The engine encrypts and sends it.
#[tauri::command]
pub async fn sync_send_update(
    state: State<'_, SyncState>,
    project_id: String,
    update_bytes: Vec<u8>,
) -> Result<(), AppError> {
    let project_uuid = Uuid::parse_str(&project_id)
        .map_err(|e| AppError::Sync(format!("Invalid project_id: {e}")))?;

    let guard = state.0.lock().await;
    let engine = guard
        .as_ref()
        .ok_or_else(|| AppError::Sync("Not connected".to_string()))?;

    engine
        .send_update(project_uuid, update_bytes)
        .await
        .map_err(|e| AppError::Sync(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_state_default_is_none() {
        let state = SyncState::default();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let guard = state.0.lock().await;
            assert!(guard.is_none());
        });
    }

    #[test]
    fn sync_status_serializes_for_tauri() {
        let status = SyncStatus::Connected;
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("Connected"));
    }

    #[test]
    fn sync_status_error_serializes() {
        let status = SyncStatus::Error {
            message: "test error".to_string(),
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("test error"));
    }

    #[test]
    fn sync_status_reconnecting_serializes() {
        let status = SyncStatus::Reconnecting { attempt: 5 };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("5"));
    }
}
