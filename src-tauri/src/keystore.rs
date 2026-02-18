//! Encrypted local key storage for Sakya.
//!
//! Provides [`Keystore`] for securely persisting cryptographic keys to disk.
//! All data is encrypted at rest using XChaCha20-Poly1305 via [`sakya_crypto::XChaCha20Encryptor`],
//! keyed by a device-specific master key. The keystore file (`keys.enc`) contains a single
//! [`sakya_crypto::EncryptedEnvelope`] wrapping JSON-serialized [`KeystoreData`].

use std::path::PathBuf;

use sakya_crypto::{EncryptedEnvelope, Encryptor, XChaCha20Encryptor};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// AAD string used for all keystore encryption/decryption operations.
const KEYSTORE_AAD: &[u8] = b"sakya-keystore";

/// Filename for the encrypted keystore on disk.
const KEYSTORE_FILENAME: &str = "keys.enc";

/// In-memory representation of stored keys for a project.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ProjectKeys {
    pub project_id: Uuid,
    /// 32-byte document encryption key.
    pub document_key: Vec<u8>,
}

/// Serializable container for all stored keys.
///
/// This is the plaintext structure that gets encrypted before writing to disk.
#[derive(Debug, Serialize, Deserialize, Default)]
struct KeystoreData {
    /// Ed25519 device keypair bytes (opaque to the keystore).
    device_keypair: Option<Vec<u8>>,
    /// X25519 keypair bytes for key exchange (opaque to the keystore).
    exchange_keypair: Option<Vec<u8>>,
    /// Per-project document encryption keys.
    project_keys: Vec<ProjectKeys>,
    /// JWT token for server authentication.
    jwt_token: Option<String>,
    /// Account ID.
    account_id: Option<Uuid>,
}

/// Errors specific to keystore operations.
#[derive(Debug, thiserror::Error)]
pub enum KeystoreError {
    /// Failed to read or write the keystore file.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Failed to serialize or deserialize keystore data.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Failed to encrypt or decrypt the keystore (e.g., wrong master key).
    #[error("Crypto error: {0}")]
    Crypto(String),
}

impl From<sakya_crypto::CryptoError> for KeystoreError {
    fn from(e: sakya_crypto::CryptoError) -> Self {
        KeystoreError::Crypto(e.to_string())
    }
}

/// Result type alias for keystore operations.
pub type Result<T> = std::result::Result<T, KeystoreError>;

/// Manages encrypted storage of cryptographic keys on disk.
///
/// Keys are encrypted with a master key derived from a device-specific secret.
/// The keystore file is a single `EncryptedEnvelope` containing JSON-serialized
/// `KeystoreData`, authenticated with the AAD "sakya-keystore".
///
/// Thread safety: `Keystore` is not internally synchronized. In Tauri, wrap it
/// in `Mutex<Keystore>` via managed state.
pub struct Keystore {
    store_path: PathBuf,
    master_key: [u8; 32],
}

impl Keystore {
    /// Create a new keystore backed by a file at `store_path/keys.enc`.
    ///
    /// The `master_key` is used to encrypt/decrypt all stored data.
    pub fn new(store_path: PathBuf, master_key: [u8; 32]) -> Self {
        Self {
            store_path,
            master_key,
        }
    }

    /// Path to the encrypted keystore file.
    fn file_path(&self) -> PathBuf {
        self.store_path.join(KEYSTORE_FILENAME)
    }

    /// Load and decrypt the keystore data from disk.
    ///
    /// Returns `KeystoreData::default()` if the file does not exist.
    /// Returns an error if the file exists but cannot be decrypted.
    fn load_data(&self) -> Result<KeystoreData> {
        let path = self.file_path();
        if !path.exists() {
            return Ok(KeystoreData::default());
        }

        let raw = std::fs::read(&path)?;
        let envelope: EncryptedEnvelope = serde_json::from_slice(&raw)?;

        let encryptor = XChaCha20Encryptor::new(self.master_key);
        let plaintext = encryptor.decrypt(&envelope)?;

        let data: KeystoreData = serde_json::from_slice(&plaintext)?;
        Ok(data)
    }

    /// Encrypt and write keystore data to disk.
    ///
    /// Creates parent directories if they do not exist.
    fn save_data(&self, data: &KeystoreData) -> Result<()> {
        let plaintext = serde_json::to_vec(data)?;

        let encryptor = XChaCha20Encryptor::new(self.master_key);
        let envelope = encryptor.encrypt(&plaintext, KEYSTORE_AAD)?;

        let path = self.file_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let serialized = serde_json::to_vec(&envelope)?;
        std::fs::write(&path, &serialized)?;
        Ok(())
    }

