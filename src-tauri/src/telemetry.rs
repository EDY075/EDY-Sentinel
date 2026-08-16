use crate::{
    collectors::{
        connections::{self, RecentProcessIdentity},
        processes::ProcessCollector,
        services,
    },
    models::{
        CollectionIssue, CollectorHealth, CollectorStatus, ConnectionRecord, LiveTelemetrySnapshot,
        ProcessRecord, ServiceRecord, TelemetryEvent,
    },
};
use chrono::Utc;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const CONNECTION_INTERVAL: Duration = Duration::from_secs(4);
const SERVICE_INTERVAL: Duration = Duration::from_secs(15);
const RECENT_PROCESS_TTL: Duration = Duration::from_secs(10);

#[derive(Clone)]
pub struct TelemetryEngine {
    inner: Arc<Mutex<EngineState>>,
}

struct EngineState {
    process_collector: ProcessCollector,
    processes: HashMap<String, ProcessRecord>,
    connections: HashMap<String, ConnectionRecord>,
    services: HashMap<String, ServiceRecord>,
    recent_processes: HashMap<u32, RecentProcessCacheEntry>,
    connection_misses: HashMap<String, u8>,
    process_baseline: bool,
    connection_baseline: bool,
    service_baseline: bool,
    last_connection_collection: Option<Instant>,
    last_service_collection: Option<Instant>,
    process_health: Option<CollectorHealth>,
    connection_health: Option<CollectorHealth>,
    service_health: Option<CollectorHealth>,
}

#[derive(Clone)]
struct RecentProcessCacheEntry {
    process: ProcessRecord,
    exited_at: Instant,
}

impl Default for TelemetryEngine {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(EngineState {
                process_collector: ProcessCollector::default(),
                processes: HashMap::new(),
                connections: HashMap::new(),
                services: HashMap::new(),
                recent_processes: HashMap::new(),
                connection_misses: HashMap::new(),
                process_baseline: false,
                connection_baseline: false,
                service_baseline: false,
                last_connection_collection: None,
                last_service_collection: None,
                process_health: None,
                connection_health: None,
                service_health: None,
            })),
        }
    }
}

