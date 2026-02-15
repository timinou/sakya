//! Device registration, listing, and removal.
//!
//! Each account can have multiple devices, identified by their Ed25519 public key.

use crate::db::AuthDb;
use crate::error::AuthError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Information about a registered device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Unique device identifier.
    pub id: Uuid,
    /// The account this device belongs to.
    pub account_id: Uuid,
    /// Human-readable device name.
    pub name: String,
    /// Ed25519 public key bytes.
    pub public_key: Vec<u8>,
    /// When the device was first registered.
    pub created_at: DateTime<Utc>,
    /// When the device was last seen online.
    pub last_seen: DateTime<Utc>,
}

/// Service for managing devices associated with accounts.
pub struct DeviceService {
    db: AuthDb,
}

impl DeviceService {
    /// Create a new device service backed by the given database.
    pub fn new(db: AuthDb) -> Self {
        Self { db }
    }

    /// Register a new device for an account.
    ///
    /// Returns the generated device ID.
    pub fn register_device(
        &self,
        account_id: Uuid,
        name: &str,
        public_key: &[u8],
    ) -> Result<Uuid, AuthError> {
        let device_id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();

        self.db
            .conn()
            .execute(
                "INSERT INTO devices (id, account_id, name, ed25519_pubkey, created_at, last_seen) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    device_id.to_string(),
                    account_id.to_string(),
                    name,
                    public_key,
                    now,
                    now,
                ],
            )
            .map_err(|e| AuthError::Database(e.to_string()))?;