    /// Save a device keypair (as opaque bytes) to the keystore.
    pub fn save_device_keypair(&self, keypair_bytes: &[u8]) -> Result<()> {
        let mut data = self.load_data()?;
        data.device_keypair = Some(keypair_bytes.to_vec());
        self.save_data(&data)
    }

    /// Load the device keypair bytes from the keystore.
    ///
    /// Returns `None` if no device keypair has been stored.
    pub fn load_device_keypair(&self) -> Result<Option<Vec<u8>>> {
        let data = self.load_data()?;
        Ok(data.device_keypair)
    }

    /// Save a per-project document encryption key (upsert).
    ///
    /// If a key already exists for `project_id`, it is replaced.
    pub fn save_project_key(&self, project_id: Uuid, key: &[u8; 32]) -> Result<()> {
        let mut data = self.load_data()?;
        if let Some(existing) = data
            .project_keys
            .iter_mut()
            .find(|pk| pk.project_id == project_id)
        {
            existing.document_key = key.to_vec();
        } else {
            data.project_keys.push(ProjectKeys {
                project_id,
                document_key: key.to_vec(),
            });
        }
        self.save_data(&data)
    }

    /// Load a per-project document encryption key.
    ///
    /// Returns `None` if no key exists for the given `project_id`.
    pub fn load_project_key(&self, project_id: &Uuid) -> Result<Option<Vec<u8>>> {
        let data = self.load_data()?;
        Ok(data
            .project_keys
            .iter()
            .find(|pk| &pk.project_id == project_id)
            .map(|pk| pk.document_key.clone()))
    }

    /// Remove a per-project document encryption key.
    ///
    /// No-op if the project key does not exist.
    pub fn remove_project_key(&self, project_id: &Uuid) -> Result<()> {
        let mut data = self.load_data()?;
        data.project_keys.retain(|pk| &pk.project_id != project_id);
        self.save_data(&data)
    }

    /// Save a JWT token for server authentication.
    pub fn save_jwt_token(&self, token: &str) -> Result<()> {
        let mut data = self.load_data()?;
        data.jwt_token = Some(token.to_string());
        self.save_data(&data)
    }

    /// Load the JWT token from the keystore.
    ///
    /// Returns `None` if no token has been stored.
    pub fn load_jwt_token(&self) -> Result<Option<String>> {
        let data = self.load_data()?;
        Ok(data.jwt_token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Helper: generate a random 32-byte master key.
    fn random_key() -> [u8; 32] {
        use rand::RngCore;
        let mut key = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut key);
        key
    }

    /// Helper: create a Keystore backed by a temporary directory.
    fn temp_keystore() -> (Keystore, TempDir) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let master_key = random_key();
        let ks = Keystore::new(dir.path().to_path_buf(), master_key);
        (ks, dir)
    }

    #[test]
    fn store_and_load_device_keypair() {
        let (ks, _dir) = temp_keystore();

        // Simulate a 64-byte Ed25519 keypair
        let keypair_bytes: Vec<u8> = (0..64).collect();
        ks.save_device_keypair(&keypair_bytes)
            .expect("save should succeed");

        let loaded = ks
            .load_device_keypair()
            .expect("load should succeed")
            .expect("keypair should be Some");

        assert_eq!(
            loaded, keypair_bytes,
            "loaded keypair should match saved keypair"
        );
    }

    #[test]
    fn store_and_load_project_key() {
        let (ks, _dir) = temp_keystore();

        let project_id = Uuid::new_v4();
        let key: [u8; 32] = [0xAB; 32];
        ks.save_project_key(project_id, &key)
            .expect("save project key should succeed");

        let loaded = ks
            .load_project_key(&project_id)
            .expect("load should succeed")
            .expect("project key should be Some");

        assert_eq!(
            loaded,
            key.to_vec(),
            "loaded project key should match saved key"
        );
    }

    #[test]
    fn encrypted_at_rest() {
        let (ks, dir) = temp_keystore();

        // Store recognizable plaintext data
        let keypair_bytes = b"THIS_IS_A_VERY_RECOGNIZABLE_DEVICE_KEYPAIR_STRING_1234567890!!!!";
        ks.save_device_keypair(keypair_bytes)
            .expect("save should succeed");

        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test.signature";
        ks.save_jwt_token(jwt).expect("save jwt should succeed");

        // Read the raw file bytes
        let file_path = dir.path().join(KEYSTORE_FILENAME);
        let raw_bytes = std::fs::read(&file_path).expect("file should exist");
        let raw_str = String::from_utf8_lossy(&raw_bytes);

        // The plaintext strings should NOT appear in the raw file
        assert!(
            !raw_str.contains("THIS_IS_A_VERY_RECOGNIZABLE_DEVICE_KEYPAIR_STRING"),
            "device keypair plaintext should not appear in encrypted file"
        );
        assert!(
            !raw_str.contains("eyJhbGciOiJIUzI1NiI"),
            "JWT plaintext should not appear in encrypted file"
        );
        assert!(
            !raw_str.contains("device_keypair"),
            "JSON field names should not appear in encrypted file"
        );
    }

