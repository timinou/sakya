//! Server error types with Axum response integration.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sakya_auth::AuthError;
use sakya_sync_protocol::ProtocolError;

/// Unified error type for the sync server.
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("Auth error: {0}")]
    Auth(#[from] AuthError),

    #[error("Protocol error: {0}")]
    Protocol(#[from] ProtocolError),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

/// JSON error body returned to clients.
#[derive(serde::Serialize)]
struct ErrorBody {
    error: String,
    code: u16,
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ServerError::Auth(e) => match e {
                AuthError::InvalidToken(_) | AuthError::TokenExpired => {
                    (StatusCode::UNAUTHORIZED, self.to_string())
                }
                AuthError::AccountNotFound | AuthError::DeviceNotFound => {
                    (StatusCode::NOT_FOUND, self.to_string())
                }
                AuthError::RateLimited(_) => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            },
            ServerError::Protocol(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ServerError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ServerError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
        };

        let body = ErrorBody {
            error: message,
            code: status.as_u16(),
        };

        (status, axum::Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    async fn status_of(err: ServerError) -> StatusCode {
        let response: Response = err.into_response();
        response.status()
    }

    #[tokio::test]
    async fn auth_invalid_token_returns_401() {
        let err = ServerError::Auth(AuthError::InvalidToken("bad".into()));
        assert_eq!(status_of(err).await, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn auth_expired_returns_401() {
        let err = ServerError::Auth(AuthError::TokenExpired);
        assert_eq!(status_of(err).await, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn auth_not_found_returns_404() {
        let err = ServerError::Auth(AuthError::DeviceNotFound);
        assert_eq!(status_of(err).await, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn auth_rate_limited_returns_429() {
        let err = ServerError::Auth(AuthError::RateLimited("too fast".into()));
        assert_eq!(status_of(err).await, StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn protocol_error_returns_400() {
        let err = ServerError::Protocol(ProtocolError::SerializationError("bad json".into()));
        assert_eq!(status_of(err).await, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn internal_error_returns_500() {
        let err = ServerError::Internal("something broke".into());
        assert_eq!(status_of(err).await, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn not_found_returns_404() {
        let err = ServerError::NotFound("missing".into());
        assert_eq!(status_of(err).await, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn response_body_is_json() {
        let err = ServerError::Internal("boom".into());
        let response = err.into_response();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(parsed["code"], 500);
        assert!(parsed["error"].as_str().unwrap().contains("boom"));
    }
}