impl TelemetryEngine {
    pub fn collect(&self) -> Result<LiveTelemetrySnapshot, String> {
        let mut state = self
            .inner
            .lock()
            .map_err(|_| "Telemetry engine unavailable".to_string())?;
        let collected_at = Utc::now().to_rfc3339();
        let mut events = Vec::new();
        let mut issues = Vec::new();
        let mut collectors = Vec::new();

        let process_started = Instant::now();
        let (processes, process_restricted, process_issues) = state.process_collector.collect();
        let process_failed = processes.is_empty();
        issues.extend(process_issues.clone());
        if process_failed {
            issues.push(CollectionIssue {
                component: "processes".into(),
                message: "Windows returned no process records; the previous process baseline was retained"
                    .into(),
            });
        } else {
            let baseline = state.process_baseline;
            let previous = std::mem::take(&mut state.processes);
            state.processes =
                track_processes(previous, processes, baseline, &collected_at, &mut events);
            state.process_baseline = true;
        }
        let recently_stopped = state
            .processes
            .values()
            .filter(|process| !process.active)
            .cloned()
            .collect::<Vec<_>>();
        for process in recently_stopped {
            state.recent_processes.insert(
                process.pid,
                RecentProcessCacheEntry {
                    process,
                    exited_at: Instant::now(),
                },
            );
        }
        state
            .recent_processes
            .retain(|_, value| value.exited_at.elapsed() <= RECENT_PROCESS_TTL);
        let process_observations = state.processes.values().filter(|item| item.active).count();
        let process_health = health(
            "processes",
            if process_failed {
                CollectorStatus::Failed
            } else if process_issues.is_empty() {
                CollectorStatus::Healthy
            } else {
                CollectorStatus::Degraded
            },
            if process_failed {
                "Previous process baseline retained"
            } else {
                "Native Windows process telemetry completed"
            },
            &collected_at,
            process_started.elapsed(),
            process_observations,
            process_restricted,
            process_failed.then_some("EMPTY_PROCESS_SNAPSHOT"),
            process_failed.then_some("Windows returned no process records"),
            state.process_health.as_ref(),
        );
        state.process_health = Some(process_health.clone());
        collectors.push(process_health);

        let connection_due = state
            .last_connection_collection
            .map_or(true, |last| last.elapsed() >= CONNECTION_INTERVAL);
        let mut connection_output = state.connections.values().cloned().collect::<Vec<_>>();
        if connection_due {
            let connection_started = Instant::now();
            let process_values = state
                .processes
                .values()
                .filter(|item| item.active)
                .cloned()
                .collect::<Vec<_>>();
            let recent_values = state
                .recent_processes
                .values()
                .map(|value| RecentProcessIdentity {
                    key: value.process.key.clone(),
                    pid: value.process.pid,
                    name: value.process.name.clone(),
                    executable_path: value.process.executable_path.clone(),
                    last_seen: value.process.last_seen.clone(),
                    age: value.exited_at.elapsed(),
                })
                .collect::<Vec<_>>();
            let (connections, connection_issues) =
                connections::collect(&process_values, &recent_values);
            let failed_families = failed_connection_families(&connection_issues);
            let connection_failed = failed_families.len() == 4;
            let baseline = state.connection_baseline;
            let previous = std::mem::take(&mut state.connections);
            let (next, visible) = track_connections(
                previous,
                connections,
                &failed_families,
                &mut state.connection_misses,
                baseline,
                &collected_at,
                &mut events,
            );
            state.connections = next;
            connection_output = visible;
            state.connection_baseline = true;
            state.last_connection_collection = Some(Instant::now());
            issues.extend(connection_issues.clone());
            let connection_health = health(
                "connections",
                if connection_failed {
                    CollectorStatus::Failed
                } else if connection_issues.is_empty() {
                    CollectorStatus::Healthy
                } else {
                    CollectorStatus::Degraded
                },
                if connection_failed {
                    "Previous network baseline retained"
                } else if connection_issues.is_empty() {
                    "IP Helper TCP/UDP IPv4/IPv6 tables"
                } else {
                    "Available protocol families collected; failed baselines retained"
                },
                &collected_at,
                connection_started.elapsed(),
                connection_output.iter().filter(|item| item.active).count(),
                0,
                connection_failed.then_some("IP_HELPER_TABLES_UNAVAILABLE"),
                connection_failed.then_some("All IP Helper protocol families failed"),
                state.connection_health.as_ref(),
            );
            state.connection_health = Some(connection_health.clone());
            collectors.push(connection_health);
        } else {
            if let Some(value) = state.connection_health.clone() {
                collectors.push(value);
            }
        }

        let service_due = state
            .last_service_collection
            .map_or(true, |last| last.elapsed() >= SERVICE_INTERVAL);
        let mut service_output = state.services.values().cloned().collect::<Vec<_>>();
        if service_due {
            let service_started = Instant::now();
            let (services, service_restricted, service_issues) = services::collect();
            let service_failed = services.is_empty() && !service_issues.is_empty();
            if !service_failed {
                let baseline = state.service_baseline;
                let previous = std::mem::take(&mut state.services);
                state.services =
                    track_services(previous, services, baseline, &collected_at, &mut events);
                service_output = state.services.values().cloned().collect();
                state.service_baseline = true;
            }
            state.last_service_collection = Some(Instant::now());
            issues.extend(service_issues.clone());
            let service_health = health(
                "services",
                if service_failed {
                    CollectorStatus::Failed
                } else if service_issues.is_empty() {
                    CollectorStatus::Healthy
                } else {
                    CollectorStatus::Degraded
                },
                if service_failed {
                    "Previous service baseline retained"
                } else if service_issues.is_empty() {
                    "Windows Service Control Manager telemetry completed"
                } else {
                    "Available service data collected"
                },
                &collected_at,
                service_started.elapsed(),
                service_output.iter().filter(|item| item.active).count(),
                service_restricted,
                service_failed.then_some("SCM_ENUMERATION_FAILED"),
                service_failed.then_some("Windows Service Control Manager enumeration failed"),
                state.service_health.as_ref(),
            );
            state.service_health = Some(service_health.clone());
            collectors.push(service_health);
        } else {
            if let Some(value) = state.service_health.clone() {
                collectors.push(value);
            }
        }

        let mut process_output = state.processes.values().cloned().collect::<Vec<_>>();
        process_output
            .sort_by(|left, right| left.name.cmp(&right.name).then(left.pid.cmp(&right.pid)));
        connection_output.sort_by(|left, right| left.key.cmp(&right.key));
        service_output.sort_by(|left, right| left.display_name.cmp(&right.display_name));
        Ok(LiveTelemetrySnapshot {
            collected_at,
            processes: process_output,
            connections: connection_output,
            services: service_output,
            events,
            collectors,
            issues,
        })
    }
}

