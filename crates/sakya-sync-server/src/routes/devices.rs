//! Device management HTTP routes.

use crate::error::ServerError;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use sakya_auth::{AuthenticatedDevice, DeviceInfo};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct RegisterDeviceRequest {
    pub name: String,
    pub public_key: Vec<u8>,
}

#[derive(serde::Serialize)]
pub struct RegisterDeviceResponse {
    pub device_id: Uuid,
}

/// GET /devices — list all devices for the authenticated account.
pub async fn list_devices(
    device: AuthenticatedDevice,
    State(state): State<AppState>,
) -> Result<Json<Vec<DeviceInfo>>, ServerError> {
    let dev_service = state.device_service.clone();
    let account_id = device.account_id;

    let devices = tokio::task::spawn_blocking(move || {
        let svc = dev_service.lock().unwrap();
        svc.list_devices(account_id)
    })
    .await
    .map_err(|e| ServerError::Internal(format!("Task join error: {e}")))?
    .map_err(ServerError::Auth)?;

    Ok(Json(devices))
}

/// POST /devices — register a new device.
pub async fn register_device(
    device: AuthenticatedDevice,
    State(state): State<AppState>,
    Json(body): Json<RegisterDeviceRequest>,
) -> Result<Json<RegisterDeviceResponse>, ServerError> {
    let dev_service = state.device_service.clone();
    let account_id = device.account_id;

    let device_id = tokio::task::spawn_blocking(move || {
        let svc = dev_service.lock().unwrap();
        svc.register_device(account_id, &body.name, &body.public_key)
    })
    .await
    .map_err(|e| ServerError::Internal(format!("Task join error: {e}")))?
    .map_err(ServerError::Auth)?;

    Ok(Json(RegisterDeviceResponse { device_id }))
}

/// DELETE /devices/:id — remove a device.
pub async fn remove_device(
    device: AuthenticatedDevice,
    State(state): State<AppState>,
    Path(device_id): Path<Uuid>,
) -> Result<StatusCode, ServerError> {
    let dev_service = state.device_service.clone();
    let account_id = device.account_id;

    tokio::task::spawn_blocking(move || {
        let svc = dev_service.lock().unwrap();
        svc.remove_device(account_id, device_id)
    })
    .await
    .map_err(|e| ServerError::Internal(format!("Task join error: {e}")))?
    .map_err(ServerError::Auth)?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::{delete, get};
    use axum::{Extension, Router};
    use tower::ServiceExt;

    fn test_app() -> (Router, AppState) {
        let state = AppState::new_test("test-device-secret");
        let jwt = state.jwt_service.clone();
        let app = Router::new()
            .route("/devices", get(list_devices).post(register_device))
            .route("/devices/{id}", delete(remove_device))
            .layer(Extension(jwt))
            .with_state(state.clone());
        (app, state)
    }

    fn create_test_account(state: &AppState) -> (Uuid, Uuid, String) {
        // Create account via magic link
        let token = {
            let svc = state.magic_link_service.lock().unwrap();
            svc.create_magic_link("device-test@example.com").unwrap()
        };
        let (account_id, _) = {
            let svc = state.magic_link_service.lock().unwrap();
            svc.verify_magic_link(&token).unwrap()
        };
        let device_id = {
            let svc = state.device_service.lock().unwrap();
            svc.register_device(account_id, "Initial Device", &[0u8; 32])
                .unwrap()
        };
        let jwt = state
            .jwt_service
            .generate_token(account_id, device_id)
            .unwrap();
        (account_id, device_id, jwt)
    }

    #[tokio::test]
    async fn list_devices_requires_auth() {
        let (app, _state) = test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/devices")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), 401);
    }

    #[tokio::test]
    async fn list_devices_returns_devices() {
        let (app, state) = test_app();
        let (_account_id, _device_id, jwt) = create_test_account(&state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/devices")
                    .header("Authorization", format!("Bearer {jwt}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0]["name"], "Initial Device");
    }

    #[tokio::test]
    async fn register_device_creates_new() {
        let (app, state) = test_app();
        let (_account_id, _device_id, jwt) = create_test_account(&state);

        let body = serde_json::json!({
            "name": "New Phone",
            "public_key": vec![1u8; 32]
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/devices")
                    .header("Authorization", format!("Bearer {jwt}"))
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(parsed["device_id"].is_string());
    }

    #[tokio::test]
    async fn remove_device_returns_204() {
        let (app, state) = test_app();
        let (account_id, _device_id, jwt) = create_test_account(&state);

        // Register another device to remove
        let new_device_id = {
            let svc = state.device_service.lock().unwrap();
            svc.register_device(account_id, "To Remove", &[2u8; 32])
                .unwrap()
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/devices/{new_device_id}"))
                    .header("Authorization", format!("Bearer {jwt}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), 204);
    }
}
