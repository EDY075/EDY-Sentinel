use crate::{
    collectors::{connections, processes::ProcessCollector, services},
    models::{
        CollectionIssue, CollectorHealth, ConnectionRecord, LiveTelemetrySnapshot, ProcessRecord,
        ServiceRecord, TelemetryEvent,
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

#[derive(Clone)]
pub struct TelemetryEngine {
    inner: Arc<Mutex<EngineState>>,
}

struct EngineState {
    process_collector: ProcessCollector,
    processes: HashMap<String, ProcessRecord>,
    connections: HashMap<String, ConnectionRecord>,
    services: HashMap<String, ServiceRecord>,
    process_baseline: bool,
    connection_baseline: bool,
    service_baseline: bool,
    last_connection_collection: Option<Instant>,
    last_service_collection: Option<Instant>,
}

impl Default for TelemetryEngine {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(EngineState {
                process_collector: ProcessCollector::default(),
                processes: HashMap::new(),
                connections: HashMap::new(),
                services: HashMap::new(),
                process_baseline: false,
                connection_baseline: false,
                service_baseline: false,
                last_connection_collection: None,
                last_service_collection: None,
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

        let (processes, process_issues) = state.process_collector.collect();
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
        collectors.push(health(
            "processes",
            if process_failed || !process_issues.is_empty() {
                "partial"
            } else {
                "active"
            },
            if process_failed {
                "Previous process baseline retained"
            } else if process_issues.is_empty() {
                "Native Windows process telemetry"
            } else {
                "Process identity collected with access restrictions"
            },
            &collected_at,
        ));

        let connection_due = state
            .last_connection_collection
            .map_or(true, |last| last.elapsed() >= CONNECTION_INTERVAL);
        let mut connection_output = state.connections.values().cloned().collect::<Vec<_>>();
        if connection_due {
            let process_values = state
                .processes
                .values()
                .filter(|item| item.active)
                .cloned()
                .collect::<Vec<_>>();
            let (connections, connection_issues) = connections::collect(&process_values);
            let failed_families = failed_connection_families(&connection_issues);
            let baseline = state.connection_baseline;
            let previous = std::mem::take(&mut state.connections);
            let (next, visible) = track_connections(
                previous,
                connections,
                &failed_families,
                baseline,
                &collected_at,
                &mut events,
            );
            state.connections = next;
            connection_output = visible;
            state.connection_baseline = true;
            state.last_connection_collection = Some(Instant::now());
            issues.extend(connection_issues.clone());
            collectors.push(health(
                "connections",
                if connection_issues.is_empty() {
                    "active"
                } else {
                    "partial"
                },
                if connection_issues.is_empty() {
                    "IP Helper TCP/UDP IPv4/IPv6 tables"
                } else {
                    "Available protocol families collected; failed baselines retained"
                },
                &collected_at,
            ));
        } else {
            collectors.push(health(
                "connections",
                "active",
                "Last native snapshot retained on a four-second cadence",
                &collected_at,
            ));
        }

        let service_due = state
            .last_service_collection
            .map_or(true, |last| last.elapsed() >= SERVICE_INTERVAL);
        let mut service_output = state.services.values().cloned().collect::<Vec<_>>();
        if service_due {
            let (services, service_issues) = services::collect();
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
            collectors.push(health(
                "services",
                if service_issues.is_empty() {
                    "active"
                } else {
                    "partial"
                },
                if service_failed {
                    "Previous service baseline retained"
                } else if service_issues.is_empty() {
                    "Windows Service Control Manager telemetry"
                } else {
                    "Service states collected with configuration restrictions"
                },
                &collected_at,
            ));
        } else {
            collectors.push(health(
                "services",
                "active",
                "Last native snapshot retained on a fifteen-second cadence",
                &collected_at,
            ));
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
    mut current: Vec<ProcessRecord>,
    baseline: bool,
    now: &str,
    events: &mut Vec<TelemetryEvent>,
) -> HashMap<String, ProcessRecord> {
    for process in &mut current {
        if let Some(old) = previous.get(&process.key).filter(|old| old.active) {
            process.first_seen = old.first_seen.clone();
            process.observation_count = old.observation_count + 1;
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
            stopped.last_seen = now.into();
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
    mut current: Vec<ConnectionRecord>,
    failed_families: &HashSet<(String, String)>,
    baseline: bool,
    now: &str,
    events: &mut Vec<TelemetryEvent>,
) -> (HashMap<String, ConnectionRecord>, Vec<ConnectionRecord>) {
    for connection in &mut current {
        if let Some(old) = previous.get(&connection.key).filter(|old| old.active) {
            connection.first_seen = old.first_seen.clone();
            connection.observation_count = old.observation_count + 1;
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
            events.push(event(
                "connection_closed",
                "connection",
                &old.key,
                "Connection is no longer observed",
                now,
            ));
            let mut closed = old.clone();
            closed.active = false;
            closed.last_seen = now.into();
            current.push(closed);
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
    mut current: Vec<ServiceRecord>,
    baseline: bool,
    now: &str,
    events: &mut Vec<TelemetryEvent>,
) -> HashMap<String, ServiceRecord> {
    for service in &mut current {
        if let Some(old) = previous.get(&service.key).filter(|old| old.active) {
            service.first_seen = old.first_seen.clone();
            service.observation_count = old.observation_count + 1;
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
            missing.last_seen = now.into();
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
    TelemetryEvent {
        id: format!("{event_type}|{subject_key}|{occurred_at}"),
        event_type: event_type.into(),
        subject_type: subject_type.into(),
        subject_key: subject_key.into(),
        message: message.into(),
        occurred_at: occurred_at.into(),
    }
}

fn health(id: &str, status: &str, detail: &str, collected_at: &str) -> CollectorHealth {
    CollectorHealth {
        id: id.into(),
        status: status.into(),
        detail: detail.into(),
        collected_at: collected_at.into(),
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
            memory_bytes: 10,
            start_time: None,
            thread_count: Some(1),
            architecture: Some("x64".into()),
            description: None,
            publisher: None,
            signature_status: "unsigned".into(),
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
            first_seen: "first".into(),
            last_seen: "last".into(),
            observation_count: 1,
            active: true,
        };
        let previous = [(old.key.clone(), old)].into_iter().collect();
        let failed = [("tcp".into(), "ipv6".into())].into_iter().collect();
        let mut events = Vec::new();
        let (next, visible) =
            track_connections(previous, Vec::new(), &failed, true, "now", &mut events);
        assert_eq!(next.len(), 1);
        assert_eq!(visible.len(), 1);
        assert!(events.is_empty());
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
