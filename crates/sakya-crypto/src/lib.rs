//! Cryptographic primitives for Sakya E2E encrypted sync.
//!
//! Provides XChaCha20-Poly1305 AEAD encryption, Ed25519 signing,
//! X25519 key exchange, BLAKE2b hash chains, and document key management.

pub mod document_key;
pub mod encryptor;
pub mod error;
pub mod hash_chain;
pub mod key_exchange;
pub mod pairing;
pub mod provisioning;
pub mod rotation;
pub mod signer;

pub use document_key::DocumentKey;
pub use encryptor::{EncryptedEnvelope, Encryptor, XChaCha20Encryptor};
pub use error::CryptoError;
pub use hash_chain::HashChain;
pub use key_exchange::EphemeralKeyPair;
pub use pairing::PairingPayload;
pub use provisioning::ProvisioningPayload;
pub use rotation::{perform_key_rotation, KeyRotationBundle};
pub use signer::{DeviceKeyPair, SignedMessage};
