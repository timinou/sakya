//! SQLite database for authentication state.
//!
//! Stores accounts, devices, and magic link tokens using rusqlite.

use crate::error::AuthError;
use rusqlite::Connection;

/// SQLite-backed authentication database.
pub struct AuthDb {
    conn: Connection,
}

impl AuthDb {
    /// Open or create a database at the given file path and run migrations.
    pub fn new(path: &str) -> Result<Self, AuthError> {
        let conn = Connection::open(path).map_err(|e| AuthError::Database(e.to_string()))?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    /// Create an in-memory database (for testing) and run migrations.
    pub fn new_in_memory() -> Result<Self, AuthError> {
        let conn = Connection::open_in_memory().map_err(|e| AuthError::Database(e.to_string()))?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    /// Run schema migrations (CREATE TABLE IF NOT EXISTS).
    pub fn migrate(&self) -> Result<(), AuthError> {
        self.conn
            .execute_batch(
                "
                CREATE TABLE IF NOT EXISTS accounts (
                    id         TEXT PRIMARY KEY,
                    email      TEXT UNIQUE NOT NULL,
                    created_at TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS devices (
                    id             TEXT PRIMARY KEY,
                    account_id     TEXT NOT NULL REFERENCES accounts(id),
                    name           TEXT NOT NULL,
                    ed25519_pubkey BLOB NOT NULL,
                    created_at     TEXT NOT NULL,
                    last_seen      TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS magic_links (
                    id         TEXT PRIMARY KEY,
                    email      TEXT NOT NULL,
                    token_hash TEXT NOT NULL,
                    expires_at TEXT NOT NULL,
                    used       INTEGER NOT NULL DEFAULT 0,
                    created_at TEXT NOT NULL
                );

                CREATE INDEX IF NOT EXISTS idx_magic_links_token_hash
                    ON magic_links(token_hash);

                CREATE INDEX IF NOT EXISTS idx_accounts_email
                    ON accounts(email);
                ",
            )
            .map_err(|e| AuthError::Database(e.to_string()))?;
        Ok(())
    }

    /// Get a reference to the underlying connection.
    pub fn conn(&self) -> &Connection {
        &self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_db_in_memory() {
        let db = AuthDb::new_in_memory();
        assert!(db.is_ok(), "should create an in-memory database");
    }

    #[test]
    fn migrate_creates_tables() {
        let db = AuthDb::new_in_memory().unwrap();

        // Verify accounts table exists by querying it
        let count: i64 = db
            .conn()
            .query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0))
            .expect("accounts table should exist");
        assert_eq!(count, 0);

        // Verify devices table exists
        let count: i64 = db
            .conn()
            .query_row("SELECT COUNT(*) FROM devices", [], |row| row.get(0))
            .expect("devices table should exist");
        assert_eq!(count, 0);

        // Verify magic_links table exists
        let count: i64 = db
            .conn()
            .query_row("SELECT COUNT(*) FROM magic_links", [], |row| row.get(0))
            .expect("magic_links table should exist");
        assert_eq!(count, 0);
    }

    #[test]
    fn migrate_is_idempotent() {
        let db = AuthDb::new_in_memory().unwrap();

        // Running migrate again should not fail
        let result = db.migrate();
        assert!(result.is_ok(), "migrate should be idempotent");

        // Third time for good measure
        let result = db.migrate();
        assert!(result.is_ok(), "migrate should be idempotent on third call");
    }

    #[test]
    fn file_based_db_creates_and_opens() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        let path_str = path.to_str().unwrap();

        // Create
        let db = AuthDb::new(path_str);
        assert!(db.is_ok(), "should create file-based database");
        drop(db);

        // Re-open
        let db2 = AuthDb::new(path_str);
        assert!(db2.is_ok(), "should re-open existing database");
    }

    #[test]
    fn tables_have_correct_columns() {
        let db = AuthDb::new_in_memory().unwrap();

        // Insert into accounts to verify schema
        db.conn()
            .execute(
                "INSERT INTO accounts (id, email, created_at) VALUES (?1, ?2, ?3)",
                rusqlite::params!["acc-1", "test@example.com", "2026-01-01T00:00:00Z"],
            )
            .expect("should insert into accounts");

        // Insert into devices to verify schema
        db.conn()
            .execute(
                "INSERT INTO devices (id, account_id, name, ed25519_pubkey, created_at, last_seen) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    "dev-1",
                    "acc-1",
                    "My Laptop",
                    vec![0u8; 32],
                    "2026-01-01T00:00:00Z",
                    "2026-01-01T00:00:00Z",
                ],
            )
            .expect("should insert into devices");

        // Insert into magic_links to verify schema
        db.conn()
            .execute(
                "INSERT INTO magic_links (id, email, token_hash, expires_at, used, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    "ml-1",
                    "test@example.com",
                    "deadbeef",
                    "2026-01-01T01:00:00Z",
                    0,
                    "2026-01-01T00:00:00Z",
                ],
            )
            .expect("should insert into magic_links");
    }

    #[test]
    fn email_uniqueness_enforced() {
        let db = AuthDb::new_in_memory().unwrap();

        db.conn()
            .execute(
                "INSERT INTO accounts (id, email, created_at) VALUES (?1, ?2, ?3)",
                rusqlite::params!["acc-1", "dup@example.com", "2026-01-01T00:00:00Z"],
            )
            .unwrap();

        let result = db.conn().execute(
            "INSERT INTO accounts (id, email, created_at) VALUES (?1, ?2, ?3)",
            rusqlite::params!["acc-2", "dup@example.com", "2026-01-01T00:00:00Z"],
        );
        assert!(
            result.is_err(),
            "duplicate email should be rejected by UNIQUE constraint"
        );
    }
}
