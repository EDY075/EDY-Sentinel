use crate::{
    models::{
        BaselineActionInput, BaselineEntityCounts, BaselineStatus, BaselineSummary,
        ConnectionRecord, LiveTelemetrySnapshot, NetworkInfo, ProcessRecord, SecurityEventRecord,
        SecurityEventStatusInput, ServiceRecord, SystemOverview,
    },
    persistence::Database,
};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use rusqlite::{params, OptionalExtension, Transaction};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    os::windows::ffi::OsStrExt,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use windows_sys::Win32::Storage::FileSystem::GetVolumeInformationW;
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

const BASELINE_SCHEMA_VERSION: u32 = 1;
const EVENT_SCHEMA_VERSION: u32 = 1;
const DEFAULT_LEARNING_PERIOD_SECONDS: u64 = 24 * 60 * 60;
const LIVE_OBSERVATION_INTERVAL: Duration = Duration::from_secs(10);
const NETWORK_OBSERVATION_INTERVAL: Duration = Duration::from_secs(15);
const STALE_AFTER: ChronoDuration = ChronoDuration::hours(24);
const SECURITY_EVENT_RETENTION_DAYS: i64 = 90;
const SECURITY_EVENT_MAX_RETENTION_DAYS: i64 = 180;
const ALLOWED_LEARNING_PERIODS: [u64; 6] = [60, 3_600, 21_600, 86_400, 259_200, 604_800];

#[derive(Clone, Default)]
pub struct BaselineEngine {
    cadence: Arc<Mutex<BaselineCadence>>,
}

#[derive(Default)]
struct BaselineCadence {
    last_live_observation: Option<Instant>,
    last_network_observation: Option<Instant>,
    last_processing_duration_ms: u64,
}

#[derive(Clone)]
struct BaselineRow {
    baseline_id: String,
    created_at: String,
    learning_started_at: String,
    learning_completed_at: Option<String>,
    version: u32,
    host_id: String,
    status: BaselineStatus,
    observation_count: u64,
    schema_version: u32,
    learning_period_seconds: u64,
    last_observed_at: Option<String>,
    error_message: Option<String>,
}

#[derive(Clone)]
struct ExecutableIdentity {
    key: String,
    normalized_path: Option<String>,
}

#[derive(Clone)]
struct StoredService {
    service_name: String,
    display_name: String,
    startup_type: String,
    binary_path: Option<String>,
    account: Option<String>,
}

#[derive(Clone, PartialEq, Eq)]
struct StoredNetwork {
    primary_interface_key: Option<String>,
    primary_interface_type: Option<String>,
    primary_ipv4: Option<String>,
    gateway: Option<String>,
    dns: Vec<String>,
    route_metric: Option<u32>,
}

impl BaselineEngine {
    pub fn summary(&self, database: &Database) -> Result<BaselineSummary, String> {
        let mut summary = database.baseline_read(|connection| {
            let row = load_active_baseline(connection)?;
            row.map(|value| summary_from_row(connection, value))
                .transpose()
                .map(|value| value.unwrap_or_default())
        })?;
        summary.last_processing_duration_ms = self
            .cadence
            .lock()
            .map_err(|_| "Baseline engine unavailable")?
            .last_processing_duration_ms;
        Ok(summary)
    }

