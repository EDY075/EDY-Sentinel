use crate::models::{LiveTelemetrySnapshot, SystemOverview};
use chrono::{Duration, Utc};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration as StdDuration, Instant},
};

const MIGRATIONS: &[(i64, &str)] = &[
    (1, include_str!("../../migrations/0001_foundation.sql")),
    (2, include_str!("../../migrations/0002_live_telemetry.sql")),
    (
        3,
        include_str!("../../migrations/0003_telemetry_accuracy.sql"),
    ),
    (
        4,
        include_str!("../../migrations/0004_behavioral_baseline.sql"),
    ),
    (
        5,
        include_str!("../../migrations/0005_sprint2_hardening.sql"),
    ),
    (
        6,
        include_str!("../../migrations/0006_detection_engine.sql"),
    ),
    (
        7,
        include_str!("../../migrations/0007_software_inventory_vulnerability_repository.sql"),
    ),
    (
        8,
        include_str!("../../migrations/0008_vulnerability_matching.sql"),
    ),
    (
        9,
        include_str!("../../migrations/0009_product_identity.sql"),
    ),
    (
        10,
        include_str!("../../migrations/0010_vulnerability_score_v2.sql"),
    ),
    (
        11,
        include_str!("../../migrations/0011_vulnerability_history_retention.sql"),
    ),
];

const VULNERABILITY_RETENTION_POLICY_KEY: &str = "vulnerability-evaluation-history";
const VULNERABILITY_RETENTION_POLICY_VERSION: i64 = 1;
const VULNERABILITY_RETENTION_INTERVAL_HOURS: i64 = 24;
const VULNERABILITY_RETENTION_RECENT_PER_SOFTWARE: i64 = 100;
const VULNERABILITY_RETENTION_DAILY_DAYS: i64 = 90;
const VULNERABILITY_RETENTION_MONTHLY_DAYS: i64 = 730;
const VULNERABILITY_RETENTION_BATCH_SIZE: i64 = 250;

#[derive(Clone)]
pub struct Database {
    connection: Arc<Mutex<Connection>>,
    cadence: Arc<Mutex<PersistenceCadence>>,
}

#[derive(Debug)]
pub(crate) struct BaselineTransactionFailure {
    pub message: String,
    pub persist_active_error: bool,
}

#[derive(Default)]
struct PersistenceCadence {
    last_system_snapshot: Option<Instant>,
    last_live_write: Option<Instant>,
    last_cleanup: Option<Instant>,
    process_counts: HashMap<String, u64>,
    connection_counts: HashMap<String, u64>,
    service_counts: HashMap<String, u64>,
}

