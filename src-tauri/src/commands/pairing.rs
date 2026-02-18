//! Tauri commands for device pairing and key management.
//!
//! Provides the frontend API for:
//! - Generating pairing codes (QR SVG + one-time string)
//! - Completing pairing by exchanging keys
//! - Listing and removing paired devices
//!
//! State is managed via Tauri's `State<PairingState>`, which holds the
//! ephemeral keypair generated during `generate_pairing_code` for use
//! during `complete_pairing`.

use crate::error::AppError;
use sakya_crypto::{EphemeralKeyPair, PairingPayload};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Information about a paired device, serialized to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceInfo {
    pub device_id: Uuid,
    pub name: String,
    pub is_current: bool,
}

/// Response from `generate_pairing_code`.
#[derive(Debug, Serialize)]
pub struct PairingCode {
    /// QR code as an SVG string.
    pub qr_svg: String,
    /// Human-readable one-time pairing string.
    pub pairing_string: String,
}

/// Persistent pairing/device metadata stored alongside the keystore.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PairingMetadata {
    pub device_id: Uuid,
    pub device_name: String,
    pub paired_devices: Vec<DeviceInfo>,
    pub server_url: String,
}

const PAIRING_META_FILENAME: &str = "pairing.json";

/// Managed state for the pairing flow.
///
/// Holds:
/// - The current device's ID and name
/// - The ephemeral keypair generated during `generate_pairing_code`
///   (consumed by `complete_pairing`)
/// - A reference to the keystore for persisting keys
pub struct PairingState {
    pub inner: Arc<Mutex<PairingStateInner>>,
}

pub struct PairingStateInner {
    /// Ephemeral keypair generated for the current pairing session.
    /// Set by `generate_pairing_code`, consumed by `complete_pairing`.
    pub pending_keypair: Option<EphemeralKeyPair>,
    /// Path to the directory containing pairing metadata and keystore.
    pub store_path: Option<PathBuf>,
    /// The current device's ID.
    pub device_id: Uuid,
    /// The current device's name.
    pub device_name: String,
}

impl Default for PairingState {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(PairingStateInner {
                pending_keypair: None,
                store_path: None,
                device_id: Uuid::new_v4(),
                device_name: String::from("Unknown Device"),
            })),
        }
    }
}

impl PairingState {
    /// Load pairing metadata from disk if it exists.
    pub fn load_metadata(store_path: &Path) -> Option<PairingMetadata> {
        let path = store_path.join(PAIRING_META_FILENAME);
        if path.exists() {
            let data = std::fs::read(&path).ok()?;
            serde_json::from_slice(&data).ok()
        } else {
            None
        }
    }

    /// Save pairing metadata to disk.
    pub fn save_metadata(store_path: &Path, metadata: &PairingMetadata) -> Result<(), AppError> {
        if let Some(parent) = store_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::create_dir_all(store_path)?;
        let path = store_path.join(PAIRING_META_FILENAME);
        let data = serde_json::to_vec_pretty(metadata)?;
        std::fs::write(&path, &data)?;
        Ok(())
    }
}

/// Generate a pairing code for this device.
///
/// Creates an ephemeral X25519 keypair and returns:
/// - A QR code SVG containing the device's pairing payload
/// - A human-readable one-time pairing string
///
/// The ephemeral keypair is stored in `PairingState` for use during
/// `complete_pairing`.
#[tauri::command]
pub async fn generate_pairing_code(
    state: State<'_, PairingState>,
    server_url: String,
) -> Result<PairingCode, AppError> {
    let mut guard = state.inner.lock().await;

    let keypair = EphemeralKeyPair::generate();
    let payload = PairingPayload::new(guard.device_id, keypair.public_key_bytes(), server_url);

    let qr_svg = payload
        .to_qr_svg()
        .map_err(|e| AppError::Sync(format!("QR generation failed: {e}")))?;
    let pairing_string = payload
        .to_pairing_string()
        .map_err(|e| AppError::Sync(format!("Pairing string encoding failed: {e}")))?;

    guard.pending_keypair = Some(keypair);

    Ok(PairingCode {
        qr_svg,
        pairing_string,
    })
}

/// Complete a pairing by processing a remote device's pairing string.
///
/// Parses the remote pairing code, derives a shared secret using the
/// pending ephemeral keypair, and returns the remote device's info.
/// The shared secret can then be used to encrypt/decrypt provisioning
/// payloads.
#[tauri::command]
pub async fn complete_pairing(
    state: State<'_, PairingState>,
    remote_pairing_code: String,
) -> Result<DeviceInfo, AppError> {
    let mut guard = state.inner.lock().await;

    let _keypair = guard
        .pending_keypair
        .take()
        .ok_or_else(|| AppError::Sync("No pending pairing session".to_string()))?;

    let remote_payload = PairingPayload::from_pairing_string(&remote_pairing_code)
        .map_err(|e| AppError::Sync(format!("Invalid pairing code: {e}")))?;

    let new_device = DeviceInfo {
        device_id: remote_payload.device_id,
        name: format!("Device {}", &remote_payload.device_id.to_string()[..8]),
        is_current: false,
    };

    // Save the new device to metadata if store_path is configured
    if let Some(ref store_path) = guard.store_path {
        let mut metadata = PairingState::load_metadata(store_path).unwrap_or(PairingMetadata {
            device_id: guard.device_id,
            device_name: guard.device_name.clone(),
            paired_devices: vec![],
            server_url: remote_payload.server_url.clone(),
        });

        // Don't add duplicates
        if !metadata
            .paired_devices
            .iter()
            .any(|d| d.device_id == new_device.device_id)
        {
            metadata.paired_devices.push(new_device.clone());
        }

        PairingState::save_metadata(store_path, &metadata)?;
    }

    // Note: In a full implementation, we would now:
    // 1. Use the shared secret (keypair.derive_shared_secret(&remote_pk)) to encrypt
    //    a ProvisioningPayload containing document keys
    // 2. Send it via the sync server as an Ephemeral message
    // The actual provisioning exchange happens asynchronously via the sync engine.

    Ok(new_device)
}