    pub fn security_events(&self, database: &Database) -> Result<Vec<SecurityEventRecord>, String> {
        database.baseline_read(|connection| {
            let mut statement = connection
                .prepare(
                    "SELECT id, event_type, entity_type, entity_key, title, occurred_at,
                            COALESCE(first_seen_at, occurred_at), COALESCE(last_seen_at, occurred_at),
                            payload_json, baseline_context_json, source, baseline_id, rule_id,
                            confidence, status, observation_count, condition_active, schema_version
                     FROM security_events
                     ORDER BY COALESCE(last_seen_at, occurred_at) DESC
                     LIMIT 250",
                )
                .map_err(|_| "Unable to prepare security event query")?;
            let rows = statement
                .query_map([], |row| {
                    let evidence: String = row.get(8)?;
                    let baseline_context: String = row.get(9)?;
                    Ok(SecurityEventRecord {
                        event_id: row.get(0)?,
                        event_type: row.get(1)?,
                        entity_type: row.get(2)?,
                        entity_key: row.get(3)?,
                        title: row.get(4)?,
                        timestamp: row.get(5)?,
                        first_seen: row.get(6)?,
                        last_seen: row.get(7)?,
                        evidence: serde_json::from_str(&evidence).unwrap_or(Value::Null),
                        baseline_context: serde_json::from_str(&baseline_context)
                            .unwrap_or(Value::Null),
                        source: row.get(10)?,
                        baseline_id: row.get(11)?,
                        rule_id: row.get(12)?,
                        confidence: row.get(13)?,
                        status: row.get(14)?,
                        observation_count: row.get(15)?,
                        condition_active: row.get::<_, i64>(16)? != 0,
                        schema_version: row.get(17)?,
                    })
                })
                .map_err(|_| "Unable to query security events")?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|_| "Unable to read security events".into())
        })
    }

    pub fn start_new_baseline(
        &self,
        database: &Database,
        input: BaselineActionInput,
    ) -> Result<BaselineSummary, String> {
        if input.confirmation != "START NEW BASELINE" {
            return Err("Type START NEW BASELINE to confirm".into());
        }
        self.begin_learning(database, input.learning_period_seconds)
    }

    pub fn reset_baseline(
        &self,
        database: &Database,
        input: BaselineActionInput,
    ) -> Result<BaselineSummary, String> {
        if input.confirmation != "RESET BASELINE" {
            return Err("Type RESET BASELINE to confirm".into());
        }
        self.begin_learning(database, input.learning_period_seconds)
    }

    pub fn complete_learning(
        &self,
        database: &Database,
        input: BaselineActionInput,
    ) -> Result<BaselineSummary, String> {
        if input.confirmation != "COMPLETE BASELINE" {
            return Err("Type COMPLETE BASELINE to confirm".into());
        }
        database.baseline_transaction(|transaction| {
            let row = load_active_baseline(transaction)?
                .ok_or_else(|| "No baseline is currently learning".to_string())?;
            if row.status != BaselineStatus::Learning {
                return Err("Only a learning baseline can be completed".into());
            }
            if row.observation_count == 0 {
                return Err("Collect at least one real observation before completing".into());
            }
            let completed_at = Utc::now().to_rfc3339();
            transaction
                .execute(
                    "UPDATE behavioral_baselines
                     SET status = 'ready', learning_completed_at = ?1, last_observed_at = ?1
                     WHERE baseline_id = ?2 AND active = 1",
                    params![completed_at, row.baseline_id],
                )
                .map_err(|_| "Unable to complete baseline learning")?;
            let updated = load_active_baseline(transaction)?
                .ok_or_else(|| "Completed baseline is unavailable".to_string())?;
            summary_from_row(transaction, updated)
        })
    }

    pub fn set_event_status(
        &self,
        database: &Database,
        input: SecurityEventStatusInput,
    ) -> Result<(), String> {
        if !matches!(
            input.status.as_str(),
            "new" | "seen" | "acknowledged" | "resolved" | "ignored"
        ) {
            return Err("Unsupported security event status".into());
        }
        database.baseline_read(|connection| {
            let changed = connection
                .execute(
                    "UPDATE security_events SET status = ?1 WHERE id = ?2",
                    params![input.status, input.event_id],
                )
                .map_err(|_| "Unable to update security event status")?;
            if changed == 0 {
                return Err("Security event was not found".into());
            }
            Ok(())
        })
    }

    pub fn observe_live(
        &self,
        database: &Database,
        snapshot: &LiveTelemetrySnapshot,
    ) -> Result<(), String> {
        {
            let mut cadence = self
                .cadence
                .lock()
                .map_err(|_| "Baseline engine unavailable")?;
            if cadence
                .last_live_observation
                .is_some_and(|last| last.elapsed() < LIVE_OBSERVATION_INTERVAL)
            {
                return Ok(());
            }
            cadence.last_live_observation = Some(Instant::now());
        }
        let processing_started = Instant::now();
        let now = Utc::now().to_rfc3339();
        let result = database.baseline_transaction(|transaction| {
            let Some(row) = load_active_baseline(transaction)? else {
                return Ok(());
            };
            match row.status {
                BaselineStatus::Learning => learn_live(transaction, &row, snapshot, &now)?,
                BaselineStatus::Ready | BaselineStatus::Stale => {
                    detect_live(transaction, &row, snapshot, &now)?
                }
                BaselineStatus::NotInitialized | BaselineStatus::Error => return Ok(()),
            }
            update_observation(transaction, &row, &now)?;
            complete_if_due(transaction, &row, &now)?;
            enforce_event_retention(transaction)?;
            Ok(())
        });
        self.cadence
            .lock()
            .map_err(|_| "Baseline engine unavailable")?
            .last_processing_duration_ms = processing_started.elapsed().as_millis() as u64;
        result
    }

    pub fn observe_network(
        &self,
        database: &Database,
        overview: &SystemOverview,
    ) -> Result<(), String> {
        {
            let mut cadence = self
                .cadence
                .lock()
                .map_err(|_| "Baseline engine unavailable")?;
            if cadence
                .last_network_observation
                .is_some_and(|last| last.elapsed() < NETWORK_OBSERVATION_INTERVAL)
            {
                return Ok(());
            }
            cadence.last_network_observation = Some(Instant::now());
        }
        let processing_started = Instant::now();
        let now = Utc::now().to_rfc3339();
        let result = database.baseline_transaction(|transaction| {
            let Some(row) = load_active_baseline(transaction)? else {
                return Ok(());
            };
            match row.status {
                BaselineStatus::Learning => {
                    learn_network(transaction, &row, &overview.network, &now)?
                }
                BaselineStatus::Ready | BaselineStatus::Stale => {
                    detect_network(transaction, &row, &overview.network, &now)?
                }
                BaselineStatus::NotInitialized | BaselineStatus::Error => return Ok(()),
            }
            update_observation(transaction, &row, &now)?;
            complete_if_due(transaction, &row, &now)?;
            Ok(())
        });
        self.cadence
            .lock()
            .map_err(|_| "Baseline engine unavailable")?
            .last_processing_duration_ms = processing_started.elapsed().as_millis() as u64;
        result
    }

    fn begin_learning(
        &self,
        database: &Database,
        requested_period: Option<u64>,
    ) -> Result<BaselineSummary, String> {
        let learning_period = requested_period.unwrap_or(DEFAULT_LEARNING_PERIOD_SECONDS);
        if !ALLOWED_LEARNING_PERIODS.contains(&learning_period) {
            return Err("Unsupported learning period".into());
        }
        let host_id = current_host_id()?;
        let now = Utc::now().to_rfc3339();
        let summary = database.baseline_transaction(|transaction| {
            let next_version: u32 = transaction
                .query_row(
                    "SELECT COALESCE(MAX(version), 0) + 1 FROM behavioral_baselines WHERE host_id = ?1",
                    [&host_id],
                    |row| row.get(0),
                )
                .map_err(|_| "Unable to select baseline version")?;
            let baseline_id = format!(
                "baseline-{}",
                &stable_hash(&format!("{host_id}|{next_version}|{now}"))[..24]
            );
            transaction
                .execute(
                    "UPDATE behavioral_baselines SET active = 0 WHERE active = 1",
                    [],
                )
                .map_err(|_| "Unable to preserve previous baseline")?;
            transaction
                .execute(
                    "INSERT INTO behavioral_baselines(
                        baseline_id, created_at, learning_started_at, version, host_id, status,
                        observation_count, schema_version, learning_period_seconds, active
                     ) VALUES (?1, ?2, ?2, ?3, ?4, 'learning', 0, ?5, ?6, 1)",
                    params![
                        baseline_id,
                        now,
                        next_version,
                        host_id,
                        BASELINE_SCHEMA_VERSION,
                        learning_period
                    ],
                )
                .map_err(|_| "Unable to create baseline")?;
            let row = load_active_baseline(transaction)?
                .ok_or_else(|| "Created baseline is unavailable".to_string())?;
            summary_from_row(transaction, row)
        })?;
        let mut cadence = self
            .cadence
            .lock()
            .map_err(|_| "Baseline engine unavailable")?;
        cadence.last_live_observation = None;
        cadence.last_network_observation = None;
        Ok(summary)
    }
}

fn load_active_baseline(connection: &rusqlite::Connection) -> Result<Option<BaselineRow>, String> {
    connection
        .query_row(
            "SELECT baseline_id, created_at, learning_started_at, learning_completed_at,
                    version, host_id, status, observation_count, schema_version,
                    learning_period_seconds, last_observed_at, error_message
             FROM behavioral_baselines WHERE active = 1 ORDER BY version DESC LIMIT 1",
            [],
            |row| {
                let status: String = row.get(6)?;
                Ok(BaselineRow {
                    baseline_id: row.get(0)?,
                    created_at: row.get(1)?,
                    learning_started_at: row.get(2)?,
                    learning_completed_at: row.get(3)?,
                    version: row.get(4)?,
                    host_id: row.get(5)?,
                    status: status_from_db(&status),
                    observation_count: row.get(7)?,
                    schema_version: row.get(8)?,
                    learning_period_seconds: row.get(9)?,
                    last_observed_at: row.get(10)?,
                    error_message: row.get(11)?,
                })
            },
        )
        .optional()
        .map_err(|_| "Unable to read active baseline".into())
}

