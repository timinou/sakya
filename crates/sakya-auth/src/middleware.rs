//! Axum middleware for JWT-based authentication.
//!
//! Provides the [`AuthenticatedDevice`] extractor that validates the
//! `Authorization: Bearer <token>` header using [`JwtService`].

use crate::jwt::JwtService;
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Extension;
use http::request::Parts;
use std::sync::Arc;
use uuid::Uuid;

/// An authenticated device extracted from a valid JWT bearer token.
///
/// Use this as an Axum handler parameter to require authentication:
///
/// ```ignore
/// async fn my_handler(device: AuthenticatedDevice) -> impl IntoResponse {
///     format!("Hello, account {}", device.account_id)
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AuthenticatedDevice {
    /// The account identifier from the JWT `sub` claim.
    pub account_id: Uuid,
    /// The device identifier from the JWT `device_id` claim.
    pub device_id: Uuid,
}

/// Error returned when authentication fails.
#[derive(Debug)]
pub struct AuthRejection {
    message: String,
}

impl IntoResponse for AuthRejection {
    fn into_response(self) -> Response {
        (StatusCode::UNAUTHORIZED, self.message).into_response()
    }
}

impl<S> FromRequestParts<S> for AuthenticatedDevice
where
    S: Send + Sync,
{
    type Rejection = AuthRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract the JwtService from the Extension layer
        let Extension(jwt_service) = Extension::<Arc<JwtService>>::from_request_parts(parts, state)
            .await
            .map_err(|_| AuthRejection {
                message: "JWT service not configured".to_string(),
            })?;

        // Get the Authorization header
        let auth_header = parts
            .headers
            .get(http::header::AUTHORIZATION)
            .ok_or_else(|| AuthRejection {
                message: "Missing Authorization header".to_string(),
            })?
            .to_str()
            .map_err(|_| AuthRejection {
                message: "Invalid Authorization header encoding".to_string(),
            })?;

        // Parse "Bearer <token>"
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AuthRejection {
                message: "Authorization header must start with 'Bearer '".to_string(),
            })?;

        // Validate the JWT
        let claims = jwt_service
            .validate_token(token)
            .map_err(|e| AuthRejection {
                message: format!("Authentication failed: {e}"),
            })?;

        Ok(AuthenticatedDevice {
            account_id: claims.sub,
            device_id: claims.device_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::routing::get;
    use axum::Router;
    use http::Request;
    use tower::ServiceExt;

    fn test_app() -> (Router, Arc<JwtService>) {
        let jwt_service = Arc::new(JwtService::new("test-middleware-secret"));
        let app = Router::new()
            .route(
                "/test",
                get(|device: AuthenticatedDevice| async move {
                    format!("{}:{}", device.account_id, device.device_id)
                }),
            )
            .layer(Extension(jwt_service.clone()));
        (app, jwt_service)
    }

    #[tokio::test]
    async fn valid_bearer_token_extracts_device() {
        let (app, jwt_service) = test_app();
        let account_id = Uuid::new_v4();
        let device_id = Uuid::new_v4();
        let token = jwt_service.generate_token(account_id, device_id).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("Authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert_eq!(body_str, format!("{account_id}:{device_id}"));
    }

    #[tokio::test]
    async fn missing_auth_header_returns_401() {
        let (app, _jwt_service) = test_app();

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn invalid_bearer_format_returns_401() {
        let (app, _jwt_service) = test_app();

        // "Basic" instead of "Bearer"
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("Authorization", "Basic abc123")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn expired_token_returns_401() {
        let (app, _) = test_app();

        // Create a JWT service with negative expiry so the token is already expired
        let expired_svc = JwtService::with_expiry("test-middleware-secret", -10);
        let token = expired_svc
            .generate_token(Uuid::new_v4(), Uuid::new_v4())
            .unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("Authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn garbage_token_returns_401() {
        let (app, _jwt_service) = test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("Authorization", "Bearer not.a.real.jwt")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
