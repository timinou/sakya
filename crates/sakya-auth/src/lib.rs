//! Authentication for Sakya sync server.
//!
//! Provides magic link email auth, JWT token management,
//! device registration, and Axum middleware.

pub mod db;
pub mod device;
pub mod error;
pub mod jwt;
pub mod magic_link;
pub mod middleware;

pub use db::AuthDb;
pub use device::{DeviceInfo, DeviceService};
pub use error::AuthError;
pub use jwt::{Claims, JwtService};
pub use magic_link::MagicLinkService;
pub use middleware::AuthenticatedDevice;