fn summary_from_row(
    connection: &rusqlite::Connection,
    mut row: BaselineRow,
) -> Result<BaselineSummary, String> {
    if matches!(row.status, BaselineStatus::Ready | BaselineStatus::Stale)
        && row
            .last_observed_at
            .as_deref()
            .and_then(parse_utc)
            .is_some_and(|last| Utc::now().signed_duration_since(last) > STALE_AFTER)
    {
        row.status = BaselineStatus::Stale;
    }
    let count = |table: &str| -> Result<u64, String> {
        connection
            .query_row(
                &format!("SELECT COUNT(*) FROM {table} WHERE baseline_id = ?1"),
                [&row.baseline_id],
                |value| value.get(0),
            )
            .map_err(|_| "Unable to count baseline entities".into())
    };
    let entities = BaselineEntityCounts {
        executables: count("baseline_executables")?,
        process_patterns: count("baseline_process_patterns")?,
        parent_child_relationships: count("baseline_parent_child_relationships")?,
        network_destinations: count("baseline_network_destinations")?,
        services: count("baseline_services")?,
        network_configurations: count("baseline_network_configurations")?,
    };
    Ok(BaselineSummary {
        baseline_id: Some(row.baseline_id),
        created_at: Some(row.created_at),
        learning_started_at: Some(row.learning_started_at),
        learning_completed_at: row.learning_completed_at,
        version: Some(row.version),
        host_id: Some(row.host_id),
        status: row.status,
        observation_count: row.observation_count,
        schema_version: row.schema_version,
        learning_period_seconds: row.learning_period_seconds,
        last_observed_at: row.last_observed_at,
        last_processing_duration_ms: 0,
        error_message: row.error_message,
        entities,
    })
}

fn learn_live(
    transaction: &Transaction<'_>,
    baseline: &BaselineRow,
    snapshot: &LiveTelemetrySnapshot,
    now: &str,
) -> Result<(), String> {
    let processes: HashMap<u32, &ProcessRecord> = snapshot
        .processes
        .iter()
        .filter(|process| process.active)
        .map(|process| (process.pid, process))
        .collect();
    for process in processes.values() {
        let executable = executable_identity(process);
        upsert_executable(transaction, baseline, process, &executable, now)?;
        let parent = process
            .parent_pid
            .and_then(|pid| processes.get(&pid).copied());
        let parent_identity = parent.map(executable_identity);
        let pattern_key = process_pattern_key(
            &executable.key,
            parent_identity
                .as_ref()
                .map(|identity| identity.key.as_str()),
            process.user.as_deref(),
        );
        transaction
            .execute(
                "INSERT INTO baseline_process_patterns(
                    baseline_id, pattern_key, executable_key, process_name,
                    parent_executable_key, parent_process_name, user_name,
                    first_seen_at, last_seen_at, observation_count
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8, 1)
                 ON CONFLICT(baseline_id, pattern_key) DO UPDATE SET
                    last_seen_at = excluded.last_seen_at,
                    observation_count = baseline_process_patterns.observation_count + 1",
                params![
                    baseline.baseline_id,
                    pattern_key,
                    executable.key,
                    process.name,
                    parent_identity.as_ref().map(|identity| &identity.key),
                    parent.map(|value| &value.name),
                    process.user,
                    now
                ],
            )
            .map_err(|_| "Unable to learn process pattern")?;
        if let (Some(parent), Some(parent_identity)) = (parent, parent_identity) {
            let relationship_key = relationship_key(&parent_identity.key, &executable.key);
            transaction
                .execute(
                    "INSERT INTO baseline_parent_child_relationships(
                        baseline_id, relationship_key, parent_executable_key,
                        parent_process_name, child_executable_key, child_process_name,
                        first_seen_at, last_seen_at, observation_count
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, 1)
                     ON CONFLICT(baseline_id, relationship_key) DO UPDATE SET
                        last_seen_at = excluded.last_seen_at,
                        observation_count = baseline_parent_child_relationships.observation_count + 1",
                    params![
                        baseline.baseline_id,
                        relationship_key,
                        parent_identity.key,
                        parent.name,
                        executable.key,
                        process.name,
                        now
                    ],
                )
                .map_err(|_| "Unable to learn parent-child relationship")?;
        }
    }

    for connection in snapshot.connections.iter().filter(|item| item.active) {
        if let Some(destination) = destination_identity(connection, &processes) {
            transaction
                .execute(
                    "INSERT INTO baseline_network_destinations(
                        baseline_id, destination_key, executable_key, process_name,
                        remote_ip, remote_port, protocol, first_seen_at, last_seen_at,
                        observation_count
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8, 1)
                     ON CONFLICT(baseline_id, destination_key) DO UPDATE SET
                        last_seen_at = excluded.last_seen_at,
                        observation_count = baseline_network_destinations.observation_count + 1",
                    params![
                        baseline.baseline_id,
                        destination.key,
                        destination.executable_key,
                        destination.process_name,
                        destination.remote_ip,
                        destination.remote_port,
                        destination.protocol,
                        now
                    ],
                )
                .map_err(|_| "Unable to learn network destination")?;
        }
    }

    for service in snapshot.services.iter().filter(|service| service.active) {
        transaction
            .execute(
                "INSERT INTO baseline_services(
                    baseline_id, service_key, service_name, display_name, status,
                    startup_type, binary_path, account_name, first_seen_at, last_seen_at,
                    observation_count
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9, 1)
                 ON CONFLICT(baseline_id, service_key) DO UPDATE SET
                    display_name = excluded.display_name,
                    status = excluded.status,
                    startup_type = excluded.startup_type,
                    binary_path = excluded.binary_path,
                    account_name = excluded.account_name,
                    last_seen_at = excluded.last_seen_at,
                    observation_count = baseline_services.observation_count + 1",
                params![
                    baseline.baseline_id,
                    service.key,
                    service.service_name,
                    service.display_name,
                    service.status,
                    service.startup_type,
                    service.binary_path,
                    service.account,
                    now
                ],
            )
            .map_err(|_| "Unable to learn service baseline")?;
    }
    Ok(())
}

