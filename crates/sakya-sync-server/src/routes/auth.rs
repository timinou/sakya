//! Authentication HTTP routes (magic link flow).

use crate::error::ServerError;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct MagicLinkRequest {
    pub email: String,
}

#[derive(Serialize)]
pub struct MagicLinkResponse {
    pub message: String,
}

#[derive(Deserialize)]
pub struct VerifyRequest {
    pub token: String,
    pub device_name: String,
    pub public_key: Vec<u8>,
}

#[derive(Serialize)]
pub struct VerifyResponse {
    pub jwt: String,
    pub account_id: Uuid,
    pub device_id: Uuid,
}

/// POST /auth/magic-link
///
/// Sends a magic link to the given email. In the current implementation,
/// the token is returned directly (no email sending yet).
pub async fn request_magic_link(
    State(state): State<AppState>,
    Json(body): Json<MagicLinkRequest>,
) -> Result<Json<MagicLinkResponse>, ServerError> {
    let ml_service = state.magic_link_service.clone();

    let _token = tokio::task::spawn_blocking(move || {
        let svc = ml_service.lock().unwrap();
        svc.create_magic_link(&body.email)
    })
    .await
    .map_err(|e| ServerError::Internal(format!("Task join error: {e}")))?
    .map_err(ServerError::Auth)?;

    Ok(Json(MagicLinkResponse {
        message: "Magic link sent. Check your email.".to_string(),
    }))
}

/// POST /auth/verify
///
/// Verifies a magic link token, registers the device, and returns a JWT.
pub async fn verify_magic_link(
    State(state): State<AppState>,
    Json(body): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, ServerError> {
    let ml_service = state.magic_link_service.clone();
    let token = body.token.clone();

    let (account_id, _email) = tokio::task::spawn_blocking(move || {
        let svc = ml_service.lock().unwrap();
        svc.verify_magic_link(&token)
    })
    .await
    .map_err(|e| ServerError::Internal(format!("Task join error: {e}")))?
    .map_err(ServerError::Auth)?;

    // Register the device
    let dev_service = state.device_service.clone();
    let device_name = body.device_name.clone();
    let public_key = body.public_key.clone();

    let device_id = tokio::task::spawn_blocking(move || {
        let svc = dev_service.lock().unwrap();
        svc.register_device(account_id, &device_name, &public_key)
    })
    .await
    .map_err(|e| ServerError::Internal(format!("Task join error: {e}")))?
    .map_err(ServerError::Auth)?;

    // Generate JWT
    let jwt = state
        .jwt_service
        .generate_token(account_id, device_id)
        .map_err(ServerError::Auth)?;

    Ok(Json(VerifyResponse {
        jwt,
        account_id,
        device_id,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use axum::routing::post;
    use axum::Router;
    use tower::ServiceExt;

    fn test_app() -> Router {
        let state = AppState::new_test("test-auth-secret");
        Router::new()
            .route("/auth/magic-link", post(request_magic_link))
            .route("/auth/verify", post(verify_magic_link))
            .with_state(state)
    }

    #[tokio::test]
    async fn request_magic_link_returns_message() {
        let app = test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/magic-link")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"email":"test@example.com"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(parsed["message"].as_str().unwrap().contains("Magic link"));
    }

    #[tokio::test]
    async fn rate_limit_after_three_requests() {
        let state = AppState::new_test("test-auth-secret");
        let app = Router::new()
            .route("/auth/magic-link", post(request_magic_link))
            .with_state(state);

        for i in 0..4 {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/auth/magic-link")
                        .header("Content-Type", "application/json")
                        .body(Body::from(r#"{"email":"ratelimit@example.com"}"#))
                        .unwrap(),
                )
                .await
                .unwrap();

            if i < 3 {
                assert_eq!(response.status(), 200, "request {i} should succeed");
            } else {
                assert_eq!(response.status(), 429, "request {i} should be rate limited");
            }
        }
    }

    #[tokio::test]
    async fn verify_returns_jwt_and_ids() {
        let state = AppState::new_test("test-auth-secret");

        // First create a magic link by directly using the service
        let token = {
            let svc = state.magic_link_service.lock().unwrap();
            svc.create_magic_link("verify@example.com").unwrap()
        };

        let app = Router::new()
            .route("/auth/verify", post(verify_magic_link))
            .with_state(state);

        let body = serde_json::json!({
            "token": token,
            "device_name": "Test Device",
            "public_key": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
                          17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32]
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/verify")
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
        assert!(parsed["jwt"].is_string());
        assert!(parsed["account_id"].is_string());
        assert!(parsed["device_id"].is_string());
    }

    #[tokio::test]
    async fn verify_invalid_token_returns_401() {
        let app = test_app();

        let body = serde_json::json!({
            "token": "invalid-token",
            "device_name": "Test",
            "public_key": vec![0u8; 32]
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/verify")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), 401);
    }
}