    #[test]
    fn missing_key_returns_none() {
        let (ks, _dir) = temp_keystore();

        // No file exists yet -- all loads should return None
        let device = ks
            .load_device_keypair()
            .expect("load should succeed on empty keystore");
        assert!(device.is_none(), "device keypair should be None");

        let project = ks
            .load_project_key(&Uuid::new_v4())
            .expect("load should succeed on empty keystore");
        assert!(project.is_none(), "project key should be None");

        let jwt = ks
            .load_jwt_token()
            .expect("load should succeed on empty keystore");
        assert!(jwt.is_none(), "jwt token should be None");
    }

    #[test]
    fn wrong_master_key_fails() {
        let dir = TempDir::new().expect("failed to create temp dir");
        let key_a = random_key();
        let key_b = random_key();

        // Save with key A
        let ks_a = Keystore::new(dir.path().to_path_buf(), key_a);
        ks_a.save_device_keypair(&[1u8; 64])
            .expect("save should succeed");

        // Try to load with key B
        let ks_b = Keystore::new(dir.path().to_path_buf(), key_b);
        let result = ks_b.load_device_keypair();

        assert!(result.is_err(), "loading with wrong master key should fail");

        // Verify it's a crypto error
        let err = result.unwrap_err();
        match err {
            KeystoreError::Crypto(_) => {} // expected
            other => panic!("expected Crypto error, got: {other:?}"),
        }
    }

    #[test]
    fn upsert_project_key() {
        let (ks, _dir) = temp_keystore();

        let project_id = Uuid::new_v4();

        // Save initial key
        let key_v1: [u8; 32] = [0x01; 32];
        ks.save_project_key(project_id, &key_v1)
            .expect("save v1 should succeed");

        // Overwrite with new key
        let key_v2: [u8; 32] = [0x02; 32];
        ks.save_project_key(project_id, &key_v2)
            .expect("save v2 should succeed");

        // Load should return v2
        let loaded = ks
            .load_project_key(&project_id)
            .expect("load should succeed")
            .expect("project key should be Some");

        assert_eq!(
            loaded,
            key_v2.to_vec(),
            "upsert should overwrite existing key"
        );
    }

    #[test]
    fn remove_project_key() {
        let (ks, _dir) = temp_keystore();

        let project_id = Uuid::new_v4();
        let key: [u8; 32] = [0xCC; 32];

        // Save and verify it exists
        ks.save_project_key(project_id, &key)
            .expect("save should succeed");
        assert!(ks.load_project_key(&project_id).unwrap().is_some());

        // Remove and verify it's gone
        ks.remove_project_key(&project_id)
            .expect("remove should succeed");
        let loaded = ks
            .load_project_key(&project_id)
            .expect("load after remove should succeed");
        assert!(loaded.is_none(), "removed project key should be None");
    }

    #[test]
    fn remove_nonexistent_project_key_is_noop() {
        let (ks, _dir) = temp_keystore();

        // Should not error even though keystore file doesn't exist yet
        let result = ks.remove_project_key(&Uuid::new_v4());
        assert!(result.is_ok(), "removing nonexistent key should not error");
    }

    #[test]
    fn jwt_token_round_trip() {
        let (ks, _dir) = temp_keystore();

        let token = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0In0.abc123"; // pragma: allowlist secret
        ks.save_jwt_token(token).expect("save jwt should succeed");

        let loaded = ks
            .load_jwt_token()
            .expect("load jwt should succeed")
            .expect("jwt should be Some");

        assert_eq!(loaded, token, "loaded JWT should match saved JWT");
    }

    #[test]
    fn multiple_project_keys_independent() {
        let (ks, _dir) = temp_keystore();

        let id_a = Uuid::new_v4();
        let id_b = Uuid::new_v4();
        let key_a: [u8; 32] = [0xAA; 32];
        let key_b: [u8; 32] = [0xBB; 32];

        ks.save_project_key(id_a, &key_a).unwrap();
        ks.save_project_key(id_b, &key_b).unwrap();

        let loaded_a = ks.load_project_key(&id_a).unwrap().unwrap();
        let loaded_b = ks.load_project_key(&id_b).unwrap().unwrap();

        assert_eq!(loaded_a, key_a.to_vec());
        assert_eq!(loaded_b, key_b.to_vec());

        // Remove A, B should still exist
        ks.remove_project_key(&id_a).unwrap();
        assert!(ks.load_project_key(&id_a).unwrap().is_none());
        assert!(ks.load_project_key(&id_b).unwrap().is_some());
    }
}