fn detect_live(
    transaction: &Transaction<'_>,
    baseline: &BaselineRow,
    snapshot: &LiveTelemetrySnapshot,
    now: &str,
) -> Result<(), String> {
    let known_executables = load_keys(
        transaction,
        "baseline_executables",
        "executable_key",
        baseline,
    )?;
    let known_patterns = load_keys(
        transaction,
        "baseline_process_patterns",
        "pattern_key",
        baseline,
    )?;
    let known_relationships = load_keys(
        transaction,
        "baseline_parent_child_relationships",
        "relationship_key",
        baseline,
    )?;
    let known_destinations = load_keys(
        transaction,
        "baseline_network_destinations",
        "destination_key",
        baseline,
    )?;
    let known_services = load_services(transaction, baseline)?;
    let processes: HashMap<u32, &ProcessRecord> = snapshot
        .processes
        .iter()
        .filter(|process| process.active)
        .map(|process| (process.pid, process))
        .collect();
    let mut seen_events = HashSet::new();

    for process in processes.values() {
        let executable = executable_identity(process);
        let parent = process
            .parent_pid
            .and_then(|pid| processes.get(&pid).copied());
        let parent_identity = parent.map(executable_identity);
        if !known_executables.contains(&executable.key) {
            seen_events.insert(upsert_event(
                transaction,
                baseline,
                "executable_first_seen",
                "executable",
                &executable.key,
                "New executable observed",
                now,
                "processes",
                json!({
                    "process": process.name,
                    "path": executable.normalized_path,
                    "company": process.company,
                    "signer": process.signer,
                    "signatureStatus": process.signature_status,
                    "fileSize": process.executable_file_size,
                    "fileModifiedAt": process.executable_modified_at,
                    "firstObserved": now
                }),
                json!({"comparison": "not previously observed", "baselineVersion": baseline.version}),
                Some("high"),
            )?);
        }
        let pattern_key = process_pattern_key(
            &executable.key,
            parent_identity
                .as_ref()
                .map(|identity| identity.key.as_str()),
            process.user.as_deref(),
        );
        if !known_patterns.contains(&pattern_key) {
            seen_events.insert(upsert_event(
                transaction,
                baseline,
                "process_first_seen",
                "process_pattern",
                &pattern_key,
                "New process pattern observed",
                now,
                "processes",
                json!({
                    "process": process.name,
                    "path": executable.normalized_path,
                    "parentProcess": parent.map(|value| &value.name),
                    "user": process.user,
                    "architecture": process.architecture,
                    "firstObserved": now
                }),
                json!({"comparison": "process pattern not present in baseline", "baselineVersion": baseline.version}),
                Some("high"),
            )?);
        }
        if let (Some(parent), Some(parent_identity)) = (parent, parent_identity) {
            let key = relationship_key(&parent_identity.key, &executable.key);
            if !known_relationships.contains(&key) {
                seen_events.insert(upsert_event(
                    transaction,
                    baseline,
                    "parent_child_first_seen",
                    "process_relationship",
                    &key,
                    "New parent-child relationship observed",
                    now,
                    "processes",
                    json!({
                        "parent": parent.name,
                        "parentPath": parent.executable_path,
                        "child": process.name,
                        "childPath": process.executable_path,
                        "firstObserved": now
                    }),
                    json!({"comparison": "relationship not present in baseline", "baselineVersion": baseline.version}),
                    Some("high"),
                )?);
            }
        }
    }

    for connection in snapshot.connections.iter().filter(|item| item.active) {
        if let Some(destination) = destination_identity(connection, &processes) {
            if !known_destinations.contains(&destination.key) {
                seen_events.insert(upsert_event(
                    transaction,
                    baseline,
                    "destination_first_seen",
                    "network_destination",
                    &destination.key,
                    "New network destination observed",
                    now,
                    "connections",
                    json!({
                        "process": destination.process_name,
                        "remoteIp": destination.remote_ip,
                        "remotePort": destination.remote_port,
                        "protocol": destination.protocol,
                        "association": connection.association_status,
                        "firstObserved": now
                    }),
                    json!({"comparison": "destination not present in baseline", "baselineVersion": baseline.version}),
                    (connection.association_status == "associated").then_some("high"),
                )?);
            }
        }
    }

    let current_services: HashMap<_, _> = snapshot
        .services
        .iter()
        .filter(|service| service.active)
        .map(|service| (service.key.clone(), service))
        .collect();
    for (key, service) in &current_services {
        let Some(known) = known_services.get(key) else {
            seen_events.insert(upsert_event(
                transaction,
                baseline,
                "service_first_seen",
                "service",
                key,
                "New Windows service observed",
                now,
                "services",
                service_evidence(service, now),
                json!({"comparison": "service not present in baseline", "baselineVersion": baseline.version}),
                Some("high"),
            )?);
            continue;
        };
        for (event_type, title, field, before, after) in [
            (
                "service_startup_changed",
                "Service startup configuration changed",
                "startupType",
                Some(known.startup_type.clone()),
                Some(service.startup_type.clone()),
            ),
            (
                "service_binary_changed",
                "Service binary path changed",
                "binaryPath",
                known.binary_path.clone(),
                service.binary_path.clone(),
            ),
            (
                "service_account_changed",
                "Service account changed",
                "account",
                known.account.clone(),
                service.account.clone(),
            ),
        ] {
            if before != after {
                let entity_key = format!("{key}|{field}");
                seen_events.insert(upsert_event(
                    transaction,
                    baseline,
                    event_type,
                    "service",
                    &entity_key,
                    title,
                    now,
                    "services",
                    json!({
                        "serviceName": service.service_name,
                        "displayName": service.display_name,
                        "field": field,
                        "before": before,
                        "after": after,
                        "firstObserved": now
                    }),
                    json!({"comparison": "current value differs from baseline", "baselineVersion": baseline.version}),
                    Some("high"),
                )?);
            }
        }
    }
    for (key, known) in &known_services {
        if !current_services.contains_key(key) {
            seen_events.insert(upsert_event(
                transaction,
                baseline,
                "service_removed",
                "service",
                key,
                "Baseline service is no longer observed",
                now,
                "services",
                json!({
                    "serviceName": known.service_name,
                    "displayName": known.display_name,
                    "lastKnownStartupType": known.startup_type,
                    "firstObservedMissing": now
                }),
                json!({"comparison": "service existed in baseline and is absent from the successful SCM snapshot", "baselineVersion": baseline.version}),
                Some("high"),
            )?);
        }
    }

    finish_event_cycle(
        transaction,
        baseline,
        &[
            "executable_first_seen",
            "process_first_seen",
            "parent_child_first_seen",
            "destination_first_seen",
            "service_first_seen",
            "service_removed",
            "service_startup_changed",
            "service_binary_changed",
            "service_account_changed",
        ],
        &seen_events,
    )
}

fn learn_network(
    transaction: &Transaction<'_>,
    baseline: &BaselineRow,
    network: &NetworkInfo,
    now: &str,
) -> Result<(), String> {
    let value = network_value(network);
    let dns = serde_json::to_string(&value.dns).map_err(|_| "Unable to serialize DNS baseline")?;
    let interfaces = serde_json::to_string(&network.interfaces)
        .map_err(|_| "Unable to serialize network interfaces")?;
    transaction
        .execute(
            "INSERT INTO baseline_network_configurations(
                baseline_id, primary_interface_key, primary_interface_type, primary_ipv4,
                gateway, dns_json, route_metric, interfaces_json, first_seen_at,
                last_seen_at, observation_count
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9, 1)
             ON CONFLICT(baseline_id) DO UPDATE SET
                primary_interface_key = excluded.primary_interface_key,
                primary_interface_type = excluded.primary_interface_type,
                primary_ipv4 = excluded.primary_ipv4,
                gateway = excluded.gateway,
                dns_json = excluded.dns_json,
                route_metric = excluded.route_metric,
                interfaces_json = excluded.interfaces_json,
                last_seen_at = excluded.last_seen_at,
                observation_count = baseline_network_configurations.observation_count + 1",
            params![
                baseline.baseline_id,
                value.primary_interface_key,
                value.primary_interface_type,
                value.primary_ipv4,
                value.gateway,
                dns,
                value.route_metric,
                interfaces,
                now
            ],
        )
        .map(|_| ())
        .map_err(|_| "Unable to learn network configuration".into())
}