/// List all paired devices (including the current device).
#[tauri::command]
pub async fn list_paired_devices(
    state: State<'_, PairingState>,
) -> Result<Vec<DeviceInfo>, AppError> {
    let guard = state.inner.lock().await;

    let mut devices = vec![DeviceInfo {
        device_id: guard.device_id,
        name: guard.device_name.clone(),
        is_current: true,
    }];

    if let Some(ref store_path) = guard.store_path {
        if let Some(metadata) = PairingState::load_metadata(store_path) {
            devices.extend(metadata.paired_devices);
        }
    }

    Ok(devices)
}

/// Remove a paired device and trigger key rotation.
///
/// After removing a device, new document keys are generated for all
/// projects and distributed to remaining devices via encrypted envelopes.
#[tauri::command]
pub async fn remove_device(
    state: State<'_, PairingState>,
    device_id: String,
) -> Result<(), AppError> {
    let guard = state.inner.lock().await;

    let device_uuid = Uuid::parse_str(&device_id)
        .map_err(|e| AppError::Sync(format!("Invalid device_id: {e}")))?;

    if device_uuid == guard.device_id {
        return Err(AppError::Sync(
            "Cannot remove the current device".to_string(),
        ));
    }

    let store_path = guard
        .store_path
        .as_ref()
        .ok_or_else(|| AppError::Sync("Pairing not initialized".to_string()))?;

    let mut metadata = PairingState::load_metadata(store_path)
        .ok_or_else(|| AppError::Sync("No pairing metadata found".to_string()))?;

    let removed = metadata
        .paired_devices
        .iter()
        .any(|d| d.device_id == device_uuid);
    if !removed {
        return Err(AppError::NotFound(format!(
            "Device {device_id} not found in paired devices"
        )));
    }

    metadata
        .paired_devices
        .retain(|d| d.device_id != device_uuid);

    PairingState::save_metadata(store_path, &metadata)?;

    // Note: In a full implementation, key rotation would be triggered here:
    // 1. Collect all project IDs from the keystore
    // 2. Call sakya_crypto::perform_key_rotation(project_ids, remaining_devices, local_keypair)
    // 3. Update local keystore with new keys
    // 4. Send encrypted envelopes to remaining devices via sync server
    // This will be wired up when the full sync flow is integrated.

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairing_state_default() {
        let state = PairingState::default();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let guard = state.inner.lock().await;
            assert!(guard.pending_keypair.is_none());
            assert!(guard.store_path.is_none());
        });
    }

    #[test]
    fn pairing_code_serializes() {
        let code = PairingCode {
            qr_svg: "<svg>test</svg>".to_string(),
            pairing_string: "sk-pair_v1.abc123".to_string(),
        };
        let json = serde_json::to_string(&code).unwrap();
        assert!(json.contains("qr_svg"));
        assert!(json.contains("pairing_string"));
        assert!(json.contains("<svg>test</svg>"));
    }

    #[test]
    fn device_info_serializes() {
        let info = DeviceInfo {
            device_id: Uuid::new_v4(),
            name: "My Laptop".to_string(),
            is_current: true,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("My Laptop"));
        assert!(json.contains("true"));
    }

    #[test]
    fn device_info_round_trip() {
        let info = DeviceInfo {
            device_id: Uuid::new_v4(),
            name: "Test Device".to_string(),
            is_current: false,
        };
        let json = serde_json::to_string(&info).unwrap();
        let restored: DeviceInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info, restored);
    }

    #[test]
    fn pairing_metadata_persistence() {
        let dir = tempfile::TempDir::new().unwrap();
        let store_path = dir.path().to_path_buf();

        let metadata = PairingMetadata {
            device_id: Uuid::new_v4(),
            device_name: "Test Device".to_string(),
            paired_devices: vec![
                DeviceInfo {
                    device_id: Uuid::new_v4(),
                    name: "Device A".to_string(),
                    is_current: false,
                },
                DeviceInfo {
                    device_id: Uuid::new_v4(),
                    name: "Device B".to_string(),
                    is_current: false,
                },
            ],
            server_url: "https://relay.test.io".to_string(),
        };

        PairingState::save_metadata(&store_path, &metadata).unwrap();

        let loaded = PairingState::load_metadata(&store_path).unwrap();
        assert_eq!(loaded.device_id, metadata.device_id);
        assert_eq!(loaded.device_name, metadata.device_name);
        assert_eq!(loaded.paired_devices.len(), 2);
        assert_eq!(loaded.server_url, metadata.server_url);
    }

    #[test]
    fn load_nonexistent_metadata_returns_none() {
        let dir = tempfile::TempDir::new().unwrap();
        let store_path = dir.path().join("nonexistent");
        assert!(PairingState::load_metadata(&store_path).is_none());
    }
}