impl Database {
    pub fn open(path: PathBuf) -> Result<Self, rusqlite::Error> {
        let mut connection = Connection::open(path)?;
        Self::configure(&connection)?;
        Self::migrate(&mut connection)?;
        Self::run_vulnerability_history_retention(&mut connection, Utc::now(), false)?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
            cadence: Arc::new(Mutex::new(PersistenceCadence::default())),
        })
    }

    #[cfg(test)]
    pub(crate) fn in_memory() -> Result<Self, rusqlite::Error> {
        let mut connection = Connection::open_in_memory()?;
        Self::configure(&connection)?;
        Self::migrate(&mut connection)?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
            cadence: Arc::new(Mutex::new(PersistenceCadence::default())),
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

    fn run_vulnerability_history_retention(
        connection: &mut Connection,
        now: chrono::DateTime<Utc>,
        force: bool,
    ) -> Result<usize, rusqlite::Error> {
        let transaction = connection.transaction()?;
        let deleted = enforce_vulnerability_history_retention(&transaction, now, force)?;
        transaction.commit()?;
        Ok(deleted)
    }

    pub fn save_snapshot(&self, overview: &SystemOverview) -> Result<(), String> {
        let mut cadence = self.cadence.lock().map_err(|_| "Database unavailable")?;
        if cadence
            .last_system_snapshot
            .is_some_and(|last| last.elapsed() < StdDuration::from_secs(300))
        {
            return Ok(());
        }
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
            .map_err(|_| "Unable to commit snapshot".to_string())?;
        cadence.last_system_snapshot = Some(Instant::now());
        Ok(())
    }

    pub fn persist_live_telemetry(
        &self,
        snapshot: &mut LiveTelemetrySnapshot,
    ) -> Result<(), String> {
        let mut cadence = self.cadence.lock().map_err(|_| "Database unavailable")?;
        let write_due = cadence
            .last_live_write
            .map_or(true, |last| last.elapsed() >= StdDuration::from_secs(60));
        if !write_due && snapshot.events.is_empty() {
            return Ok(());
        }
        let cleanup_due = cadence.last_cleanup.map_or(true, |last| {
            last.elapsed() >= StdDuration::from_secs(6 * 60 * 60)
        });
        let engine_process_counts: HashMap<_, _> = snapshot
            .processes
            .iter()
            .map(|item| (item.key.clone(), item.observation_count))
            .collect();
        let engine_connection_counts: HashMap<_, _> = snapshot
            .connections
            .iter()
            .map(|item| (item.key.clone(), item.observation_count))
            .collect();
        let engine_service_counts: HashMap<_, _> = snapshot
            .services
            .iter()
            .map(|item| (item.key.clone(), item.observation_count))
            .collect();
        let mut connection = self.connection.lock().map_err(|_| "Database unavailable")?;
        let transaction = connection
            .transaction()
            .map_err(|_| "Database unavailable")?;

        let process_tracking = load_tracking(&transaction, "process_observations", "process_key")?;
        let connection_tracking =
            load_tracking(&transaction, "connection_observations", "connection_key")?;
        let service_tracking = load_tracking(&transaction, "service_observations", "service_key")?;

        transaction
            .execute("UPDATE process_observations SET active = 0", [])
            .map_err(|_| "Unable to update process observations")?;
        transaction
            .execute("UPDATE connection_observations SET active = 0", [])
            .map_err(|_| "Unable to update connection observations")?;
        transaction
            .execute("UPDATE service_observations SET active = 0", [])
            .map_err(|_| "Unable to update service observations")?;

        for process in &mut snapshot.processes {
            apply_tracking(
                &mut process.first_seen,
                &mut process.observation_count,
                process.active,
                process_tracking.get(&process.key),
                cadence.process_counts.get(&process.key).copied(),
            );
            transaction
                .execute(
                    "INSERT INTO process_observations(
                        process_key, pid, process_name, parent_pid, user_name, executable_path,
                        start_time, first_seen_at, last_seen_at, observation_count, active,
                        access_status, company_name, signature_status, signer_name
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
                     ON CONFLICT(process_key) DO UPDATE SET
                        pid = excluded.pid,
                        process_name = excluded.process_name,
                        parent_pid = excluded.parent_pid,
                        user_name = excluded.user_name,
                        executable_path = excluded.executable_path,
                        start_time = excluded.start_time,
                        first_seen_at = excluded.first_seen_at,
                        last_seen_at = excluded.last_seen_at,
                        observation_count = excluded.observation_count,
                        active = excluded.active,
                        access_status = excluded.access_status,
                        company_name = excluded.company_name,
                        signature_status = excluded.signature_status,
                        signer_name = excluded.signer_name",
                    params![
                        process.key,
                        process.pid,
                        process.name,
                        process.parent_pid,
                        process.user,
                        process.executable_path,
                        process.start_time,
                        process.first_seen,
                        process.last_seen,
                        process.observation_count,
                        process.active as i64,
                        process.access_status,
                        process.company,
                        process.signature_status,
                        process.signer,
                    ],
                )
                .map_err(|_| "Unable to persist process observations")?;
        }

        for item in &mut snapshot.connections {
            apply_tracking(
                &mut item.first_seen,
                &mut item.observation_count,
                item.active,
                connection_tracking.get(&item.key),
                cadence.connection_counts.get(&item.key).copied(),
            );
            transaction
                .execute(
                    "INSERT INTO connection_observations(
                        connection_key, protocol, ip_version, process_id, local_address, local_port,
                        remote_address, remote_port, tcp_state, first_seen_at, last_seen_at,
                        observation_count, active, association_status, process_last_seen
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
                     ON CONFLICT(connection_key) DO UPDATE SET
                        process_id = excluded.process_id,
                        local_address = excluded.local_address,
                        local_port = excluded.local_port,
                        remote_address = excluded.remote_address,
                        remote_port = excluded.remote_port,
                        tcp_state = excluded.tcp_state,
                        first_seen_at = excluded.first_seen_at,
                        last_seen_at = excluded.last_seen_at,
                        observation_count = excluded.observation_count,
                        active = excluded.active,
                        association_status = excluded.association_status,
                        process_last_seen = excluded.process_last_seen",
                    params![
                        item.key,
                        item.protocol,
                        item.ip_version,
                        item.pid,
                        item.local_address,
                        item.local_port,
                        item.remote_address,
                        item.remote_port,
                        item.state,
                        item.first_seen,
                        item.last_seen,
                        item.observation_count,
                        item.active as i64,
                        item.association_status,
                        item.process_last_seen,
                    ],
                )
                .map_err(|_| "Unable to persist connection observations")?;
        }

        for service in &mut snapshot.services {
            apply_tracking(
                &mut service.first_seen,
                &mut service.observation_count,
                service.active,
                service_tracking.get(&service.key),
                cadence.service_counts.get(&service.key).copied(),
            );
            transaction
                .execute(
                    "INSERT INTO service_observations(
                        service_key, service_name, display_name, binary_path, account_name,
                        process_id, first_seen_at, last_seen_at, observation_count, active,
                        status, startup_type
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                     ON CONFLICT(service_key) DO UPDATE SET
                        service_name = excluded.service_name,
                        display_name = excluded.display_name,
                        binary_path = excluded.binary_path,
                        account_name = excluded.account_name,
                        process_id = excluded.process_id,
                        first_seen_at = excluded.first_seen_at,
                        last_seen_at = excluded.last_seen_at,
                        observation_count = excluded.observation_count,
                        active = excluded.active,
                        status = excluded.status,
                        startup_type = excluded.startup_type",
                    params![
                        service.key,
                        service.service_name,
                        service.display_name,
                        service.binary_path,
                        service.account,
                        service.pid,
                        service.first_seen,
                        service.last_seen,
                        service.observation_count,
                        service.active as i64,
                        service.status,
                        service.startup_type,
                    ],
                )
                .map_err(|_| "Unable to persist service observations")?;
        }

        for event in &snapshot.events {
            let factual_payload = serde_json::to_string(&event.factual_payload)
                .map_err(|_| "Unable to serialize factual event payload")?;
            transaction
                .execute(
                    "INSERT OR IGNORE INTO telemetry_events(
                        id, event_type, subject_type, subject_key, message, occurred_at,
                        collector, factual_payload_json, schema_version
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![
                        event.event_id,
                        event.event_type,
                        event.entity_type,
                        event.entity_key,
                        event.message,
                        event.timestamp,
                        event.collector,
                        factual_payload,
                        event.schema_version,
                    ],
                )
                .map_err(|_| "Unable to persist telemetry events")?;
        }

        if cleanup_due {
            let observation_cutoff = (Utc::now() - Duration::days(7)).to_rfc3339();
            let event_cutoff = (Utc::now() - Duration::days(30)).to_rfc3339();
            for table in [
                "process_observations",
                "connection_observations",
                "service_observations",
            ] {
                transaction
                    .execute(
                        &format!("DELETE FROM {table} WHERE active = 0 AND last_seen_at < ?1"),
                        [&observation_cutoff],
                    )
                    .map_err(|_| "Unable to enforce observation retention")?;
            }
            transaction
                .execute(
                    "DELETE FROM telemetry_events WHERE occurred_at < ?1",
                    [&event_cutoff],
                )
                .map_err(|_| "Unable to enforce event retention")?;
            transaction
                .execute(
                    "DELETE FROM system_snapshots WHERE collected_at < ?1",
                    [&observation_cutoff],
                )
                .map_err(|_| "Unable to enforce snapshot retention")?;
            transaction
                .execute(
                    "DELETE FROM network_snapshots WHERE collected_at < ?1",
                    [&observation_cutoff],
                )
                .map_err(|_| "Unable to enforce snapshot retention")?;
            enforce_vulnerability_history_retention(&transaction, Utc::now(), false)
                .map_err(|_| "Unable to enforce vulnerability history retention")?;
        }
        transaction
            .commit()
            .map_err(|_| "Unable to commit live telemetry".to_string())?;
        cadence.last_live_write = Some(Instant::now());
        if cleanup_due {
            cadence.last_cleanup = Some(Instant::now());
        }
        cadence.process_counts = engine_process_counts;
        cadence.connection_counts = engine_connection_counts;
        cadence.service_counts = engine_service_counts;
        Ok(())
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

    pub fn get_language(&self) -> Result<Option<String>, String> {
        let connection = self.connection.lock().map_err(|_| "Database unavailable")?;
        connection
            .query_row(
                "SELECT value FROM settings WHERE key = 'language'",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| "Unable to load language preference".into())
    }

    pub fn set_language(&self, language: &str) -> Result<(), String> {
        let connection = self.connection.lock().map_err(|_| "Database unavailable")?;
        connection
            .execute(
                "INSERT INTO settings(key, value, updated_at) VALUES ('language', ?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
                params![language, Utc::now().to_rfc3339()],
            )
            .map(|_| ())
            .map_err(|_| "Unable to persist language preference".into())
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
        let writable = !connection
            .is_readonly("main")
            .map_err(|_| "Unable to read database access mode")?;
        Ok((version, writable))
    }

    pub(crate) fn baseline_read<T>(
        &self,
        operation: impl FnOnce(&Connection) -> Result<T, String>,
    ) -> Result<T, String> {
        let connection = self.connection.lock().map_err(|_| "Database unavailable")?;
        operation(&connection)
    }

    pub(crate) fn baseline_transaction<T>(
        &self,
        operation: impl FnOnce(&Transaction<'_>) -> Result<T, String>,
    ) -> Result<T, String> {
        let mut connection = self.connection.lock().map_err(|_| "Database unavailable")?;
        let transaction = connection
            .transaction()
            .map_err(|_| "Unable to start baseline transaction")?;
        let value = operation(&transaction)?;
        transaction
            .commit()
            .map_err(|_| "Unable to commit baseline transaction")?;
        Ok(value)
    }

    pub(crate) fn analysis_read<T>(
        &self,
        operation: impl FnOnce(&Connection) -> Result<T, String>,
    ) -> Result<T, String> {
        let connection = self.connection.lock().map_err(|_| "Database unavailable")?;
        operation(&connection)
    }

    /// Detection and score writes deliberately use an independent transaction.
    /// An analysis failure can be retried from its durable checkpoint and must
    /// never poison a valid behavioral baseline.
    pub(crate) fn analysis_transaction<T>(
        &self,
        operation: impl FnOnce(&Transaction<'_>) -> Result<T, String>,
    ) -> Result<T, String> {
        let mut connection = self.connection.lock().map_err(|_| "Database unavailable")?;
        let transaction = connection
            .transaction()
            .map_err(|_| "Unable to start security analysis transaction")?;
        let value = operation(&transaction)?;
        transaction
            .commit()
            .map_err(|_| "Unable to commit security analysis transaction")?;
        Ok(value)
    }

    /// Repository reads keep the concrete SQLite error available to callers
    /// that need to compose several fallible statements before translating the
    /// failure at the IPC boundary.
    pub(crate) fn repository_read<T>(
        &self,
        operation: impl FnOnce(&Connection) -> rusqlite::Result<T>,
    ) -> rusqlite::Result<T> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| rusqlite::Error::InvalidQuery)?;
        operation(&connection)
    }

    /// Inventory and vulnerability repository writes are independent from the
    /// baseline lifecycle and retain typed SQLite errors until their boundary.
    pub(crate) fn repository_transaction<T>(
        &self,
        operation: impl FnOnce(&Transaction<'_>) -> rusqlite::Result<T>,
    ) -> rusqlite::Result<T> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| rusqlite::Error::InvalidQuery)?;
        let transaction = connection.transaction()?;
        let value = operation(&transaction)?;
        transaction.commit()?;
        Ok(value)
    }

    pub(crate) fn baseline_engine_transaction<T>(
        &self,
        operation: impl FnOnce(&Transaction<'_>) -> Result<T, String>,
    ) -> Result<T, BaselineTransactionFailure> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| BaselineTransactionFailure {
                message: "Database unavailable".into(),
                persist_active_error: false,
            })?;
        let transaction = connection
            .transaction()
            .map_err(|_| BaselineTransactionFailure {
                message: "Unable to start baseline transaction".into(),
                persist_active_error: false,
            })?;
        let value = operation(&transaction).map_err(|message| BaselineTransactionFailure {
            message,
            persist_active_error: true,
        })?;
        transaction
            .commit()
            .map_err(|_| BaselineTransactionFailure {
                message: "Unable to commit baseline transaction".into(),
                persist_active_error: false,
            })?;
        Ok(value)
    }

    pub(crate) fn mark_active_baseline_error(
        &self,
        error_code: &str,
        public_message: &str,
    ) -> Result<(), String> {
        let connection = self.connection.lock().map_err(|_| "Database unavailable")?;
        connection
            .execute(
                "UPDATE behavioral_baselines
                 SET status = 'error', error_code = ?1, error_message = ?2, updated_at = ?3
                 WHERE active = 1",
                params![error_code, public_message, Utc::now().to_rfc3339()],
            )
            .map(|_| ())
            .map_err(|_| "Unable to persist baseline error state".into())
    }
}