fn detect_network(
    transaction: &Transaction<'_>,
    baseline: &BaselineRow,
    network: &NetworkInfo,
    now: &str,
) -> Result<(), String> {
    let current = network_value(network);
    let stored = load_network(transaction, baseline)?;
    if stored.is_none() {
        return learn_network(transaction, baseline, network, now);
    }
    let mut seen = HashSet::new();
    if let Some(stored) = stored {
        if stored.primary_interface_key != current.primary_interface_key
            || stored.primary_ipv4 != current.primary_ipv4
            || stored.route_metric != current.route_metric
        {
            seen.insert(upsert_event(
                transaction,
                baseline,
                "primary_route_changed",
                "network_configuration",
                "primary-route",
                "Primary route changed",
                now,
                "system",
                json!({
                    "beforeInterface": stored.primary_interface_key,
                    "afterInterface": current.primary_interface_key,
                    "beforeAddress": stored.primary_ipv4,
                    "afterAddress": current.primary_ipv4,
                    "beforeMetric": stored.route_metric,
                    "afterMetric": current.route_metric,
                    "interfaceType": current.primary_interface_type,
                    "firstObserved": now
                }),
                json!({"comparison": "current Windows best route differs from baseline", "baselineVersion": baseline.version}),
                Some("high"),
            )?);
        }
        if stored.gateway != current.gateway {
            seen.insert(upsert_event(
                transaction,
                baseline,
                "gateway_changed",
                "network_configuration",
                "default-gateway",
                "Default gateway changed",
                now,
                "system",
                json!({"before": stored.gateway, "after": current.gateway, "firstObserved": now}),
                json!({"comparison": "current gateway differs from baseline", "baselineVersion": baseline.version}),
                Some("high"),
            )?);
        }
        if stored.dns != current.dns {
            seen.insert(upsert_event(
                transaction,
                baseline,
                "dns_changed",
                "network_configuration",
                "dns-resolvers",
                "DNS resolver configuration changed",
                now,
                "system",
                json!({"before": stored.dns, "after": current.dns, "firstObserved": now}),
                json!({"comparison": "current DNS resolver set differs from baseline", "baselineVersion": baseline.version}),
                Some("high"),
            )?);
        }
    }
    finish_event_cycle(
        transaction,
        baseline,
        &["primary_route_changed", "gateway_changed", "dns_changed"],
        &seen,
    )
}

fn upsert_executable(
    transaction: &Transaction<'_>,
    baseline: &BaselineRow,
    process: &ProcessRecord,
    identity: &ExecutableIdentity,
    now: &str,
) -> Result<(), String> {
    transaction
        .execute(
            "INSERT INTO baseline_executables(
                baseline_id, executable_key, normalized_path, process_name, company_name,
                signer_name, signature_status, architecture, file_size, file_modified_at,
                first_seen_at, last_seen_at, observation_count
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11, 1)
             ON CONFLICT(baseline_id, executable_key) DO UPDATE SET
                company_name = COALESCE(excluded.company_name, baseline_executables.company_name),
                signer_name = COALESCE(excluded.signer_name, baseline_executables.signer_name),
                signature_status = excluded.signature_status,
                last_seen_at = excluded.last_seen_at,
                observation_count = baseline_executables.observation_count + 1",
            params![
                baseline.baseline_id,
                identity.key,
                identity.normalized_path,
                process.name,
                process.company,
                process.signer,
                process.signature_status,
                process.architecture,
                process.executable_file_size,
                process.executable_modified_at,
                now
            ],
        )
        .map(|_| ())
        .map_err(|_| "Unable to learn executable baseline".into())
}

struct DestinationIdentity {
    key: String,
    executable_key: Option<String>,
    process_name: Option<String>,
    remote_ip: String,
    remote_port: u16,
    protocol: String,
}

fn destination_identity(
    connection: &ConnectionRecord,
    processes: &HashMap<u32, &ProcessRecord>,
) -> Option<DestinationIdentity> {
    let remote_ip = connection.remote_address.as_ref()?.clone();
    let remote_port = connection.remote_port?;
    let process = connection.pid.and_then(|pid| processes.get(&pid).copied());
    let executable_key = process
        .map(executable_identity)
        .map(|identity| identity.key);
    let process_name = process
        .map(|value| value.name.clone())
        .or_else(|| connection.process_name.clone());
    let owner = executable_key
        .as_deref()
        .or(process_name.as_deref())
        .unwrap_or("unresolved-owner");
    Some(DestinationIdentity {
        key: stable_hash(&format!(
            "destination|{owner}|{}|{remote_ip}|{remote_port}",
            connection.protocol
        )),
        executable_key,
        process_name,
        remote_ip,
        remote_port,
        protocol: connection.protocol.clone(),
    })
}

fn executable_identity(process: &ProcessRecord) -> ExecutableIdentity {
    let normalized_path = process
        .executable_path
        .as_deref()
        .map(normalize_windows_path);
    let material = format!(
        "executable-v1|{}|{}|{}|{}|{}|{}",
        normalized_path.as_deref().unwrap_or("path-unavailable"),
        process.executable_file_size.unwrap_or(0),
        process
            .executable_modified_at
            .as_deref()
            .unwrap_or("mtime-unavailable"),
        process.signer.as_deref().unwrap_or("signer-unavailable"),
        process.signature_status,
        process
            .architecture
            .as_deref()
            .unwrap_or("architecture-unavailable")
    );
    ExecutableIdentity {
        key: stable_hash(&material),
        normalized_path,
    }
}

fn process_pattern_key(
    executable_key: &str,
    parent_executable_key: Option<&str>,
    user: Option<&str>,
) -> String {
    stable_hash(&format!(
        "process-pattern-v1|{executable_key}|{}|{}",
        parent_executable_key.unwrap_or("parent-unavailable"),
        user.unwrap_or("user-unavailable").to_lowercase()
    ))
}

fn relationship_key(parent: &str, child: &str) -> String {
    stable_hash(&format!("parent-child-v1|{parent}|{child}"))
}

fn network_value(network: &NetworkInfo) -> StoredNetwork {
    let primary = network.interfaces.iter().find(|item| item.primary_route);
    let primary_interface_key = primary.map(|item| {
        stable_hash(&format!(
            "interface-v1|{}|{}|{}",
            item.name.to_lowercase(),
            item.description.to_lowercase(),
            item.interface_type.to_lowercase()
        ))
    });
    let mut dns = network.dns_servers.clone();
    dns.sort();
    dns.dedup();
    StoredNetwork {
        primary_interface_key,
        primary_interface_type: network.primary_interface_type.clone(),
        primary_ipv4: network.primary_ipv4.clone(),
        gateway: network.primary_gateway.clone(),
        dns,
        route_metric: network.primary_route_metric,
    }
}