fn track_processes(
    previous: HashMap<String, ProcessRecord>,
    current: Vec<ProcessRecord>,
    baseline: bool,
    now: &str,
    events: &mut Vec<TelemetryEvent>,
) -> HashMap<String, ProcessRecord> {
    let mut current: Vec<_> = current
        .into_iter()
        .map(|item| (item.key.clone(), item))
        .collect::<HashMap<_, _>>()
        .into_values()
        .collect();
    for process in &mut current {
        if let Some(old) = previous.get(&process.key).filter(|old| old.active) {
            process.first_seen = old.first_seen.clone();
            process.last_seen = monotonic_timestamp(&old.last_seen, &process.last_seen);
            process.observation_count = old.observation_count.saturating_add(1);
        } else if baseline {
            events.push(event(
                "process_started",
                "process",
                &process.key,
                "New process observed",
                now,
            ));
        }
    }
    if baseline {
        let keys: HashSet<_> = current.iter().map(|item| item.key.clone()).collect();
        for old in previous
            .values()
            .filter(|item| item.active && !keys.contains(&item.key))
        {
            events.push(event(
                "process_stopped",
                "process",
                &old.key,
                "Process is no longer observed",
                now,
            ));
            let mut stopped = old.clone();
            stopped.active = false;
            stopped.last_seen = monotonic_timestamp(&old.last_seen, now);
            current.push(stopped);
        }
    }
    current
        .into_iter()
        .map(|item| (item.key.clone(), item))
        .collect()
}

fn track_connections(
    previous: HashMap<String, ConnectionRecord>,
    current: Vec<ConnectionRecord>,
    failed_families: &HashSet<(String, String)>,
    misses: &mut HashMap<String, u8>,
    baseline: bool,
    now: &str,
    events: &mut Vec<TelemetryEvent>,
) -> (HashMap<String, ConnectionRecord>, Vec<ConnectionRecord>) {
    let mut current: Vec<_> = current
        .into_iter()
        .map(|item| (item.key.clone(), item))
        .collect::<HashMap<_, _>>()
        .into_values()
        .collect();
    for connection in &mut current {
        misses.remove(&connection.key);
        if let Some(old) = previous.get(&connection.key).filter(|old| old.active) {
            connection.first_seen = old.first_seen.clone();
            connection.last_seen = monotonic_timestamp(&old.last_seen, &connection.last_seen);
            connection.observation_count = old.observation_count.saturating_add(1);
        } else if baseline {
            events.push(event(
                "connection_opened",
                "connection",
                &connection.key,
                "New connection observed",
                now,
            ));
        }
    }
    if baseline {
        let keys: HashSet<_> = current.iter().map(|item| item.key.clone()).collect();
        for old in previous.values().filter(|item| {
            item.active
                && !keys.contains(&item.key)
                && !failed_families.contains(&(item.protocol.clone(), item.ip_version.clone()))
        }) {
            let missed = misses.entry(old.key.clone()).or_insert(0);
            *missed = missed.saturating_add(1);
            if *missed >= 2 {
                events.push(event(
                    "connection_closed",
                    "connection",
                    &old.key,
                    "Connection is no longer observed",
                    now,
                ));
                let mut closed = old.clone();
                closed.active = false;
                closed.last_seen = monotonic_timestamp(&old.last_seen, now);
                current.push(closed);
                misses.remove(&old.key);
            } else {
                current.push(old.clone());
            }
        }
    }
    let mut next: HashMap<_, _> = current
        .into_iter()
        .map(|item| (item.key.clone(), item))
        .collect();
    for old in previous
        .into_values()
        .filter(|item| failed_families.contains(&(item.protocol.clone(), item.ip_version.clone())))
    {
        next.entry(old.key.clone()).or_insert(old);
    }
    let visible = next.values().cloned().collect();
    (next, visible)
}

