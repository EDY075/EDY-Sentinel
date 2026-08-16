use crate::models::SystemOverview;
use chrono::Utc;
use rusqlite::{params, Connection};
use std::{path::PathBuf, sync::Mutex};

const MIGRATIONS: &[(i64, &str)] = &[(1, include_str!("../../migrations/0001_foundation.sql"))];

pub struct Database {
    connection: Mutex<Connection>,
}

impl Database {
    pub fn open(path: PathBuf) -> Result<Self, rusqlite::Error> {
        let mut connection = Connection::open(path)?;
        Self::configure(&connection)?;
        Self::migrate(&mut connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    #[cfg(test)]
    fn in_memory() -> Result<Self, rusqlite::Error> {
        let mut connection = Connection::open_in_memory()?;
        Self::configure(&connection)?;
        Self::migrate(&mut connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    fn configure(connection: &Connection) -> Result<(), rusqlite::Error> {
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 5000;",
        )
    }

    fn migrate(connection: &mut Connection) -> Result<(), rusqlite::Error> {
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL
            );",
        )?;
        for (version, sql) in MIGRATIONS {
            let applied: bool = connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = ?1)",
                [version],
                |row| row.get(0),
            )?;
            if !applied {
                let transaction = connection.transaction()?;
                transaction.execute_batch(sql)?;
                transaction.execute(
                    "INSERT INTO schema_migrations(version, applied_at) VALUES (?1, ?2)",
                    params![version, Utc::now().to_rfc3339()],
                )?;
                transaction.commit()?;
            }
        }
        Ok(())
    }

    pub fn save_snapshot(&self, overview: &SystemOverview) -> Result<(), String> {
        let payload =
            serde_json::to_string(overview).map_err(|_| "Unable to serialize snapshot")?;
        let network =
            serde_json::to_string(&overview.network).map_err(|_| "Unable to serialize network")?;
        let mut connection = self.connection.lock().map_err(|_| "Database unavailable")?;
        let transaction = connection
            .transaction()
            .map_err(|_| "Database unavailable")?;
        transaction
            .execute(
                "INSERT INTO system_snapshots(collected_at, payload_json) VALUES (?1, ?2)",
                params![overview.collected_at, payload],
            )
            .map_err(|_| "Unable to persist system snapshot")?;
        transaction
            .execute(
                "INSERT INTO network_snapshots(collected_at, payload_json) VALUES (?1, ?2)",
                params![overview.collected_at, network],
            )
            .map_err(|_| "Unable to persist network snapshot")?;
        transaction
            .commit()
            .map_err(|_| "Unable to commit snapshot".to_string())
    }

    pub fn get_theme(&self) -> Result<String, String> {
        let connection = self.connection.lock().map_err(|_| "Database unavailable")?;
        Ok(connection
            .query_row(
                "SELECT value FROM settings WHERE key = 'theme'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "sentinel-blue".into()))
    }

    pub fn set_theme(&self, theme: &str) -> Result<(), String> {
        let connection = self.connection.lock().map_err(|_| "Database unavailable")?;
        connection
            .execute(
                "INSERT INTO settings(key, value, updated_at) VALUES ('theme', ?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
                params![theme, Utc::now().to_rfc3339()],
            )
            .map(|_| ())
            .map_err(|_| "Unable to persist theme".into())
    }

    pub fn status(&self) -> Result<(i64, bool), String> {
        let connection = self.connection.lock().map_err(|_| "Database unavailable")?;
        let version = connection
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                [],
                |row| row.get(0),
            )
            .map_err(|_| "Unable to read schema version")?;
        let writable = connection
            .execute(
                "INSERT OR REPLACE INTO settings(key, value, updated_at) VALUES ('database_probe', 'ok', ?1)",
                [Utc::now().to_rfc3339()],
            )
            .is_ok();
        Ok((version, writable))
    }
}

#[cfg(test)]
mod tests {
    use super::Database;

    #[test]
    fn migrations_are_versioned_and_theme_round_trips() {
        let database = Database::in_memory().expect("database should initialize");
        assert_eq!(database.status().expect("status").0, 1);
        database.set_theme("terminal").expect("theme should save");
        assert_eq!(database.get_theme().expect("theme should load"), "terminal");
    }

    #[test]
    fn invalid_alert_severity_is_rejected_by_schema() {
        let database = Database::in_memory().expect("database should initialize");
        let connection = database.connection.lock().expect("connection lock");
        let result = connection.execute(
            "INSERT INTO alerts(id, severity, status, title, created_at) VALUES ('1', 'invented', 'new', 'test', 'now')",
            [],
        );
        assert!(result.is_err());
    }
}