fn load_network(
    connection: &rusqlite::Connection,
    baseline: &BaselineRow,
) -> Result<Option<StoredNetwork>, String> {
    connection
        .query_row(
            "SELECT primary_interface_key, primary_interface_type, primary_ipv4,
                    gateway, dns_json, route_metric
             FROM baseline_network_configurations WHERE baseline_id = ?1",
            [&baseline.baseline_id],
            |row| {
                let dns: String = row.get(4)?;
                Ok(StoredNetwork {
                    primary_interface_key: row.get(0)?,
                    primary_interface_type: row.get(1)?,
                    primary_ipv4: row.get(2)?,
                    gateway: row.get(3)?,
                    dns: serde_json::from_str(&dns).unwrap_or_default(),
                    route_metric: row.get(5)?,
                })
            },
        )
        .optional()
        .map_err(|_| "Unable to read network baseline".into())
}

fn load_keys(
    connection: &rusqlite::Connection,
    table: &str,
    column: &str,
    baseline: &BaselineRow,
) -> Result<HashSet<String>, String> {
    let mut statement = connection
        .prepare(&format!(
            "SELECT {column} FROM {table} WHERE baseline_id = ?1"
        ))
        .map_err(|_| "Unable to prepare baseline identity query")?;
    let rows = statement
        .query_map([&baseline.baseline_id], |row| row.get(0))
        .map_err(|_| "Unable to query baseline identities")?;
    rows.collect::<Result<HashSet<_>, _>>()
        .map_err(|_| "Unable to read baseline identities".into())
}

fn load_services(
    connection: &rusqlite::Connection,
    baseline: &BaselineRow,
) -> Result<HashMap<String, StoredService>, String> {
    let mut statement = connection
        .prepare(
            "SELECT service_key, service_name, display_name, startup_type, binary_path,
                    account_name FROM baseline_services WHERE baseline_id = ?1",
        )
        .map_err(|_| "Unable to prepare service baseline query")?;
    let rows = statement
        .query_map([&baseline.baseline_id], |row| {
            Ok((
                row.get(0)?,
                StoredService {
                    service_name: row.get(1)?,
                    display_name: row.get(2)?,
                    startup_type: row.get(3)?,
                    binary_path: row.get(4)?,
                    account: row.get(5)?,
                },
            ))
        })
        .map_err(|_| "Unable to query service baseline")?;
    rows.collect::<Result<HashMap<_, _>, _>>()
        .map_err(|_| "Unable to read service baseline".into())
}

#[allow(clippy::too_many_arguments)]
fn upsert_event(
    transaction: &Transaction<'_>,
    baseline: &BaselineRow,
    event_type: &str,
    entity_type: &str,
    entity_key: &str,
    title: &str,
    now: &str,
    source: &str,
    evidence: Value,
    baseline_context: Value,
    confidence: Option<&str>,
) -> Result<String, String> {
    let event_id = format!(
        "security-{}",
        &stable_hash(&format!(
            "event-v1|{}|{event_type}|{entity_key}",
            baseline.baseline_id
        ))[..32]
    );
    let evidence = serde_json::to_string(&evidence).map_err(|_| "Unable to serialize evidence")?;
    let baseline_context = serde_json::to_string(&baseline_context)
        .map_err(|_| "Unable to serialize baseline context")?;
    transaction
        .execute(
            "INSERT INTO security_events(
                id, event_type, occurred_at, source, payload_json, entity_type, entity_key,
                title, first_seen_at, last_seen_at, baseline_context_json, baseline_id,
                rule_id, confidence, status, observation_count, condition_active, schema_version
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?3, ?3, ?9, ?10, NULL, ?11,
                       'new', 1, 1, ?12)
             ON CONFLICT(id) DO UPDATE SET
                occurred_at = excluded.occurred_at,
                last_seen_at = excluded.last_seen_at,
                payload_json = excluded.payload_json,
                baseline_context_json = excluded.baseline_context_json,
                confidence = excluded.confidence,
                status = CASE
                    WHEN security_events.condition_active = 0
                         AND security_events.status = 'resolved' THEN 'new'
                    ELSE security_events.status
                END,
                observation_count = security_events.observation_count + 1,
                condition_active = 1",
            params![
                event_id,
                event_type,
                now,
                source,
                evidence,
                entity_type,
                entity_key,
                title,
                baseline_context,
                baseline.baseline_id,
                confidence,
                EVENT_SCHEMA_VERSION
            ],
        )
        .map_err(|_| "Unable to persist factual security event")?;
    Ok(event_id)
}

fn finish_event_cycle(
    transaction: &Transaction<'_>,
    baseline: &BaselineRow,
    categories: &[&str],
    seen: &HashSet<String>,
) -> Result<(), String> {
    let mut statement = transaction
        .prepare("SELECT id, event_type FROM security_events WHERE baseline_id = ?1 AND condition_active = 1")
        .map_err(|_| "Unable to prepare event activity query")?;
    let rows = statement
        .query_map([&baseline.baseline_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|_| "Unable to query event activity")?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Unable to read event activity")?;
    drop(statement);
    for (event_id, event_type) in rows {
        if categories.contains(&event_type.as_str()) && !seen.contains(&event_id) {
            transaction
                .execute(
                    "UPDATE security_events SET condition_active = 0 WHERE id = ?1",
                    [&event_id],
                )
                .map_err(|_| "Unable to update event activity")?;
        }
    }
    Ok(())
}

fn update_observation(
    transaction: &Transaction<'_>,
    baseline: &BaselineRow,
    now: &str,
) -> Result<(), String> {
    transaction
        .execute(
            "UPDATE behavioral_baselines
             SET observation_count = observation_count + 1,
                 last_observed_at = ?1,
                 status = CASE WHEN status = 'stale' THEN 'ready' ELSE status END,
                 error_message = NULL
             WHERE baseline_id = ?2 AND active = 1",
            params![now, baseline.baseline_id],
        )
        .map(|_| ())
        .map_err(|_| "Unable to update baseline observation".into())
}

fn complete_if_due(
    transaction: &Transaction<'_>,
    baseline: &BaselineRow,
    now: &str,
) -> Result<(), String> {
    if baseline.status != BaselineStatus::Learning {
        return Ok(());
    }
    let elapsed = parse_utc(&baseline.learning_started_at)
        .map(|started| {
            Utc::now()
                .signed_duration_since(started)
                .num_seconds()
                .max(0) as u64
        })
        .unwrap_or(0);
    if elapsed < baseline.learning_period_seconds {
        return Ok(());
    }
    transaction
        .execute(
            "UPDATE behavioral_baselines
             SET status = 'ready', learning_completed_at = ?1
             WHERE baseline_id = ?2 AND active = 1 AND observation_count > 0",
            params![now, baseline.baseline_id],
        )
        .map(|_| ())
        .map_err(|_| "Unable to finalize elapsed baseline learning".into())
}

fn enforce_event_retention(transaction: &Transaction<'_>) -> Result<(), String> {
    let cutoff = (Utc::now() - ChronoDuration::days(SECURITY_EVENT_RETENTION_DAYS)).to_rfc3339();
    let maximum_cutoff =
        (Utc::now() - ChronoDuration::days(SECURITY_EVENT_MAX_RETENTION_DAYS)).to_rfc3339();
    transaction
        .execute(
            "DELETE FROM security_events
             WHERE condition_active = 0 AND status IN ('resolved', 'ignored')
               AND COALESCE(last_seen_at, occurred_at) < ?1",
            [&cutoff],
        )
        .map(|_| ())
        .map_err(|_| "Unable to enforce security event retention")?;
    transaction
        .execute(
            "DELETE FROM security_events
             WHERE condition_active = 0 AND COALESCE(last_seen_at, occurred_at) < ?1",
            [&maximum_cutoff],
        )
        .map(|_| ())
        .map_err(|_| "Unable to enforce maximum security event retention".into())
}

fn service_evidence(service: &ServiceRecord, now: &str) -> Value {
    json!({
        "serviceName": service.service_name,
        "displayName": service.display_name,
        "status": service.status,
        "startupType": service.startup_type,
        "binaryPath": service.binary_path,
        "account": service.account,
        "firstObserved": now
    })
}

fn status_from_db(status: &str) -> BaselineStatus {
    match status {
        "learning" => BaselineStatus::Learning,
        "ready" => BaselineStatus::Ready,
        "stale" => BaselineStatus::Stale,
        "error" => BaselineStatus::Error,
        _ => BaselineStatus::Error,
    }
}

fn parse_utc(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|parsed| parsed.with_timezone(&Utc))
}