fn track_services(
    previous: HashMap<String, ServiceRecord>,
    current: Vec<ServiceRecord>,
    baseline: bool,
    now: &str,
    events: &mut Vec<TelemetryEvent>,
) -> HashMap<String, ServiceRecord> {
    let mut current: Vec<_> = current
        .into_iter()
        .map(|item| (item.key.clone(), item))
        .collect::<HashMap<_, _>>()
        .into_values()
        .collect();
    for service in &mut current {
        if let Some(old) = previous.get(&service.key).filter(|old| old.active) {
            service.first_seen = old.first_seen.clone();
            service.last_seen = monotonic_timestamp(&old.last_seen, &service.last_seen);
            service.observation_count = old.observation_count.saturating_add(1);
            if baseline && old.status != service.status {
                let (kind, message) = if service.status == "Running" {
                    ("service_started", "Service entered the Running state")
                } else if service.status == "Stopped" {
                    ("service_stopped", "Service entered the Stopped state")
                } else {
                    ("service_state_changed", "Service state changed")
                };
                events.push(event(kind, "service", &service.key, message, now));
            }
            if baseline && old.startup_type != service.startup_type {
                events.push(event(
                    "startup_type_changed",
                    "service",
                    &service.key,
                    "Service startup type changed",
                    now,
                ));
            }
        } else if baseline && service.status == "Running" {
            events.push(event(
                "service_started",
                "service",
                &service.key,
                "New running service observed",
                now,
            ));
        }
    }
    if baseline {
        let keys: HashSet<_> = current.iter().map(|item| item.key.clone()).collect();
        for old in previous
            .values()
            .filter(|item| item.active && !keys.contains(&item.key))
        {
            events.push(event(
                "service_state_changed",
                "service",
                &old.key,
                "Service is no longer present in the current SCM enumeration",
                now,
            ));
            let mut missing = old.clone();
            missing.active = false;
            missing.last_seen = monotonic_timestamp(&old.last_seen, now);
            current.push(missing);
        }
    }
    current
        .into_iter()
        .map(|item| (item.key.clone(), item))
        .collect()
}

fn failed_connection_families(issues: &[CollectionIssue]) -> HashSet<(String, String)> {
    issues
        .iter()
        .filter_map(|issue| {
            issue
                .component
                .strip_prefix("connections-")
                .and_then(|suffix| suffix.split_once('-'))
                .map(|(protocol, version)| (protocol.into(), version.into()))
        })
        .collect()
}

fn event(
    event_type: &str,
    subject_type: &str,
    subject_key: &str,
    message: &str,
    occurred_at: &str,
) -> TelemetryEvent {
    let collector = match subject_type {
        "process" => "processes",
        "connection" => "connections",
        "service" => "services",
        _ => "system",
    };
    TelemetryEvent {
        event_id: format!("v1|{collector}|{event_type}|{subject_key}|{occurred_at}"),
        event_type: event_type.into(),
        entity_type: subject_type.into(),
        entity_key: subject_key.into(),
        timestamp: occurred_at.into(),
        collector: collector.into(),
        factual_payload: serde_json::json!({ "message": message }),
        schema_version: 1,
        message: message.into(),
    }
}

