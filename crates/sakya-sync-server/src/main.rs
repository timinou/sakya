//! Sakya sync server binary.
//!
//! An Axum-based WebSocket relay server that stores and relays
//! encrypted CRDT updates between Sakya clients.

pub mod error;
pub mod room;
pub mod routes;
pub mod state;
pub mod storage;
pub mod ws;

use crate::routes::build_router;
use crate::state::AppState;
use listenfd::ListenFd;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sakya_sync_server=info,tower_http=info".into()),
        )
        .init();

    // Configuration from environment
    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string());
    let auth_db_path = std::env::var("AUTH_DB_PATH").unwrap_or_else(|_| "auth.db".to_string());
    let sync_db_path = std::env::var("SYNC_DB_PATH").unwrap_or_else(|_| "sync.db".to_string());
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());

    // Build application state
    let state = AppState::new(&jwt_secret, &auth_db_path, &sync_db_path)
        .expect("Failed to initialize application state");

    // Build router: HTTP routes + WebSocket upgrade
    let app = build_router(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // Socket activation or direct bind
    let listener = {
        let mut listenfd = ListenFd::from_env();
        if let Ok(Some(listener)) = listenfd.take_tcp_listener(0) {
            listener.set_nonblocking(true).unwrap();
            TcpListener::from_std(listener).unwrap()
        } else {
            let addr: SocketAddr = bind_addr.parse().expect("Invalid BIND_ADDR");
            TcpListener::bind(addr)
                .await
                .expect("Failed to bind address")
        }
    };

    tracing::info!(
        "Sakya sync server v{} listening on {}",
        env!("CARGO_PKG_VERSION"),
        listener.local_addr().unwrap()
    );

    // Graceful shutdown on SIGTERM
    let shutdown = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        tracing::info!("Shutdown signal received, draining connections...");
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
        .expect("Server error");

    tracing::info!("Server shut down cleanly");
}