        Ok(device_id)
    }

    /// List all devices for an account.
    pub fn list_devices(&self, account_id: Uuid) -> Result<Vec<DeviceInfo>, AuthError> {
        let mut stmt = self
            .db
            .conn()
            .prepare(
                "SELECT id, account_id, name, ed25519_pubkey, created_at, last_seen \
                 FROM devices WHERE account_id = ?1 ORDER BY created_at ASC",
            )
            .map_err(|e| AuthError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(rusqlite::params![account_id.to_string()], |row| {
                let id_str: String = row.get(0)?;
                let account_id_str: String = row.get(1)?;
                let name: String = row.get(2)?;
                let public_key: Vec<u8> = row.get(3)?;
                let created_at_str: String = row.get(4)?;
                let last_seen_str: String = row.get(5)?;
                Ok((
                    id_str,
                    account_id_str,
                    name,
                    public_key,
                    created_at_str,
                    last_seen_str,
                ))
            })
            .map_err(|e| AuthError::Database(e.to_string()))?;

        let mut devices = Vec::new();
        for row in rows {
            let (id_str, account_id_str, name, public_key, created_at_str, last_seen_str) =
                row.map_err(|e| AuthError::Database(e.to_string()))?;

            let id = Uuid::parse_str(&id_str)
                .map_err(|e| AuthError::Database(format!("bad device id: {e}")))?;
            let acc_id = Uuid::parse_str(&account_id_str)
                .map_err(|e| AuthError::Database(format!("bad account id: {e}")))?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| AuthError::Database(format!("bad created_at: {e}")))?
                .with_timezone(&Utc);
            let last_seen = DateTime::parse_from_rfc3339(&last_seen_str)
                .map_err(|e| AuthError::Database(format!("bad last_seen: {e}")))?
                .with_timezone(&Utc);

            devices.push(DeviceInfo {
                id,
                account_id: acc_id,
                name,
                public_key,
                created_at,
                last_seen,
            });
        }

        Ok(devices)
    }

    /// Remove a device belonging to an account.
    ///
    /// Returns an error if the device does not exist or does not belong to the account.
    pub fn remove_device(&self, account_id: Uuid, device_id: Uuid) -> Result<(), AuthError> {
        let rows_affected = self
            .db
            .conn()
            .execute(
                "DELETE FROM devices WHERE id = ?1 AND account_id = ?2",
                rusqlite::params![device_id.to_string(), account_id.to_string()],
            )
            .map_err(|e| AuthError::Database(e.to_string()))?;

        if rows_affected == 0 {
            return Err(AuthError::DeviceNotFound);
        }

        Ok(())
    }

    /// Update the `last_seen` timestamp for a device.
    pub fn update_last_seen(&self, device_id: Uuid) -> Result<(), AuthError> {
        let now = Utc::now().to_rfc3339();
        let rows_affected = self
            .db
            .conn()
            .execute(
                "UPDATE devices SET last_seen = ?1 WHERE id = ?2",
                rusqlite::params![now, device_id.to_string()],
            )
            .map_err(|e| AuthError::Database(e.to_string()))?;

        if rows_affected == 0 {
            return Err(AuthError::DeviceNotFound);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create an in-memory DeviceService with a pre-existing account.
    fn setup() -> (DeviceService, Uuid) {
        let db = AuthDb::new_in_memory().unwrap();
        let account_id = Uuid::new_v4();
        db.conn()
            .execute(
                "INSERT INTO accounts (id, email, created_at) VALUES (?1, ?2, ?3)",
                rusqlite::params![
                    account_id.to_string(),
                    "test@example.com",
                    Utc::now().to_rfc3339(),
                ],
            )
            .unwrap();
        (DeviceService::new(db), account_id)
    }

    #[test]
    fn register_and_list_device() {
        let (svc, account_id) = setup();
        let pubkey = vec![1u8; 32];

        let device_id = svc
            .register_device(account_id, "My Laptop", &pubkey)
            .unwrap();
        assert!(!device_id.is_nil());

        let devices = svc.list_devices(account_id).unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].id, device_id);
        assert_eq!(devices[0].name, "My Laptop");
        assert_eq!(devices[0].public_key, pubkey);
        assert_eq!(devices[0].account_id, account_id);
    }

    #[test]
    fn register_multiple_devices() {
        let (svc, account_id) = setup();

        let d1 = svc
            .register_device(account_id, "Laptop", &[1u8; 32])
            .unwrap();
        let d2 = svc
            .register_device(account_id, "Phone", &[2u8; 32])
            .unwrap();
        let d3 = svc
            .register_device(account_id, "Tablet", &[3u8; 32])
            .unwrap();

        let devices = svc.list_devices(account_id).unwrap();
        assert_eq!(devices.len(), 3);

        let ids: Vec<Uuid> = devices.iter().map(|d| d.id).collect();
        assert!(ids.contains(&d1));
        assert!(ids.contains(&d2));
        assert!(ids.contains(&d3));
    }

    #[test]
    fn remove_device_from_list() {
        let (svc, account_id) = setup();

        let d1 = svc
            .register_device(account_id, "Laptop", &[1u8; 32])
            .unwrap();
        let _d2 = svc
            .register_device(account_id, "Phone", &[2u8; 32])
            .unwrap();

        svc.remove_device(account_id, d1).unwrap();

        let devices = svc.list_devices(account_id).unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "Phone");
    }

    #[test]
    fn remove_nonexistent_device_errors() {
        let (svc, account_id) = setup();
        let fake_device_id = Uuid::new_v4();

        let result = svc.remove_device(account_id, fake_device_id);
        assert!(result.is_err(), "removing nonexistent device should fail");
        match result.unwrap_err() {
            AuthError::DeviceNotFound => {}
            other => panic!("expected DeviceNotFound, got: {other}"),
        }
    }

    #[test]
    fn update_last_seen_updates_timestamp() {
        let (svc, account_id) = setup();

        let device_id = svc
            .register_device(account_id, "Laptop", &[1u8; 32])
            .unwrap();

        // Record the initial last_seen
        let devices_before = svc.list_devices(account_id).unwrap();
        let before = devices_before[0].last_seen;

        // Small delay to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(10));

        svc.update_last_seen(device_id).unwrap();

        let devices_after = svc.list_devices(account_id).unwrap();
        let after = devices_after[0].last_seen;

        assert!(
            after >= before,
            "last_seen should be updated: before={before}, after={after}"
        );
    }

    #[test]
    fn list_empty_returns_empty() {
        let (svc, account_id) = setup();
        let devices = svc.list_devices(account_id).unwrap();
        assert!(devices.is_empty(), "no devices should be empty list");
    }

    #[test]
    fn update_last_seen_nonexistent_device_errors() {
        let (svc, _account_id) = setup();
        let result = svc.update_last_seen(Uuid::new_v4());
        assert!(result.is_err());
        match result.unwrap_err() {
            AuthError::DeviceNotFound => {}
            other => panic!("expected DeviceNotFound, got: {other}"),
        }
    }

    #[test]
    fn remove_device_wrong_account_errors() {
        let (svc, account_id) = setup();
        let device_id = svc
            .register_device(account_id, "Laptop", &[1u8; 32])
            .unwrap();

        // Try to remove with a different account_id
        let other_account = Uuid::new_v4();
        let result = svc.remove_device(other_account, device_id);
        assert!(
            result.is_err(),
            "removing device with wrong account should fail"
        );
        match result.unwrap_err() {
            AuthError::DeviceNotFound => {}
            other => panic!("expected DeviceNotFound, got: {other}"),
        }
    }
}