#[allow(clippy::too_many_arguments)]
fn health(
    id: &str,
    status: CollectorStatus,
    detail: &str,
    attempted_at: &str,
    duration: Duration,
    observation_count: usize,
    restricted_count: usize,
    error_code: Option<&str>,
    error_message: Option<&str>,
    previous: Option<&CollectorHealth>,
) -> CollectorHealth {
    let last_success = if status == CollectorStatus::Failed {
        previous.and_then(|value| value.last_success.clone())
    } else {
        Some(attempted_at.into())
    };
    CollectorHealth {
        id: id.into(),
        status,
        detail: detail.into(),
        last_success,
        last_attempt: attempted_at.into(),
        duration_ms: duration.as_millis() as u64,
        observation_count,
        restricted_count,
        error_code: error_code.map(Into::into),
        error_message: error_message.map(Into::into),
    }
}

fn monotonic_timestamp(previous: &str, candidate: &str) -> String {
    if candidate < previous {
        previous.into()
    } else {
        candidate.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn process(key: &str) -> ProcessRecord {
        ProcessRecord {
            key: key.into(),
            name: "sample.exe".into(),
            pid: 7,
            parent_pid: None,
            user: None,
            executable_path: None,
            command_line: None,
            cpu_percent: Some(1.0),
            core_equivalent_cpu_percent: Some(8.0),
            memory_bytes: 10,
            start_time: None,
            thread_count: Some(1),
            architecture: Some("x64".into()),
            description: None,
            company: None,
            signature_status: "unsigned".into(),
            signer: None,
            access_status: "partial".into(),
            first_seen: "first".into(),
            last_seen: "last".into(),
            observation_count: 1,
            active: true,
        }
    }

    #[test]
    fn first_snapshot_is_baseline_without_started_events() {
        let mut events = Vec::new();
        let result = track_processes(
            HashMap::new(),
            vec![process("7:1")],
            false,
            "now",
            &mut events,
        );
        assert_eq!(result.len(), 1);
        assert!(events.is_empty());
    }

    #[test]
    fn process_diff_tracks_start_and_stop_factually() {
        let previous = [("7:1".into(), process("7:1"))].into_iter().collect();
        let mut events = Vec::new();
        let result = track_processes(previous, vec![process("8:1")], true, "now", &mut events);
        assert_eq!(result.len(), 2);
        assert_eq!(events.len(), 2);
        assert!(events
            .iter()
            .any(|item| item.event_type == "process_started"));
        assert!(events
            .iter()
            .any(|item| item.event_type == "process_stopped"));
    }

    #[test]
    fn failed_connection_family_does_not_emit_mass_closures() {
        let old = ConnectionRecord {
            key: "tcp|ipv6|::1|80|-|-|7".into(),
            protocol: "tcp".into(),
            ip_version: "ipv6".into(),
            local_address: "::1".into(),
            local_port: 80,
            remote_address: None,
            remote_port: None,
            state: Some("listening".into()),
            pid: Some(7),
            process_name: None,
            executable_path: None,
            association_status: "unresolved".into(),
            process_last_seen: None,
            first_seen: "first".into(),
            last_seen: "last".into(),
            observation_count: 1,
            active: true,
        };
        let previous = [(old.key.clone(), old)].into_iter().collect();
        let failed = [("tcp".into(), "ipv6".into())].into_iter().collect();
        let mut events = Vec::new();
        let mut misses = HashMap::new();
        let (next, visible) = track_connections(
            previous,
            Vec::new(),
            &failed,
            &mut misses,
            true,
            "now",
            &mut events,
        );
        assert_eq!(next.len(), 1);
        assert_eq!(visible.len(), 1);
        assert!(events.is_empty());
    }

    #[test]
    fn connection_close_requires_two_missing_snapshots() {
        let old = ConnectionRecord {
            key: "tcp|ipv4|127.0.0.1|80|-|-|7".into(),
            protocol: "tcp".into(),
            ip_version: "ipv4".into(),
            local_address: "127.0.0.1".into(),
            local_port: 80,
            remote_address: None,
            remote_port: None,
            state: Some("listening".into()),
            pid: Some(7),
            process_name: None,
            executable_path: None,
            association_status: "unresolved".into(),
            process_last_seen: None,
            first_seen: "2026-08-16T00:00:00Z".into(),
            last_seen: "2026-08-16T00:00:01Z".into(),
            observation_count: 3,
            active: true,
        };
        let mut misses = HashMap::new();
        let mut events = Vec::new();
        let (first, _) = track_connections(
            [(old.key.clone(), old.clone())].into_iter().collect(),
            Vec::new(),
            &HashSet::new(),
            &mut misses,
            true,
            "2026-08-16T00:00:02Z",
            &mut events,
        );
        assert!(first[&old.key].active);
        assert!(events.is_empty());
        let (second, _) = track_connections(
            first,
            Vec::new(),
            &HashSet::new(),
            &mut misses,
            true,
            "2026-08-16T00:00:03Z",
            &mut events,
        );
        assert!(!second[&old.key].active);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn process_tracking_keeps_timestamps_monotonic_and_counts_once() {
        let mut old = process("7:1");
        old.first_seen = "2026-08-16T00:00:00Z".into();
        old.last_seen = "2026-08-16T00:00:02Z".into();
        old.observation_count = 4;
        let mut duplicate = process("7:1");
        duplicate.last_seen = "2026-08-16T00:00:01Z".into();
        let result = track_processes(
            [(old.key.clone(), old)].into_iter().collect(),
            vec![duplicate.clone(), duplicate],
            true,
            "2026-08-16T00:00:01Z",
            &mut Vec::new(),
        );
        assert_eq!(result["7:1"].last_seen, "2026-08-16T00:00:02Z");
        assert_eq!(result["7:1"].observation_count, 5);
    }

    #[test]
    fn collector_health_keeps_restricted_coverage_separate_from_status() {
        let value = health(
            "processes",
            CollectorStatus::Healthy,
            "completed",
            "2026-08-16T00:00:00Z",
            Duration::from_millis(5),
            262,
            17,
            None,
            None,
            None,
        );
        assert_eq!(value.status, CollectorStatus::Healthy);
        assert_eq!(value.restricted_count, 17);
        assert_eq!(value.observation_count, 262);
    }

    #[test]
    fn factual_events_have_stable_schema_fields_without_risk() {
        let value = event(
            "process_started",
            "process",
            "7:1",
            "New process observed",
            "2026-08-16T00:00:00Z",
        );
        assert_eq!(value.collector, "processes");
        assert_eq!(value.schema_version, 1);
        assert!(value
            .event_id
            .starts_with("v1|processes|process_started|7:1|"));
        assert!(value.factual_payload.get("message").is_some());
    }

    #[test]
    #[ignore = "manual Windows collector smoke test"]
    fn native_windows_collectors_smoke() {
        let engine = TelemetryEngine::default();
        let first = engine.collect().expect("native baseline should collect");
        assert!(first.processes.iter().any(|item| item.active));
        assert!(first.services.iter().any(|item| item.active));
        assert!(first
            .connections
            .iter()
            .all(|item| matches!(item.protocol.as_str(), "tcp" | "udp")));
        std::thread::sleep(std::time::Duration::from_millis(250));
        let second = engine
            .collect()
            .expect("second native snapshot should collect");
        assert!(second
            .processes
            .iter()
            .filter(|item| item.active)
            .all(|item| item.cpu_percent.is_some()));
    }
}