fn enforce_vulnerability_history_retention(
    transaction: &Transaction<'_>,
    now: chrono::DateTime<Utc>,
    force: bool,
) -> Result<usize, rusqlite::Error> {
    let (stored_policy_version, last_run_at): (i64, Option<String>) = transaction.query_row(
        "SELECT policy_version, last_run_at
         FROM vulnerability_history_retention_state
         WHERE policy_key = ?1",
        [VULNERABILITY_RETENTION_POLICY_KEY],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let due = force
        || stored_policy_version != VULNERABILITY_RETENTION_POLICY_VERSION
        || last_run_at
            .as_deref()
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .map_or(true, |last| {
                last.with_timezone(&Utc)
                    <= now - Duration::hours(VULNERABILITY_RETENTION_INTERVAL_HOURS)
            });
    if !due {
        return Ok(0);
    }

    transaction.execute_batch(
        "CREATE TEMP TABLE IF NOT EXISTS vulnerability_retention_candidates(
             evaluation_id TEXT PRIMARY KEY
         ) WITHOUT ROWID;
         DELETE FROM vulnerability_retention_candidates;",
    )?;
    let now_text = now.to_rfc3339();
    transaction.execute(
        "WITH ranked AS (
             SELECT evaluation_id, software_id, completed_at,
                    ROW_NUMBER() OVER (
                        PARTITION BY software_id
                        ORDER BY completed_at DESC, evaluation_id DESC
                    ) AS recent_rank,
                    ROW_NUMBER() OVER (
                        PARTITION BY software_id, date(completed_at)
                        ORDER BY completed_at DESC, evaluation_id DESC
                    ) AS daily_rank,
                    ROW_NUMBER() OVER (
                        PARTITION BY software_id, strftime('%Y-%m', completed_at)
                        ORDER BY completed_at DESC, evaluation_id DESC
                    ) AS monthly_rank,
                    CAST(julianday(?1) - julianday(completed_at) AS INTEGER) AS age_days
             FROM software_vulnerability_evaluations
         )
         INSERT OR IGNORE INTO vulnerability_retention_candidates(evaluation_id)
         SELECT evaluation_id
         FROM ranked
         WHERE age_days >= 0
           AND recent_rank > ?2
           AND NOT (age_days <= ?3 AND daily_rank = 1)
           AND NOT (age_days <= ?4 AND monthly_rank = 1)
         ORDER BY completed_at, evaluation_id
         LIMIT ?5",
        params![
            now_text,
            VULNERABILITY_RETENTION_RECENT_PER_SOFTWARE,
            VULNERABILITY_RETENTION_DAILY_DAYS,
            VULNERABILITY_RETENTION_MONTHLY_DAYS,
            VULNERABILITY_RETENTION_BATCH_SIZE,
        ],
    )?;

    transaction.execute(
        "DELETE FROM vulnerability_match_evidence
         WHERE match_id IN (
             SELECT match_id FROM vulnerability_matches
             WHERE evaluation_id IN (
                 SELECT evaluation_id FROM vulnerability_retention_candidates
             )
         )",
        [],
    )?;
    transaction.execute(
        "DELETE FROM vulnerability_matches
         WHERE evaluation_id IN (
             SELECT evaluation_id FROM vulnerability_retention_candidates
         )",
        [],
    )?;
    transaction.execute(
        "DELETE FROM software_cpe_candidates
         WHERE evaluation_id IN (
             SELECT evaluation_id FROM vulnerability_retention_candidates
         )",
        [],
    )?;
    transaction.execute(
        "DELETE FROM software_identity_evaluations
         WHERE evaluation_id IN (
             SELECT evaluation_id FROM vulnerability_retention_candidates
         )",
        [],
    )?;
    let deleted = transaction.execute(
        "DELETE FROM software_vulnerability_evaluations
         WHERE evaluation_id IN (
             SELECT evaluation_id FROM vulnerability_retention_candidates
         )",
        [],
    )?;
    transaction.execute(
        "UPDATE vulnerability_history_retention_state
         SET policy_version = ?2,
             last_run_at = ?3,
             last_deleted_evaluations = ?4,
             total_deleted_evaluations = total_deleted_evaluations + ?4
         WHERE policy_key = ?1",
        params![
            VULNERABILITY_RETENTION_POLICY_KEY,
            VULNERABILITY_RETENTION_POLICY_VERSION,
            now_text,
            deleted as i64,
        ],
    )?;
    Ok(deleted)
}