fn normalize_windows_path(value: &str) -> String {
    value.trim().replace('/', "\\").to_lowercase()
}

fn stable_hash(value: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(value.as_bytes());
    format!("{:x}", digest.finalize())
}

fn derive_host_id(machine_guid: &str, volume_serial: u32) -> String {
    format!(
        "host-v1-{}",
        &stable_hash(&format!(
            "edy-sentinel-host-v1|{}|{volume_serial:08x}",
            machine_guid.trim().to_lowercase()
        ))[..32]
    )
}

fn current_host_id() -> Result<String, String> {
    let machine_guid: String = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey("SOFTWARE\\Microsoft\\Cryptography")
        .and_then(|key| key.get_value("MachineGuid"))
        .map_err(|_| "Windows machine identity is unavailable")?;
    let volume_serial = system_volume_serial()?;
    Ok(derive_host_id(&machine_guid, volume_serial))
}

fn system_volume_serial() -> Result<u32, String> {
    let root = wide(OsStr::new("C:\\"));
    let mut serial = 0u32;
    let success = unsafe {
        GetVolumeInformationW(
            root.as_ptr(),
            std::ptr::null_mut(),
            0,
            &mut serial,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
        )
    };
    if success == 0 {
        Err("Windows system volume identity is unavailable".into())
    } else {
        Ok(serial)
    }
}

