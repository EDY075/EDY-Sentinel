use crate::models::{LiveTelemetrySnapshot, SystemOverview};
use chrono::{Duration, Utc};
use rusqlite::{params, Connection};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration as StdDuration, Instant},
};

const MIGRATIONS: &[(i64, &str)] = &[
    (1, include_str!("../../migrations/0001_foundation.sql")),
    (2, include_str!("../../migrations/0002_live_telemetry.sql")),
];

#[derive(Clone)]
pub struct Database {
    connection: Arc<Mutex<Connection>>,
    cadence: Arc<Mutex<PersistenceCadence>>,
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
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
            cadence: Arc::new(Mutex::new(PersistenceCadence::default())),
        })
    }

    #[cfg(test)]
    fn in_memory() -> Result<Self, rusqlite::Error> {
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
                        access_status
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
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
                        access_status = excluded.access_status",
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
                        observation_count, active
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
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
                        active = excluded.active",
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
            transaction
                .execute(
                    "INSERT OR IGNORE INTO telemetry_events(
                        id, event_type, subject_type, subject_key, message, occurred_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        event.id,
                        event.event_type,
                        event.subject_type,
                        event.subject_key,
                        event.message,
                        event.occurred_at,
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
    use super::Database;
    use crate::models::{
        CollectorHealth, ConnectionRecord, LiveTelemetrySnapshot, ProcessRecord, ServiceRecord,
        TelemetryEvent,
    };

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
                memory_bytes: 42,
                start_time: Some("2026-08-16T11:00:00Z".into()),
                thread_count: Some(2),
                architecture: Some("x64".into()),
                description: Some("Sample".into()),
                publisher: None,
                signature_status: "unsigned".into(),
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
                    id: "event-1".into(),
                    event_type: "connection_closed".into(),
                    subject_type: "connection".into(),
                    subject_key: "sample".into(),
                    message: "Connection is no longer observed".into(),
                    occurred_at: "2026-08-16T12:00:01Z".into(),
                })
                .into_iter()
                .collect(),
            collectors: vec![CollectorHealth {
                id: "processes".into(),
                status: "active".into(),
                detail: "test".into(),
                collected_at: "2026-08-16T12:00:00Z".into(),
            }],
            issues: Vec::new(),
        }
    }

    #[test]
    fn migrations_are_versioned_and_theme_round_trips() {
        let database = Database::in_memory().expect("database should initialize");
        assert_eq!(database.status().expect("status").0, 2);
        database.set_theme("terminal").expect("theme should save");
        assert_eq!(database.get_theme().expect("theme should load"), "terminal");
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
        Database::migrate(&mut connection).expect("Sprint 1 migration");
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
        assert_eq!(version, 2);
        assert_eq!(live_tables, 4);
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