#[derive(Debug)]
struct StoredTracking {
    first_seen: String,
    observation_count: u64,
    active: bool,
}

type Tracking = HashMap<String, StoredTracking>;

fn load_tracking(
    connection: &Connection,
    table: &str,
    key_column: &str,
) -> Result<Tracking, String> {
    let sql = format!("SELECT {key_column}, first_seen_at, observation_count, active FROM {table}");
    let mut statement = connection
        .prepare(&sql)
        .map_err(|_| "Unable to prepare observation tracking")?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                StoredTracking {
                    first_seen: row.get(1)?,
                    observation_count: row.get(2)?,
                    active: row.get::<_, i64>(3)? != 0,
                },
            ))
        })
        .map_err(|_| "Unable to read observation tracking")?;
    rows.collect::<Result<Tracking, _>>()
        .map_err(|_| "Unable to read observation tracking".into())
}

fn apply_tracking(
    first_seen: &mut String,
    count: &mut u64,
    active: bool,
    stored: Option<&StoredTracking>,
    previous_engine_count: Option<u64>,
) {
    if let Some(stored) = stored {
        if active && stored.active {
            let delta = count.saturating_sub(previous_engine_count.unwrap_or(0));
            *first_seen = stored.first_seen.clone();
            *count = stored.observation_count.saturating_add(delta);
        } else if !active {
            *first_seen = stored.first_seen.clone();
            *count = stored.observation_count;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        enforce_vulnerability_history_retention, Database, VULNERABILITY_RETENTION_BATCH_SIZE,
    };
    use crate::models::{
        CollectorHealth, CollectorStatus, ConnectionRecord, LiveTelemetrySnapshot, ProcessRecord,
        ServiceRecord, TelemetryEvent,
    };
    use chrono::{Duration, Utc};
    use rusqlite::params;

    fn snapshot(active: bool, with_event: bool) -> LiveTelemetrySnapshot {
        LiveTelemetrySnapshot {
            collected_at: "2026-08-16T12:00:00Z".into(),
            processes: vec![ProcessRecord {
                key: "7:100".into(),
                name: "sample.exe".into(),
                pid: 7,
                parent_pid: Some(1),
                user: Some("local-user".into()),
                executable_path: Some("C:\\sample.exe".into()),
                command_line: Some("sample.exe --sensitive-argument must-not-be-persisted".into()),
                cpu_percent: Some(1.5),
                core_equivalent_cpu_percent: Some(12.0),
                memory_bytes: 42,
                start_time: Some("2026-08-16T11:00:00Z".into()),
                thread_count: Some(2),
                architecture: Some("x64".into()),
                description: Some("Sample".into()),
                company: Some("Sample Company".into()),
                signature_status: "unsigned".into(),
                signer: None,
                executable_file_size: Some(1024),
                executable_modified_at: Some("2026-08-16T00:00:00Z".into()),
                access_status: "available".into(),
                first_seen: "2026-08-16T12:00:00Z".into(),
                last_seen: "2026-08-16T12:00:00Z".into(),
                observation_count: 1,
                active,
            }],
            connections: vec![ConnectionRecord {
                key: "tcp|ipv4|127.0.0.1|80|-|-|7".into(),
                protocol: "tcp".into(),
                ip_version: "ipv4".into(),
                local_address: "127.0.0.1".into(),
                local_port: 80,
                remote_address: None,
                remote_port: None,
                state: Some("listening".into()),
                pid: Some(7),
                process_name: Some("sample.exe".into()),
                executable_path: Some("C:\\sample.exe".into()),
                association_status: "associated".into(),
                process_last_seen: None,
                first_seen: "2026-08-16T12:00:00Z".into(),
                last_seen: "2026-08-16T12:00:00Z".into(),
                observation_count: 1,
                active,
            }],
            services: vec![ServiceRecord {
                key: "sample".into(),
                service_name: "Sample".into(),
                display_name: "Sample Service".into(),
                status: "Running".into(),
                startup_type: "Manual".into(),
                binary_path: Some("C:\\sample.exe".into()),
                account: Some("LocalSystem".into()),
                pid: Some(7),
                first_seen: "2026-08-16T12:00:00Z".into(),
                last_seen: "2026-08-16T12:00:00Z".into(),
                observation_count: 1,
                active,
            }],
            events: with_event
                .then(|| TelemetryEvent {
                    event_id: "event-1".into(),
                    event_type: "connection_closed".into(),
                    entity_type: "connection".into(),
                    entity_key: "sample".into(),
                    timestamp: "2026-08-16T12:00:01Z".into(),
                    collector: "connections".into(),
                    factual_payload: serde_json::json!({"message": "Connection is no longer observed"}),
                    schema_version: 1,
                    message: "Connection is no longer observed".into(),
                })
                .into_iter()
                .collect(),
            collectors: vec![CollectorHealth {
                id: "processes".into(),
                status: CollectorStatus::Healthy,
                detail: "test".into(),
                last_success: Some("2026-08-16T12:00:00Z".into()),
                last_attempt: "2026-08-16T12:00:00Z".into(),
                duration_ms: 1,
                observation_count: 1,
                restricted_count: 0,
                error_code: None,
                error_message: None,
            }],
            issues: Vec::new(),
        }
    }

    #[test]
    fn migrations_are_versioned_and_preferences_round_trip() {
        let database = Database::in_memory().expect("database should initialize");
        assert_eq!(database.status().expect("status").0, 11);
        database.set_theme("terminal").expect("theme should save");
        assert_eq!(database.get_theme().expect("theme should load"), "terminal");
        assert_eq!(database.get_language().expect("language query"), None);
        database
            .set_language("pt-BR")
            .expect("language should save");
        assert_eq!(
            database.get_language().expect("language should load"),
            Some("pt-BR".into())
        );
    }

    #[test]
    fn sprint_zero_schema_upgrades_without_recreating_existing_tables() {
        let mut connection = rusqlite::Connection::open_in_memory().expect("in-memory database");
        Database::configure(&connection).expect("database configuration");
        connection
            .execute_batch(
                "CREATE TABLE schema_migrations(version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);",
            )
            .expect("migration registry");
        connection
            .execute_batch(include_str!("../../migrations/0001_foundation.sql"))
            .expect("Sprint 0 schema");
        connection
            .execute(
                "INSERT INTO schema_migrations(version, applied_at) VALUES (1, '2026-08-16T00:00:00Z')",
                [],
            )
            .expect("Sprint 0 migration marker");
        Database::migrate(&mut connection).expect("current migrations");
        let version: i64 = connection
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .expect("schema version");
        let live_tables: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN (
                    'process_observations', 'connection_observations',
                    'service_observations', 'telemetry_events'
                )",
                [],
                |row| row.get(0),
            )
            .expect("live telemetry tables");
        let baseline_tables: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN (
                    'behavioral_baselines', 'baseline_executables',
                    'baseline_process_patterns', 'baseline_parent_child_relationships',
                    'baseline_network_destinations', 'baseline_services',
                    'baseline_network_configurations'
                )",
                [],
                |row| row.get(0),
            )
            .expect("behavioral baseline tables");
        let event_columns: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('security_events') WHERE name IN (
                    'entity_key', 'title', 'first_seen_at', 'last_seen_at',
                    'baseline_context_json', 'baseline_id', 'status',
                    'observation_count', 'condition_active', 'schema_version', 'rule_version'
                )",
                [],
                |row| row.get(0),
            )
            .expect("security event foundation columns");
        let hardening_columns: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('behavioral_baselines')
                 WHERE name IN ('error_code', 'updated_at')",
                [],
                |row| row.get(0),
            )
            .expect("hardening columns");
        let history_table: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'table' AND name = 'security_event_history'",
                [],
                |row| row.get(0),
            )
            .expect("history table");
        let detection_tables: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN (
                    'detection_rule_versions', 'detection_rule_state', 'detections',
                    'detection_evidence', 'detection_history', 'security_score_snapshots',
                    'analysis_checkpoints'
                )",
                [],
                |row| row.get(0),
            )
            .expect("detection tables");
        let sprint_three_tables: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN (
                    'software_inventory_snapshots', 'installed_software',
                    'software_inventory_observations', 'software_inventory_events',
                    'nvd_vulnerabilities', 'cisa_kev_vulnerabilities',
                    'vulnerability_provider_state'
                )",
                [],
                |row| row.get(0),
            )
            .expect("Sprint 3 foundation tables");
        let vulnerability_score_columns: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('security_score_snapshots')
                 WHERE name IN ('detection_penalty', 'vulnerability_penalty',
                                'vulnerability_coverage_json',
                                'product_vulnerability_risks_json')",
                [],
                |row| row.get(0),
            )
            .expect("vulnerability score v2 columns");
        let retention_tables: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'table'
                   AND name = 'vulnerability_history_retention_state'",
                [],
                |row| row.get(0),
            )
            .expect("vulnerability retention state table");
        assert_eq!(version, 11);
        assert_eq!(live_tables, 4);
        assert_eq!(baseline_tables, 7);
        assert_eq!(event_columns, 11);
        assert_eq!(hardening_columns, 2);
        assert_eq!(history_table, 1);
        assert_eq!(detection_tables, 7);
        assert_eq!(sprint_three_tables, 7);
        assert_eq!(vulnerability_score_columns, 4);
        assert_eq!(retention_tables, 1);
    }

    #[test]
    fn vulnerability_history_retention_is_bounded_and_preserves_current_evidence() {
        let database = Database::in_memory().expect("database should initialize");
        let now = chrono::DateTime::parse_from_rfc3339("2026-08-17T12:00:00Z")
            .expect("fixed time")
            .with_timezone(&Utc);
        let mut connection = database.connection.lock().expect("connection lock");
        connection
            .execute(
                "INSERT INTO software_inventory_snapshots(
                    snapshot_id, collected_at, duration_ms, raw_entry_count,
                    software_count, source_count
                 ) VALUES ('snapshot', ?1, 1, 1, 1, 1)",
                [now.to_rfc3339()],
            )
            .expect("inventory snapshot");
        connection
            .execute(
                "INSERT INTO installed_software(
                    software_id, display_name, display_version, architecture,
                    install_scope, sources_json, registry_identities_json,
                    normalized_vendor, normalized_product, normalized_version,
                    identity_status, first_seen_at, last_seen_at, last_snapshot_id
                 ) VALUES (
                    'software', 'Product', '1.0', 'x64', 'machine', '[]', '[]',
                    'vendor', 'product', '1.0', 'resolved', ?1, ?1, 'snapshot'
                 )",
                [now.to_rfc3339()],
            )
            .expect("installed software");
        for index in 0..105 {
            let completed_at = (now - Duration::days(800 + index)).to_rfc3339();
            connection
                .execute(
                    "INSERT INTO software_vulnerability_evaluations(
                        evaluation_id, software_id, software_fingerprint,
                        matching_engine_version, nvd_source_version, kev_source_version,
                        outcome, candidate_count, confirmed_count, possible_count,
                        unresolved_count, not_affected_count, started_at, completed_at,
                        duration_ms
                     ) VALUES (
                        ?1, 'software', ?2, 1, 'nvd-v1', 'kev-v1', ?3, 1, ?4,
                        0, 0, 0, ?5, ?5, 1
                     )",
                    params![
                        format!("evaluation-{index:03}"),
                        format!("fingerprint-{index:03}"),
                        if index == 0 {
                            "confirmed"
                        } else {
                            "no_confirmed"
                        },
                        if index == 0 { 1 } else { 0 },
                        completed_at,
                    ],
                )
                .expect("evaluation history");
        }
        connection
            .execute(
                "INSERT INTO nvd_vulnerabilities(
                    cve_id, published_at, last_modified_at, description,
                    weaknesses_json, references_json, applicability_json,
                    repository_updated_at
                 ) VALUES ('CVE-2026-1', ?1, ?1, 'Fixture', '[]', '[]', '{}', ?1)",
                [now.to_rfc3339()],
            )
            .expect("CVE fixture");
        connection
            .execute(
                "INSERT INTO vulnerability_matches(
                    match_id, evaluation_id, software_id, cve_id, match_state,
                    confidence, installed_version, affected_range, comparison_result,
                    matching_engine_version, nvd_source_version, kev_source_version,
                    last_evaluated_at
                 ) VALUES (
                    'current-match', 'evaluation-000', 'software', 'CVE-2026-1',
                    'confirmed', 'high', '1.0', '<= 2.0', 'within_range', 1,
                    'nvd-v1', 'kev-v1', ?1
                 )",
                [now.to_rfc3339()],
            )
            .expect("current confirmed match");
        connection
            .execute(
                "INSERT INTO vulnerability_match_evidence(
                    match_id, evidence_type, source, observed_at, evidence_json
                 ) VALUES ('current-match', 'version-range', 'nvd', ?1, '{}')",
                [now.to_rfc3339()],
            )
            .expect("current match evidence");

        let transaction = connection.transaction().expect("retention transaction");
        let deleted = enforce_vulnerability_history_retention(&transaction, now, true)
            .expect("retention run");
        transaction.commit().expect("retention commit");

        assert_eq!(deleted, 5);
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM software_vulnerability_evaluations",
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .expect("evaluation count"),
            100
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM vulnerability_match_evidence
                     WHERE match_id = 'current-match'",
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .expect("current evidence count"),
            1
        );
        let transaction = connection.transaction().expect("cadence transaction");
        assert_eq!(
            enforce_vulnerability_history_retention(&transaction, now, false)
                .expect("cadence check"),
            0
        );
        transaction.commit().expect("cadence commit");

        for index in 105..505 {
            let completed_at = (now - Duration::days(1_000 + index)).to_rfc3339();
            connection
                .execute(
                    "INSERT INTO software_vulnerability_evaluations(
                        evaluation_id, software_id, software_fingerprint,
                        matching_engine_version, outcome, candidate_count,
                        confirmed_count, possible_count, unresolved_count,
                        not_affected_count, started_at, completed_at, duration_ms
                     ) VALUES (
                        ?1, 'software', ?2, 1, 'no_confirmed', 0, 0, 0, 0, 0,
                        ?3, ?3, 1
                     )",
                    params![
                        format!("evaluation-{index:03}"),
                        format!("fingerprint-{index:03}"),
                        completed_at,
                    ],
                )
                .expect("retention backlog");
        }
        let transaction = connection.transaction().expect("bounded transaction");
        assert_eq!(
            enforce_vulnerability_history_retention(&transaction, now, true)
                .expect("bounded retention run"),
            VULNERABILITY_RETENTION_BATCH_SIZE as usize
        );
        transaction.commit().expect("bounded retention commit");
        assert_eq!(
            connection
                .query_row(
                    "SELECT last_deleted_evaluations, total_deleted_evaluations
                     FROM vulnerability_history_retention_state
                     WHERE policy_key = 'vulnerability-evaluation-history'",
                    [],
                    |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
                )
                .expect("retention counters"),
            (VULNERABILITY_RETENTION_BATCH_SIZE, 255)
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM vulnerability_match_evidence
                     WHERE match_id = 'current-match'",
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .expect("current evidence after backlog"),
            1
        );
    }

    #[test]
    fn sprint_two_b_schema_preserves_rule_and_detection_provenance() {
        let database = Database::in_memory().expect("database should initialize");
        let connection = database.connection.lock().expect("connection lock");
        let checkpoint: (i64, i64) = connection
            .query_row(
                "SELECT cutover_security_event_history_id,
                        last_security_event_history_id
                 FROM analysis_checkpoints
                 WHERE consumer_id = 'detection-engine-v1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("checkpoint");
        assert_eq!(checkpoint, (0, 0));

        connection
            .execute(
                "INSERT INTO detection_rule_versions(
                    rule_id, rule_version, name, description, category, definition_json,
                    definition_sha256, registered_at
                 ) VALUES ('EDY-PROC-001', 1, 'Rule', 'Description', 'process', '{}',
                           ?1, '2026-08-16T10:00:00Z')",
                ["a".repeat(64)],
            )
            .expect("rule version");
        assert!(connection
            .execute("UPDATE detection_rule_versions SET name = 'Changed'", [],)
            .is_err());
        assert!(connection
            .execute("DELETE FROM detection_rule_versions", [],)
            .is_err());

        let invalid_detection = connection.execute(
            "INSERT INTO detections(
                detection_id, dedup_key, correlation_key, rule_id, rule_version,
                baseline_id, entity_type, entity_key, title, summary, severity,
                confidence, severity_reason, confidence_reason, first_detected_at,
                last_detected_at, explanation_json, created_at, updated_at
             ) VALUES ('detection', 'dedup', 'correlation', 'EDY-PROC-001', 1,
                       'missing-baseline', 'process', 'entity', 'Title', 'Summary',
                       'invented', 'high', 'Reason', 'Reason', '2026-08-16T10:00:00Z',
                       '2026-08-16T10:00:00Z', '{}', '2026-08-16T10:00:00Z',
                       '2026-08-16T10:00:00Z')",
            [],
        );
        assert!(invalid_detection.is_err());
        assert_eq!(
            connection
                .query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
                .expect("integrity"),
            "ok"
        );
    }

    #[test]
    fn sprint_three_foundation_is_integral_and_keeps_facts_without_severity() {
        let database = Database::in_memory().expect("database should initialize");
        let connection = database.connection.lock().expect("connection lock");
        let providers: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM vulnerability_provider_state
                 WHERE provider IN ('nvd', 'cisa_kev') AND status = 'idle'",
                [],
                |row| row.get(0),
            )
            .expect("provider state seeds");
        let severity_columns: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('software_inventory_events')
                 WHERE name = 'severity'",
                [],
                |row| row.get(0),
            )
            .expect("factual event columns");
        assert_eq!(providers, 2);
        assert_eq!(severity_columns, 0);
        assert!(connection
            .execute(
                "UPDATE vulnerability_provider_state SET status='invented' WHERE provider='nvd'",
                [],
            )
            .is_err());
        let foreign_key_issues: i64 = connection
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .expect("foreign key check");
        let integrity: String = connection
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .expect("integrity check");
        assert_eq!(foreign_key_issues, 0);
        assert_eq!(integrity, "ok");
    }

    #[test]
    fn sprint_three_matching_schema_preserves_provenance_and_incremental_queue() {
        let database = Database::in_memory().expect("database should initialize");
        let connection = database.connection.lock().expect("connection lock");
        let tables: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN (
                    'nvd_cpe_matches','software_vulnerability_evaluations',
                    'software_identity_evaluations','software_cpe_candidates',
                    'vulnerability_matches','vulnerability_match_evidence',
                    'vulnerability_evaluation_queue')",
                [],
                |row| row.get(0),
            )
            .expect("matching tables");
        let configurations: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('nvd_vulnerabilities') WHERE name='configurations_json'",
                [],
                |row| row.get(0),
            )
            .expect("NVD configurations column");
        assert_eq!(tables, 7);
        assert_eq!(configurations, 1);
        let foreign_key_issues: i64 = connection
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .expect("foreign key check");
        assert_eq!(foreign_key_issues, 0);
    }

    #[test]
    fn sprint_two_hardening_enforces_event_integrity_and_append_only_history() {
        let database = Database::in_memory().expect("database should initialize");
        let connection = database.connection.lock().expect("connection lock");
        connection
            .execute(
                "INSERT INTO behavioral_baselines(
                    baseline_id, created_at, learning_started_at, version, host_id, status,
                    learning_period_seconds, updated_at
                 ) VALUES ('baseline-v5', '2026-08-16T10:00:00Z', '2026-08-16T10:00:00Z',
                           1, 'host-v5', 'ready', 86400, '2026-08-16T10:00:00Z')",
                [],
            )
            .expect("baseline fixture");
        connection
            .execute(
                "INSERT INTO security_events(
                    id, event_type, occurred_at, source, payload_json, entity_type, entity_key,
                    title, first_seen_at, last_seen_at, baseline_context_json, baseline_id,
                    status, observation_count, condition_active, schema_version
                 ) VALUES ('event-v5', 'process_first_seen', '2026-08-16T10:01:00Z',
                           'processes', '{}', 'process', 'entity-v5', 'Observed process',
                           '2026-08-16T10:01:00Z', '2026-08-16T10:01:00Z', '{}',
                           'baseline-v5', 'new', 1, 1, 1)",
                [],
            )
            .expect("event fixture");
        assert!(connection
            .execute(
                "UPDATE security_events SET status = 'invented' WHERE id = 'event-v5'",
                [],
            )
            .is_err());
        assert!(connection
            .execute(
                "UPDATE security_events SET condition_active = 7 WHERE id = 'event-v5'",
                [],
            )
            .is_err());
        assert!(connection
            .execute(
                "UPDATE security_events SET first_seen_at = NULL WHERE id = 'event-v5'",
                [],
            )
            .is_err());
        assert!(connection
            .execute(
                "DELETE FROM behavioral_baselines WHERE baseline_id = 'baseline-v5'",
                [],
            )
            .is_err());
        connection
            .execute(
                "INSERT INTO security_event_history(
                    event_id, transition, observed_at, recorded_at, source, event_type,
                    entity_type, entity_key, baseline_id, evidence_json,
                    baseline_context_json, event_schema_version, observation_count
                 ) VALUES ('event-v5', 'first_observed', '2026-08-16T10:01:00Z',
                           '2026-08-16T10:01:00Z', 'processes', 'process_first_seen',
                           'process', 'entity-v5', 'baseline-v5', '{}', '{}', 1, 1)",
                [],
            )
            .expect("history fixture");
        assert!(connection
            .execute(
                "UPDATE security_event_history SET evidence_json = '{\"changed\":true}'",
                [],
            )
            .is_err());
        let foreign_key_issues: i64 = connection
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .expect("foreign key check");
        let integrity: String = connection
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .expect("integrity check");
        assert_eq!(foreign_key_issues, 0);
        assert_eq!(integrity, "ok");
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

    #[test]
    fn live_observations_persist_factual_fields_without_command_line() {
        let database = Database::in_memory().expect("database should initialize");
        let mut value = snapshot(true, false);
        database
            .persist_live_telemetry(&mut value)
            .expect("live telemetry should persist");
        let connection = database.connection.lock().expect("connection lock");
        let process: (u32, String, Option<u32>, i64) = connection
            .query_row(
                "SELECT pid, process_name, parent_pid, active FROM process_observations",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .expect("process observation");
        assert_eq!(process, (7, "sample.exe".into(), Some(1), 1));
        let command_column: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('process_observations') WHERE name = 'command_line'",
                [],
                |row| row.get(0),
            )
            .expect("schema query");
        assert_eq!(command_column, 0);
    }

    #[test]
    fn closed_observations_are_retained_as_inactive() {
        let database = Database::in_memory().expect("database should initialize");
        let mut active = snapshot(true, false);
        database
            .persist_live_telemetry(&mut active)
            .expect("active observation should persist");
        let mut closed = snapshot(false, true);
        closed.collected_at = "2026-08-16T12:00:01Z".into();
        for process in &mut closed.processes {
            process.last_seen = closed.collected_at.clone();
        }
        database
            .persist_live_telemetry(&mut closed)
            .expect("closed observation should persist with event");
        let connection = database.connection.lock().expect("connection lock");
        let active: i64 = connection
            .query_row(
                "SELECT active FROM process_observations WHERE process_key = '7:100'",
                [],
                |row| row.get(0),
            )
            .expect("process active state");
        assert_eq!(active, 0);
        let event_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM telemetry_events", [], |row| {
                row.get(0)
            })
            .expect("event count");
        assert_eq!(event_count, 1);
    }
}