fn wide(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::{
        derive_host_id, detect_network, load_active_baseline, normalize_windows_path,
        process_pattern_key, relationship_key, stable_hash, status_from_db, BaselineEngine,
    };
    use crate::{
        models::{
            BaselineActionInput, BaselineStatus, CollectionIssue, CollectorHealth, CollectorStatus,
            ConnectionRecord, LiveTelemetrySnapshot, NetworkInfo, ProcessRecord,
            SecurityEventStatusInput, ServiceRecord,
        },
        persistence::Database,
    };
    use chrono::DateTime;

    fn process(key: &str, name: &str, pid: u32, parent_pid: Option<u32>) -> ProcessRecord {
        ProcessRecord {
            key: key.into(),
            name: name.into(),
            pid,
            parent_pid,
            user: Some("local-user".into()),
            executable_path: Some(format!("C:\\Apps\\{name}")),
            command_line: None,
            cpu_percent: Some(1.0),
            core_equivalent_cpu_percent: Some(4.0),
            memory_bytes: 1024,
            start_time: Some("2026-08-16T12:00:00Z".into()),
            thread_count: Some(2),
            architecture: Some("x64".into()),
            description: None,
            company: Some("Example Company".into()),
            signature_status: "signed".into(),
            signer: Some("Example Signer".into()),
            executable_file_size: Some(2048),
            executable_modified_at: Some("2026-08-16T10:00:00Z".into()),
            access_status: "available".into(),
            first_seen: "2026-08-16T12:00:00Z".into(),
            last_seen: "2026-08-16T12:00:00Z".into(),
            observation_count: 1,
            active: true,
        }
    }

    fn snapshot(include_new_executable: bool, changed_service: bool) -> LiveTelemetrySnapshot {
        let mut processes = vec![
            process("10:1", "parent.exe", 10, None),
            process("11:1", "child.exe", 11, Some(10)),
        ];
        if include_new_executable {
            processes.push(process("12:1", "new-tool.exe", 12, Some(10)));
        }
        LiveTelemetrySnapshot {
            collected_at: "2026-08-16T12:00:00Z".into(),
            processes,
            connections: vec![ConnectionRecord {
                key: "tcp|ipv4|127.0.0.1|5000|192.0.2.1|443|11".into(),
                protocol: "tcp".into(),
                ip_version: "ipv4".into(),
                local_address: "127.0.0.1".into(),
                local_port: 5000,
                remote_address: Some("192.0.2.1".into()),
                remote_port: Some(443),
                state: Some("Established".into()),
                pid: Some(11),
                process_name: Some("child.exe".into()),
                executable_path: Some("C:\\Apps\\child.exe".into()),
                association_status: "associated".into(),
                process_last_seen: None,
                first_seen: "2026-08-16T12:00:00Z".into(),
                last_seen: "2026-08-16T12:00:00Z".into(),
                observation_count: 1,
                active: true,
            }],
            services: vec![ServiceRecord {
                key: "sample".into(),
                service_name: "Sample".into(),
                display_name: "Sample Service".into(),
                status: "Running".into(),
                startup_type: if changed_service {
                    "Automatic"
                } else {
                    "Manual"
                }
                .into(),
                binary_path: Some("C:\\Apps\\service.exe".into()),
                account: Some("LocalSystem".into()),
                pid: Some(42),
                first_seen: "2026-08-16T12:00:00Z".into(),
                last_seen: "2026-08-16T12:00:00Z".into(),
                observation_count: 1,
                active: true,
            }],
            events: Vec::new(),
            collectors: vec![CollectorHealth {
                id: "processes".into(),
                status: CollectorStatus::Healthy,
                detail: "test".into(),
                last_success: Some("2026-08-16T12:00:00Z".into()),
                last_attempt: "2026-08-16T12:00:00Z".into(),
                duration_ms: 1,
                observation_count: 2,
                restricted_count: 0,
                error_code: None,
                error_message: None,
            }],
            issues: Vec::<CollectionIssue>::new(),
        }
    }

    fn start(engine: &BaselineEngine, database: &Database) {
        engine
            .start_new_baseline(
                database,
                BaselineActionInput {
                    confirmation: "START NEW BASELINE".into(),
                    learning_period_seconds: Some(60),
                },
            )
            .expect("baseline should start");
    }

    fn complete(engine: &BaselineEngine, database: &Database) {
        engine
            .complete_learning(
                database,
                BaselineActionInput {
                    confirmation: "COMPLETE BASELINE".into(),
                    learning_period_seconds: None,
                },
            )
            .expect("baseline should complete");
    }

    #[test]
    fn host_identity_is_stable_and_does_not_embed_source_identifiers() {
        let first = derive_host_id("machine-guid-value", 0x1234_abcd);
        let second = derive_host_id("machine-guid-value", 0x1234_abcd);
        assert_eq!(first, second);
        assert!(!first.contains("machine-guid-value"));
        assert!(!first.contains("1234"));
    }

    #[test]
    fn executable_paths_are_normalized_for_windows_identity() {
        assert_eq!(
            normalize_windows_path(" C:/Program Files/App/APP.EXE "),
            "c:\\program files\\app\\app.exe"
        );
    }

    #[test]
    fn process_and_parent_child_keys_are_deterministic() {
        let process = process_pattern_key("child", Some("parent"), Some("LOCAL\\User"));
        assert_eq!(
            process,
            process_pattern_key("child", Some("parent"), Some("local\\user"))
        );
        assert_eq!(
            relationship_key("parent", "child"),
            relationship_key("parent", "child")
        );
        assert_ne!(
            relationship_key("parent", "child"),
            relationship_key("child", "parent")
        );
    }

    #[test]
    fn baseline_states_remain_explicit() {
        assert_eq!(status_from_db("learning"), BaselineStatus::Learning);
        assert_eq!(status_from_db("ready"), BaselineStatus::Ready);
        assert_eq!(status_from_db("unexpected"), BaselineStatus::Error);
        assert_eq!(stable_hash("fact").len(), 64);
    }

    #[test]
    fn learning_persists_across_engine_restart_and_completes_after_real_observation() {
        let database = Database::in_memory().expect("database");
        let engine = BaselineEngine::default();
        start(&engine, &database);
        engine
            .observe_live(&database, &snapshot(false, false))
            .expect("learning observation");
        assert_eq!(engine.security_events(&database).expect("events").len(), 0);
        let recovered = BaselineEngine::default();
        let learning = recovered.summary(&database).expect("recovered summary");
        assert_eq!(learning.status, BaselineStatus::Learning);
        assert_eq!(learning.observation_count, 1);
        assert_eq!(learning.entities.executables, 2);
        assert_eq!(learning.entities.parent_child_relationships, 1);
        assert_eq!(learning.entities.network_destinations, 1);
        complete(&recovered, &database);
        assert_eq!(
            recovered.summary(&database).expect("ready").status,
            BaselineStatus::Ready
        );
    }

    #[test]
    fn reset_starts_a_new_version_without_deleting_baseline_history() {
        let database = Database::in_memory().expect("database");
        let engine = BaselineEngine::default();
        start(&engine, &database);
        engine
            .observe_live(&database, &snapshot(false, false))
            .expect("learn first version");
        engine
            .reset_baseline(
                &database,
                BaselineActionInput {
                    confirmation: "RESET BASELINE".into(),
                    learning_period_seconds: Some(60),
                },
            )
            .expect("reset baseline");

        let current = engine.summary(&database).expect("current baseline");
        assert_eq!(current.version, Some(2));
        assert_eq!(current.status, BaselineStatus::Learning);
        let (versions, active): (u32, u32) = database
            .baseline_read(|connection| {
                connection
                    .query_row(
                        "SELECT COUNT(*), SUM(active) FROM behavioral_baselines",
                        [],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .map_err(|_| "Unable to verify preserved baseline history".into())
            })
            .expect("baseline history");
        assert_eq!(versions, 2);
        assert_eq!(active, 1);
    }

    #[test]
    fn first_ready_network_observation_seeds_missing_network_baseline_without_event() {
        let database = Database::in_memory().expect("database");
        let engine = BaselineEngine::default();
        start(&engine, &database);
        engine
            .observe_live(&database, &snapshot(false, false))
            .expect("learn live families");
        complete(&engine, &database);

        database
            .baseline_transaction(|transaction| {
                let row = load_active_baseline(transaction)?
                    .ok_or_else(|| "Missing active baseline".to_string())?;
                detect_network(
                    transaction,
                    &row,
                    &NetworkInfo::default(),
                    "2026-08-16T12:05:00Z",
                )
            })
            .expect("seed missing network baseline");
        let summary = engine.summary(&database).expect("summary");
        assert_eq!(summary.entities.network_configurations, 1);
        assert!(engine
            .security_events(&database)
            .expect("events")
            .is_empty());
    }

    #[test]
    fn factual_event_is_deduplicated_and_resolved_event_reopens_after_reappearance() {
        let database = Database::in_memory().expect("database");
        let engine = BaselineEngine::default();
        start(&engine, &database);
        engine
            .observe_live(&database, &snapshot(false, false))
            .expect("learn");
        complete(&engine, &database);

        BaselineEngine::default()
            .observe_live(&database, &snapshot(true, false))
            .expect("detect");
        let first = BaselineEngine::default()
            .security_events(&database)
            .expect("events");
        let executable = first
            .iter()
            .find(|event| event.event_type == "executable_first_seen")
            .expect("executable event")
            .clone();
        BaselineEngine::default()
            .observe_live(&database, &snapshot(true, false))
            .expect("deduplicate");
        let repeated = BaselineEngine::default()
            .security_events(&database)
            .expect("events");
        let same = repeated
            .iter()
            .find(|event| event.event_id == executable.event_id)
            .expect("same event");
        assert_eq!(same.observation_count, 2);

        engine
            .set_event_status(
                &database,
                SecurityEventStatusInput {
                    event_id: executable.event_id.clone(),
                    status: "resolved".into(),
                },
            )
            .expect("resolve");
        BaselineEngine::default()
            .observe_live(&database, &snapshot(false, false))
            .expect("condition disappears");
        BaselineEngine::default()
            .observe_live(&database, &snapshot(true, false))
            .expect("condition reappears");
        let reopened = BaselineEngine::default()
            .security_events(&database)
            .expect("events")
            .into_iter()
            .find(|event| event.event_id == executable.event_id)
            .expect("reopened event");
        assert_eq!(reopened.status, "new");
        assert!(reopened.observation_count >= 3);
        assert!(
            DateTime::parse_from_rfc3339(&reopened.last_seen).expect("last timestamp")
                >= DateTime::parse_from_rfc3339(&reopened.first_seen).expect("first timestamp")
        );
    }

    #[test]
    fn ready_service_change_creates_factual_evidence_without_severity() {
        let database = Database::in_memory().expect("database");
        let engine = BaselineEngine::default();
        start(&engine, &database);
        engine
            .observe_live(&database, &snapshot(false, false))
            .expect("learn");
        complete(&engine, &database);
        BaselineEngine::default()
            .observe_live(&database, &snapshot(false, true))
            .expect("detect service change");
        let event = BaselineEngine::default()
            .security_events(&database)
            .expect("events")
            .into_iter()
            .find(|event| event.event_type == "service_startup_changed")
            .expect("service event");
        assert_eq!(event.evidence["before"], "Manual");
        assert_eq!(event.evidence["after"], "Automatic");
        assert!(event.rule_id.is_none());
        assert!(!event.title.to_lowercase().contains("malware"));
    }
}
