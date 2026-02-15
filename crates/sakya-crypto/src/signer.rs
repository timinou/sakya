//! Ed25519 digital signatures for device identity.
//!
//! Provides [`DeviceKeyPair`] for signing data and [`SignedMessage`] for
//! verifiable message payloads.

use crate::error::{CryptoError, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use uuid::Uuid;

/// A signed message containing the payload, signature, and signer's public key.
#[derive(Debug, Clone)]
pub struct SignedMessage {
    /// The original payload bytes.
    pub payload: Vec<u8>,
    /// The Ed25519 signature over the payload.
    pub signature: Signature,
    /// The public key of the signer, used for verification.
    pub signer_public_key: VerifyingKey,
}

impl SignedMessage {
    /// Verify this signed message using the embedded public key.
    ///
    /// Returns `Ok(())` if the signature is valid, or
    /// [`CryptoError::VerificationFailed`] otherwise.
    pub fn verify(&self) -> Result<()> {
        self.signer_public_key
            .verify(&self.payload, &self.signature)
            .map_err(|_| CryptoError::VerificationFailed)
    }
}

/// An Ed25519 keypair associated with a device identity.
///
/// Used to sign CRDT updates so that peers can verify authorship.
pub struct DeviceKeyPair {
    signing_key: SigningKey,
    device_id: Uuid,
}

impl DeviceKeyPair {
    /// Generate a new random keypair with a random device ID.
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let device_id = Uuid::new_v4();
        Self {
            signing_key,
            device_id,
        }
    }

    /// Sign arbitrary data, returning a [`SignedMessage`].
    pub fn sign(&self, data: &[u8]) -> SignedMessage {
        let signature = self.signing_key.sign(data);
        SignedMessage {
            payload: data.to_vec(),
            signature,
            signer_public_key: self.signing_key.verifying_key(),
        }
    }

    /// Get the public verifying key.
    pub fn public_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// Get the device identifier.
    pub fn device_id(&self) -> &Uuid {
        &self.device_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_verify_round_trip() {
        let kp = DeviceKeyPair::generate();
        let data = b"important message";

        let signed = kp.sign(data);
        let result = signed.verify();

        assert!(
            result.is_ok(),
            "verification of valid signature should succeed"
        );
    }

    #[test]
    fn wrong_key_rejects() {
        let kp_a = DeviceKeyPair::generate();
        let kp_b = DeviceKeyPair::generate();
        let data = b"important message";

        let mut signed = kp_a.sign(data);
        // Replace the signer's public key with B's key
        signed.signer_public_key = kp_b.public_key();

        let result = signed.verify();
        assert!(
            result.is_err(),
            "verification with wrong public key should fail"
        );
    }

    #[test]
    fn tampered_message_rejects() {
        let kp = DeviceKeyPair::generate();
        let data = b"important message";

        let mut signed = kp.sign(data);
        // Tamper with the payload
        signed.payload = b"tampered message".to_vec();

        let result = signed.verify();
        assert!(
            result.is_err(),
            "verification of tampered message should fail"
        );
    }

    #[test]
    fn public_key_extraction() {
        let kp = DeviceKeyPair::generate();
        let pk = kp.public_key();

        // Sign and verify using the extracted public key
        let signed = kp.sign(b"test");
        assert_eq!(
            signed.signer_public_key, pk,
            "signed message should contain the same public key"
        );
    }

    #[test]
    fn different_messages_different_signatures() {
        let kp = DeviceKeyPair::generate();

        let signed_a = kp.sign(b"message A");
        let signed_b = kp.sign(b"message B");

        assert_ne!(
            signed_a.signature.to_bytes(),
            signed_b.signature.to_bytes(),
            "different messages should produce different signatures"
        );
    }

    #[test]
    fn empty_message_sign_verify() {
        let kp = DeviceKeyPair::generate();

        let signed = kp.sign(b"");
        let result = signed.verify();

        assert!(
            result.is_ok(),
            "signing and verifying empty data should succeed"
        );
    }
}
