//! HTTP route assembly.

pub mod auth;
pub mod devices;
pub mod health;

use crate::state::AppState;
use crate::ws;
use axum::routing::{delete, get, post};
use axum::{Extension, Router};

/// Build the complete router (HTTP routes + WebSocket upgrade).
pub fn build_router(state: AppState) -> Router {
    let jwt_service = state.jwt_service.clone();

    Router::new()
        // Public routes
        .route("/health", get(health::health))
        .route("/auth/magic-link", post(auth::request_magic_link))
        .route("/auth/verify", post(auth::verify_magic_link))
        // Protected routes (require JWT via AuthenticatedDevice extractor)
        .route(
            "/devices",
            get(devices::list_devices).post(devices::register_device),
        )
        .route("/devices/{id}", delete(devices::remove_device))
        // WebSocket upgrade
        .route("/sync", get(ws::ws_upgrade))
        .layer(Extension(jwt_service))
        .with_state(state)
}
