use crate::{
    models::{DetectionConfidence, DetectionSeverity, DetectionStatus, DetectionStatusInput},
    persistence::Database,
    rules::RuleDefinition,
};
use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, OptionalExtension, Transaction};
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

const CONSUMER_ID: &str = "detection-engine-v1";
const DETECTION_SCHEMA_VERSION: u32 = 1;
const EVIDENCE_SCHEMA_VERSION: u32 = 1;
const HISTORY_SCHEMA_VERSION: u32 = 1;
const MAX_HISTORY_BATCH: u32 = 250;
const CORRELATION_WINDOW_SECONDS: i64 = 60;
const SERVICE_CORRELATION_WINDOW_SECONDS: i64 = 45;
const NETWORK_CORRELATION_WINDOW_SECONDS: i64 = 30;
const SUPPORTED_RULE_IDS: [&str; 6] = [
    "EDY-PROC-001",
    "EDY-PROC-002",
    "EDY-NET-001",
    "EDY-SVC-001",
    "EDY-SVC-002",
    "EDY-NET-002",
];

#[derive(Clone, Default)]
pub struct DetectionEngine;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DetectionProcessingSummary {
    pub processed_history: u32,
    pub detections_changed: u32,
    pub has_more: bool,
}

#[derive(Debug, Clone)]
struct FactualEvent {
    history_id: i64,
    event_id: String,
    transition: String,
    observed_at: String,
    source: String,
    event_type: String,
    entity_key: String,
    baseline_id: String,
    evidence: Value,
    event_schema_version: u32,
    observation_count: u64,
    condition_active: bool,
}

#[derive(Debug, Clone)]
struct RuleMatch {
    rule_id: &'static str,
    rule_version: u32,
    baseline_id: String,
    entity_type: String,
    entity_key: String,
    dedup_key: String,
    correlation_key: String,
    title: &'static str,
    summary: &'static str,
    severity: DetectionSeverity,
    confidence: DetectionConfidence,
    severity_reason: &'static str,
    confidence_reason: &'static str,
    what_happened: &'static str,
    why_flagged: &'static str,
    remediation_guidance: Vec<String>,
    observed_at: String,
    evidence: Vec<(String, String, FactualEvent)>,
}

#[derive(Debug)]
struct ExistingDetection {
    status: DetectionStatus,
    severity: DetectionSeverity,
    confidence: DetectionConfidence,
    condition_active: bool,
}

impl DetectionEngine {
    /// Registers immutable rule versions and their local enabled state.
    /// A changed definition with the same version fails closed.
    pub fn initialize(
        &self,
        database: &Database,
        definitions: &[RuleDefinition],
    ) -> Result<(), String> {
        validate_registry(definitions)?;
        database.analysis_transaction(|transaction| {
            let now = Utc::now().to_rfc3339();
            for definition in definitions {
                let definition_json = serde_json::to_string(definition)
                    .map_err(|_| "Unable to serialize detection rule definition")?;
                let definition_sha256 = stable_hash(&definition_json);
                let existing_hash: Option<String> = transaction
                    .query_row(
                        "SELECT definition_sha256 FROM detection_rule_versions
                         WHERE rule_id = ?1 AND rule_version = ?2",
                        params![definition.rule_id, definition.version],
                        |row| row.get(0),
                    )
                    .optional()
                    .map_err(|_| "Unable to inspect registered detection rule")?;
                if existing_hash
                    .as_ref()
                    .is_some_and(|value| value != &definition_sha256)
                {
                    return Err(format!(
                        "Detection rule {} version {} changed without a version increment",
                        definition.rule_id, definition.version
                    ));
                }
                if existing_hash.is_none() {
                    transaction
                        .execute(
                            "INSERT INTO detection_rule_versions(
                                rule_id, rule_version, name, description, category,
                                definition_json, definition_sha256, registered_at,
                                schema_version
                             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1)",
                            params![
                                definition.rule_id,
                                definition.version,
                                definition.name,
                                definition.description,
                                definition.category,
                                definition_json,
                                definition_sha256,
                                now
                            ],
                        )
                        .map_err(|_| "Unable to register detection rule version")?;
                }
                transaction
                    .execute(
                        "INSERT INTO detection_rule_state(
                            rule_id, rule_version, enabled, updated_at
                         ) VALUES (?1, ?2, ?3, ?4)
                         ON CONFLICT(rule_id) DO UPDATE SET
                            rule_version = excluded.rule_version,
                            updated_at = excluded.updated_at",
                        params![
                            definition.rule_id,
                            definition.version,
                            definition.enabled as i64,
                            now
                        ],
                    )
                    .map_err(|_| "Unable to initialize detection rule state")?;
                supersede_prior_rule_versions(
                    transaction,
                    &definition.rule_id,
                    definition.version,
                    &now,
                )?;
            }
            Ok(())
        })
    }

    /// Consumes only factual provenance created after the durable checkpoint.
    /// Migration v6 seeds the checkpoint from MAX(history_id), so pre-v6 facts
    /// are never retroactively classified.
    pub fn process_pending(
        &self,
        database: &Database,
    ) -> Result<DetectionProcessingSummary, String> {
        database.analysis_transaction(|transaction| {
            let checkpoint = load_checkpoint(transaction)?;
            let pending = load_pending_history(transaction, checkpoint, MAX_HISTORY_BATCH + 1)?;
            let has_more = pending.len() > MAX_HISTORY_BATCH as usize;
            let pending = pending
                .into_iter()
                .take(MAX_HISTORY_BATCH as usize)
                .collect::<Vec<_>>();
            let enabled = load_enabled_rule_versions(transaction)?;
            let mut matches = HashMap::<String, RuleMatch>::new();
            let mut changed = 0u32;
            for event in &pending {
                if event.transition == "inactive" {
                    changed = changed.saturating_add(deactivate_for_source_event(
                        transaction,
                        &event.event_id,
                        &event.observed_at,
                    )?);
                    continue;
                }
                if !matches!(event.transition.as_str(), "first_observed" | "reactivated") {
                    continue;
                }
                let related = load_related_events(transaction, event)?;
                for candidate in evaluate_rules(transaction, event, &related, &enabled) {
                    let identity = detection_identity(&candidate);
                    matches
                        .entry(identity)
                        .and_modify(|existing| merge_match_evidence(existing, &candidate))
                        .or_insert(candidate);
                }
            }

            // Network facts are aggregate conditions: the v5 provenance ledger records
            // lifecycle transitions, not every steady-state refresh. Re-evaluate only
            // the three active network aggregates created after the v6 cutover so
            // EDY-NET-002 can require two observations without scanning old history.
            if enabled.contains_key("EDY-NET-002") {
                let mature_network = load_mature_network_events(transaction)?;
                if let Some(trigger) = mature_network
                    .iter()
                    .find(|event| event.event_type == "gateway_changed")
                {
                    if let Some(candidate) =
                        evaluate_net_002(trigger, &mature_network, enabled["EDY-NET-002"])
                    {
                        matches.insert(detection_identity(&candidate), candidate);
                    }
                }
            }

            let candidates = apply_rule_precedence(transaction, matches.into_values().collect())?;
            for candidate in candidates {
                changed =
                    changed.saturating_add(deactivate_lower_precedence(transaction, &candidate)?);
                changed = changed.saturating_add(upsert_detection(transaction, candidate)?);
            }

            let last_history_id = pending.last().map_or(checkpoint, |event| event.history_id);
            transaction
                .execute(
                    "UPDATE analysis_checkpoints
                     SET last_security_event_history_id = ?1, updated_at = ?2
                     WHERE consumer_id = ?3",
                    params![last_history_id, Utc::now().to_rfc3339(), CONSUMER_ID],
                )
                .map_err(|_| "Unable to advance detection checkpoint")?;
            Ok(DetectionProcessingSummary {
                processed_history: pending.len() as u32,
                detections_changed: changed,
                has_more,
            })
        })
    }

    pub fn set_detection_status(
        &self,
        database: &Database,
        input: DetectionStatusInput,
    ) -> Result<(), String> {
        validate_identifier(&input.detection_id, "detection ID")?;
        database.analysis_transaction(|transaction| {
            let existing = load_existing_detection_by_id(transaction, &input.detection_id)?
                .ok_or_else(|| "Detection was not found".to_string())?;
            if existing.status == input.status {
                return Ok(());
            }
            let now = Utc::now().to_rfc3339();
            transaction
                .execute(
                    "UPDATE detections SET status = ?1, updated_at = ?2
                     WHERE detection_id = ?3",
                    params![input.status.as_str(), now, input.detection_id],
                )
                .map_err(|_| "Unable to update detection status")?;
            append_detection_history(
                transaction,
                &input.detection_id,
                "status_changed",
                &now,
                "Local investigation workflow status changed",
                Some(existing.status),
                Some(input.status),
            )
        })
    }

    pub fn set_rule_enabled(
        &self,
        database: &Database,
        rule_id: &str,
        enabled: bool,
    ) -> Result<(), String> {
        validate_identifier(rule_id, "rule ID")?;
        database.analysis_transaction(|transaction| {
            let now = Utc::now().to_rfc3339();
            let changed = transaction
                .execute(
                    "UPDATE detection_rule_state SET enabled = ?1, updated_at = ?2
                     WHERE rule_id = ?3",
                    params![enabled as i64, now, rule_id],
                )
                .map_err(|_| "Unable to update detection rule state")?;
            if changed == 0 {
                return Err("Detection rule was not found".into());
            }
            if !enabled {
                deactivate_rule_detections(transaction, rule_id, &now, "rule_disabled")?;
            }
            Ok(())
        })
    }

    pub fn rule_definitions(&self, database: &Database) -> Result<Vec<RuleDefinition>, String> {
        database.analysis_read(|connection| {
            let mut statement = connection
                .prepare(
                    "SELECT version.definition_json, state.enabled
                     FROM detection_rule_state AS state
                     JOIN detection_rule_versions AS version
                       ON version.rule_id = state.rule_id
                      AND version.rule_version = state.rule_version
                     ORDER BY version.category, version.rule_id",
                )
                .map_err(|_| "Unable to prepare detection rule query")?;
            let rows = statement
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? != 0))
                })
                .map_err(|_| "Unable to query detection rules")?;
            rows.map(|row| {
                let (json, enabled) = row.map_err(|_| "Unable to read detection rule")?;
                let mut definition: RuleDefinition =
                    serde_json::from_str(&json).map_err(|_| "Stored detection rule is invalid")?;
                definition.enabled = enabled;
                Ok(definition)
            })
            .collect()
        })
    }
}

fn validate_registry(definitions: &[RuleDefinition]) -> Result<(), String> {
    let mut identities = HashSet::new();
    for definition in definitions {
        definition.validate()?;
        if !SUPPORTED_RULE_IDS.contains(&definition.rule_id.as_str()) {
            return Err(format!("Unsupported detection rule {}", definition.rule_id));
        }
        if !identities.insert((definition.rule_id.as_str(), definition.version)) {
            return Err("Detection rule registry contains a duplicate version".into());
        }
    }
    Ok(())
}

fn load_checkpoint(transaction: &Transaction<'_>) -> Result<i64, String> {
    transaction
        .query_row(
            "SELECT last_security_event_history_id FROM analysis_checkpoints
             WHERE consumer_id = ?1",
            [CONSUMER_ID],
            |row| row.get(0),
        )
        .map_err(|_| "Detection checkpoint is unavailable".into())
}

fn load_mature_network_events(transaction: &Transaction<'_>) -> Result<Vec<FactualEvent>, String> {
    let cutover: i64 = transaction
        .query_row(
            "SELECT cutover_security_event_history_id FROM analysis_checkpoints
             WHERE consumer_id = ?1",
            [CONSUMER_ID],
            |row| row.get(0),
        )
        .map_err(|_| "Detection cutover checkpoint is unavailable")?;
    let recent_cutoff =
        (Utc::now() - Duration::seconds(CORRELATION_WINDOW_SECONDS * 2)).to_rfc3339();
    let mut statement = transaction
        .prepare(
            "SELECT origin.history_id, event.id, 'aggregate_observed',
                    event.last_seen_at, event.source, event.event_type,
                    event.entity_type, event.entity_key, event.baseline_id,
                    event.payload_json, event.schema_version,
                    event.observation_count, event.condition_active
             FROM security_events AS event
             JOIN security_event_history AS origin
               ON origin.history_id = (
                   SELECT MIN(history.history_id)
                   FROM security_event_history AS history
                   WHERE history.event_id = event.id
                     AND history.history_id > ?1
                     AND history.transition IN ('first_observed', 'reactivated')
               )
             WHERE event.event_type IN (
                       'gateway_changed', 'dns_changed', 'primary_route_changed'
                   )
               AND event.condition_active = 1
               AND event.schema_version >= 2
               AND EXISTS (
                   SELECT 1 FROM behavioral_baselines AS baseline
                   WHERE baseline.baseline_id = event.baseline_id
                     AND baseline.active = 1 AND baseline.status = 'ready'
               )
               AND event.last_seen_at >= ?2
             ORDER BY event.last_seen_at DESC, event.id DESC",
        )
        .map_err(|_| "Unable to prepare mature network fact query")?;
    let rows = statement
        .query_map(params![cutover, recent_cutoff], map_factual_event)
        .map_err(|_| "Unable to query mature network facts")?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Unable to read mature network facts".into())
}

fn load_pending_history(
    transaction: &Transaction<'_>,
    checkpoint: i64,
    limit: u32,
) -> Result<Vec<FactualEvent>, String> {
    let mut statement = transaction
        .prepare(
            "SELECT history.history_id, history.event_id, history.transition,
                    history.observed_at, history.source, history.event_type,
                    history.entity_type, history.entity_key, history.baseline_id,
                    history.evidence_json, history.event_schema_version,
                    history.observation_count, event.condition_active
             FROM security_event_history AS history
             JOIN security_events AS event ON event.id = history.event_id
             JOIN behavioral_baselines AS baseline
               ON baseline.baseline_id = history.baseline_id
             WHERE history.history_id > ?1
               AND baseline.active = 1 AND baseline.status = 'ready'
             ORDER BY history.history_id ASC
             LIMIT ?2",
        )
        .map_err(|_| "Unable to prepare pending factual provenance query")?;
    let rows = statement
        .query_map(params![checkpoint, limit], map_factual_event)
        .map_err(|_| "Unable to query pending factual provenance")?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Unable to read pending factual provenance".into())
}

fn map_factual_event(row: &rusqlite::Row<'_>) -> rusqlite::Result<FactualEvent> {
    let evidence_json: String = row.get(9)?;
    Ok(FactualEvent {
        history_id: row.get(0)?,
        event_id: row.get(1)?,
        transition: row.get(2)?,
        observed_at: row.get(3)?,
        source: row.get(4)?,
        event_type: row.get(5)?,
        entity_key: row.get(7)?,
        baseline_id: row.get(8)?,
        evidence: serde_json::from_str(&evidence_json).unwrap_or(Value::Null),
        event_schema_version: row.get(10)?,
        observation_count: row.get(11)?,
        condition_active: row.get::<_, i64>(12)? != 0,
    })
}

fn load_enabled_rule_versions(
    transaction: &Transaction<'_>,
) -> Result<HashMap<String, u32>, String> {
    let mut statement = transaction
        .prepare("SELECT rule_id, rule_version FROM detection_rule_state WHERE enabled = 1")
        .map_err(|_| "Unable to prepare enabled rule query")?;
    let rows = statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|_| "Unable to query enabled rules")?;
    rows.collect::<Result<HashMap<_, _>, _>>()
        .map_err(|_| "Unable to read enabled rules".into())
}

fn load_related_events(
    transaction: &Transaction<'_>,
    event: &FactualEvent,
) -> Result<Vec<FactualEvent>, String> {
    let observed = parse_timestamp(&event.observed_at)?;
    let from = (observed - Duration::seconds(CORRELATION_WINDOW_SECONDS)).to_rfc3339();
    let mut statement = transaction
        .prepare(
            "SELECT history.history_id, history.event_id, history.transition,
                    history.observed_at, history.source, history.event_type,
                    history.entity_type, history.entity_key, history.baseline_id,
                    history.evidence_json, history.event_schema_version,
                    history.observation_count, aggregate.condition_active
             FROM security_event_history AS history
             JOIN security_events AS aggregate ON aggregate.id = history.event_id
             WHERE history.baseline_id = ?1
               AND history.transition IN ('first_observed', 'reactivated')
               AND history.observed_at >= ?2 AND history.observed_at <= ?3
               AND aggregate.condition_active = 1
             ORDER BY history.history_id DESC
             LIMIT 500",
        )
        .map_err(|_| "Unable to prepare correlated factual event query")?;
    let rows = statement
        .query_map(
            params![event.baseline_id, from, event.observed_at],
            map_factual_event,
        )
        .map_err(|_| "Unable to query correlated factual events")?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Unable to read correlated factual events".into())
}

fn evaluate_rules(
    transaction: &Transaction<'_>,
    trigger: &FactualEvent,
    related: &[FactualEvent],
    enabled: &HashMap<String, u32>,
) -> Vec<RuleMatch> {
    let mut results = Vec::new();
    if let Some(version) = enabled.get("EDY-PROC-001") {
        if let Some(value) = evaluate_proc_001(transaction, trigger, related, *version) {
            results.push(value);
        }
    }
    if let Some(version) = enabled.get("EDY-PROC-002") {
        if let Some(value) = evaluate_proc_002(transaction, trigger, related, *version) {
            results.push(value);
        }
    }
    if let Some(version) = enabled.get("EDY-NET-001") {
        if let Some(value) = evaluate_net_001(transaction, trigger, related, *version) {
            results.push(value);
        }
    }
    if let Some(version) = enabled.get("EDY-SVC-001") {
        if let Some(value) = evaluate_svc_001(trigger, *version) {
            results.push(value);
        }
    }
    if let Some(version) = enabled.get("EDY-SVC-002") {
        if let Some(value) = evaluate_svc_002(trigger, related, *version) {
            results.push(value);
        }
    }
    if let Some(version) = enabled.get("EDY-NET-002") {
        if let Some(value) = evaluate_net_002(trigger, related, *version) {
            results.push(value);
        }
    }
    results
}

fn evaluate_proc_001(
    transaction: &Transaction<'_>,
    trigger: &FactualEvent,
    related: &[FactualEvent],
    version: u32,
) -> Option<RuleMatch> {
    let (executable, process) = correlated_pair(
        trigger,
        related,
        "executable_first_seen",
        "process_first_seen",
        |executable, process| {
            let path = evidence_text(&executable.evidence, "path");
            paths_match(path, evidence_text(&process.evidence, "path"))
                && path.is_some_and(is_user_temp_path)
                && evidence_text(&executable.evidence, "signatureStatus") == Some("unsigned")
        },
    )?;
    let path = evidence_text(&executable.evidence, "path")?;
    if baseline_has_path(transaction, &trigger.baseline_id, path) {
        return None;
    }
    let correlation =
        evidence_text(&executable.evidence, "executableKey").unwrap_or(&executable.entity_key);
    Some(base_match(
        "EDY-PROC-001",
        version,
        trigger,
        "process",
        correlation,
        correlation,
        "New unsigned executable ran from a temporary path",
        "An executable absent from the baseline ran from the user's local temporary directory and has no trusted signature.",
        DetectionSeverity::Low,
        DetectionConfidence::High,
        "Multiple local facts are present, but legitimate installers and development tools remain plausible.",
        "Path, signature state, execution and baseline novelty are all present in one factual event.",
        "A new unsigned executable was executed from the user's local temporary directory.",
        "The executable was absent from the baseline, was actually running, was unsigned, and used a narrowly defined temporary path.",
        vec![
            "Verify the executable origin and the software that created it".into(),
            "Review its digital signature and parent process".into(),
        ],
        vec![
            ("executable_identity".into(), "Unsigned temporary executable".into(), executable.clone()),
            ("executed_process".into(), "Executed process pattern".into(), process.clone()),
        ],
    ))
}

fn evaluate_proc_002(
    transaction: &Transaction<'_>,
    trigger: &FactualEvent,
    related: &[FactualEvent],
    version: u32,
) -> Option<RuleMatch> {
    let (relationship, executable) = correlated_pair(
        trigger,
        related,
        "parent_child_first_seen",
        "executable_first_seen",
        |relationship, executable| {
            let child_path = evidence_text(&relationship.evidence, "childPath");
            let executable_path = evidence_text(&executable.evidence, "path");
            paths_match(child_path, executable_path)
                && executable_path.is_some_and(is_user_temp_path)
                && evidence_text(&executable.evidence, "signatureStatus") == Some("unsigned")
        },
    )?;
    let parent_path = evidence_text(&relationship.evidence, "parentPath")?;
    let child_path = evidence_text(&executable.evidence, "path")?;
    if baseline_has_path(transaction, &trigger.baseline_id, child_path) {
        return None;
    }
    let parent_signed: bool = transaction
        .query_row(
            "SELECT EXISTS(
                SELECT 1 FROM baseline_executables
                WHERE baseline_id = ?1 AND normalized_path = ?2
                  AND signature_status = 'signed'
            )",
            params![trigger.baseline_id, normalize_windows_path(parent_path)],
            |row| row.get(0),
        )
        .ok()?;
    if !parent_signed {
        return None;
    }
    let has_qualifying_outbound = related.iter().any(|event| {
        event.event_type == "destination_first_seen"
            && evidence_text(&event.evidence, "association") == Some("associated")
            && evidence_text(&event.evidence, "process")
                == evidence_text(&executable.evidence, "process")
    });
    if has_qualifying_outbound {
        return None;
    }
    let process = related.iter().find(|event| {
        event.event_type == "process_first_seen"
            && paths_match(
                evidence_text(&event.evidence, "path"),
                evidence_text(&executable.evidence, "path"),
            )
    })?;
    let correlation = evidence_text(&executable.evidence, "executableKey")
        .or_else(|| evidence_text(&relationship.evidence, "childExecutableKey"))
        .unwrap_or(&executable.entity_key);
    Some(base_match(
        "EDY-PROC-002",
        version,
        trigger,
        "process_relationship",
        correlation,
        correlation,
        "New parent-child relationship launched an unsigned temporary executable",
        "A new process relationship and a new unsigned temporary executable were correlated in the same observation window.",
        DetectionSeverity::Medium,
        DetectionConfidence::High,
        "The relationship, execution context, temporary path and unsigned state corroborate each other; no maliciousness is asserted.",
        "Two independent factual event types identify the same child executable within sixty seconds.",
        "A previously unseen parent-child relationship launched a previously unseen unsigned executable from the user temporary directory.",
        "The rule correlated the child path or executable identity across two factual events rather than relying on a process name alone.",
        vec![
            "Inspect the parent process and verify whether it belongs to an expected installer or updater".into(),
            "Review the executable origin and signature before taking any action".into(),
        ],
        vec![
            ("process_relationship".into(), "New parent-child relationship".into(), relationship.clone()),
            ("executable_identity".into(), "Unsigned temporary executable".into(), executable.clone()),
            ("executed_process".into(), "Executed process pattern".into(), process.clone()),
        ],
    ))
}

fn evaluate_net_001(
    transaction: &Transaction<'_>,
    trigger: &FactualEvent,
    related: &[FactualEvent],
    version: u32,
) -> Option<RuleMatch> {
    let (destination, executable) = correlated_pair(
        trigger,
        related,
        "destination_first_seen",
        "executable_first_seen",
        |destination, executable| {
            if evidence_text(&destination.evidence, "association") != Some("associated") {
                return false;
            }
            let destination_key = evidence_text(&destination.evidence, "executableKey");
            let executable_key = evidence_text(&executable.evidence, "executableKey");
            let same_executable = destination_key.is_some() && destination_key == executable_key
                || (evidence_text(&destination.evidence, "process")
                    == evidence_text(&executable.evidence, "process")
                    && related
                        .iter()
                        .filter(|event| {
                            event.event_type == "executable_first_seen"
                                && evidence_text(&event.evidence, "process")
                                    == evidence_text(&destination.evidence, "process")
                        })
                        .count()
                        == 1);
            same_executable
                && evidence_text(&executable.evidence, "path").is_some_and(is_user_temp_path)
                && evidence_text(&executable.evidence, "signatureStatus") == Some("unsigned")
                && evidence_text(&destination.evidence, "remoteIp")
                    .is_some_and(|ip| !is_loopback(ip))
        },
    )?;
    let executable_path = evidence_text(&executable.evidence, "path")?;
    if baseline_has_path(transaction, &trigger.baseline_id, executable_path) {
        return None;
    }
    let process = related.iter().find(|event| {
        event.event_type == "process_first_seen"
            && paths_match(
                evidence_text(&event.evidence, "path"),
                evidence_text(&executable.evidence, "path"),
            )
    })?;
    let correlation = evidence_text(&executable.evidence, "executableKey")
        .or_else(|| evidence_text(&destination.evidence, "executableKey"))
        .unwrap_or(&executable.entity_key);
    Some(base_match(
        "EDY-NET-001",
        version,
        trigger,
        "executable",
        correlation,
        correlation,
        "New unsigned temporary executable opened a new outbound destination",
        "A new destination was associated with the same new unsigned executable running from the user temporary directory.",
        DetectionSeverity::Medium,
        DetectionConfidence::Medium,
        "Execution, signature, path and associated network activity provide corroboration, without external reputation claims.",
        "The process association and executable identity were correlated inside a sixty-second window.",
        "A newly observed unsigned temporary executable initiated newly observed non-loopback network activity.",
        "The destination alone was not sufficient; the rule required associated process and executable context.",
        vec![
            "Verify the executable origin and whether the destination is expected for the software".into(),
            "Review the parent process and recent installation activity".into(),
        ],
        vec![
            ("network_destination".into(), "New associated destination".into(), destination.clone()),
            ("executable_identity".into(), "Unsigned temporary executable".into(), executable.clone()),
            ("executed_process".into(), "Executed process pattern".into(), process.clone()),
        ],
    ))
}

fn evaluate_svc_001(trigger: &FactualEvent, version: u32) -> Option<RuleMatch> {
    if trigger.event_type != "service_first_seen" || !trigger.condition_active {
        return None;
    }
    let path = service_binary_executable_path(evidence_text(&trigger.evidence, "binaryPath")?)?;
    let startup = evidence_text(&trigger.evidence, "startupType")?;
    if !is_user_writable_path(path)
        || !matches!(startup, "Automatic" | "Automatic Delayed")
        || evidence_text(&trigger.evidence, "status") != Some("Running")
        || !evidence_text(&trigger.evidence, "account").is_some_and(is_privileged_service_account)
    {
        return None;
    }
    let service_key = evidence_text(&trigger.evidence, "serviceKey").unwrap_or(&trigger.entity_key);
    Some(base_match(
        "EDY-SVC-001",
        version,
        trigger,
        "service",
        service_key,
        service_key,
        "New automatic service uses a binary from a user-writable path",
        "A newly observed automatic Windows service points to a user-writable application, downloads or desktop directory.",
        DetectionSeverity::Medium,
        DetectionConfidence::High,
        "Automatic startup, a user-writable binary path and a privileged service account are corroborated.",
        "The service identity, startup type and binary path are present in one successful SCM observation.",
        "A new automatic service was configured to run a binary from a user-writable directory.",
        "A new service alone was insufficient; automatic startup, privileged account and an unambiguous user-writable executable path were required.",
        vec![
            "Review the service installation source and binary path".into(),
            "Verify whether a recent trusted installer created the service".into(),
        ],
        vec![("service_configuration".into(), "New automatic user-writable-path service".into(), trigger.clone())],
    ))
}

fn evaluate_svc_002(
    trigger: &FactualEvent,
    related: &[FactualEvent],
    version: u32,
) -> Option<RuleMatch> {
    let change_types = [
        "service_binary_changed",
        "service_startup_changed",
        "service_account_changed",
    ];
    if !change_types.contains(&trigger.event_type.as_str()) {
        return None;
    }
    let service_key = evidence_text(&trigger.evidence, "serviceKey")
        .or_else(|| evidence_text(&trigger.evidence, "serviceName"))?;
    let trigger_time = parse_timestamp(&trigger.observed_at).ok()?;
    let cutoff = trigger_time - Duration::seconds(SERVICE_CORRELATION_WINDOW_SECONDS);
    let mut changes = related
        .iter()
        .filter(|event| {
            change_types.contains(&event.event_type.as_str())
                && evidence_text(&event.evidence, "serviceKey")
                    .or_else(|| evidence_text(&event.evidence, "serviceName"))
                    == Some(service_key)
                && parse_timestamp(&event.observed_at).is_ok_and(|time| time >= cutoff)
        })
        .cloned()
        .collect::<Vec<_>>();
    changes.sort_by(|left, right| left.event_type.cmp(&right.event_type));
    changes.dedup_by(|left, right| left.event_type == right.event_type);
    let includes_binary = changes
        .iter()
        .any(|event| event.event_type == "service_binary_changed");
    let missing_after = changes
        .iter()
        .any(|event| match event.evidence.get("after") {
            None => true,
            Some(value) => value.is_null(),
        });
    if changes.len() < 2 || !includes_binary || missing_after {
        return None;
    }
    Some(base_match(
        "EDY-SVC-002",
        version,
        trigger,
        "service",
        service_key,
        service_key,
        "Multiple persistent service properties changed together",
        "At least two baseline service properties changed for the same service within forty-five seconds.",
        DetectionSeverity::Medium,
        DetectionConfidence::High,
        "A binary-path change combined with another persistent property justifies investigation.",
        "Distinct factual service-change events identify the same service inside one correlation window.",
        "Multiple persistent properties of a baseline service changed together.",
        "The rule required at least two independent service configuration changes and never inferred intent from a single change.",
        vec![
            "Review the software or administrator action that modified the service".into(),
            "Compare the new binary, startup and account settings with expected configuration".into(),
        ],
        changes
            .into_iter()
            .map(|event| ("service_change".into(), event.event_type.clone(), event))
            .collect(),
    ))
}

fn evaluate_net_002(
    trigger: &FactualEvent,
    related: &[FactualEvent],
    version: u32,
) -> Option<RuleMatch> {
    if !matches!(
        trigger.event_type.as_str(),
        "gateway_changed" | "dns_changed"
    ) {
        return None;
    }
    let gateway = related
        .iter()
        .find(|event| event.event_type == "gateway_changed")?;
    let dns = related
        .iter()
        .find(|event| event.event_type == "dns_changed")?;
    if gateway.observation_count < 2 || dns.observation_count < 2 {
        return None;
    }
    if gateway.event_schema_version < 2
        || dns.event_schema_version < 2
        || !has_before_after(&gateway.evidence)
        || !has_before_after(&dns.evidence)
    {
        return None;
    }
    let gateway_seen = parse_timestamp(&gateway.observed_at).ok()?;
    let dns_seen = parse_timestamp(&dns.observed_at).ok()?;
    if (gateway_seen - dns_seen).num_seconds().abs() > NETWORK_CORRELATION_WINDOW_SECONDS {
        return None;
    }
    let route_changed = related
        .iter()
        .any(|event| event.event_type == "primary_route_changed");
    if route_changed {
        return None;
    }
    let interface_key = evidence_text(&gateway.evidence, "primaryInterfaceKey")
        .or_else(|| evidence_text(&dns.evidence, "primaryInterfaceKey"))
        .unwrap_or("primary-network");
    Some(base_match(
        "EDY-NET-002",
        version,
        trigger,
        "network_configuration",
        interface_key,
        interface_key,
        "Gateway and DNS changed without a primary-route transition",
        "The default gateway and DNS resolver set changed together while the primary route identity remained stable.",
        DetectionSeverity::Low,
        DetectionConfidence::High,
        "The combined change warrants review, but VPN, DHCP and managed-network changes remain common legitimate causes.",
        "Two independent network configuration facts occurred in the same collection window.",
        "Gateway and DNS configuration changed together without evidence of a primary interface transition.",
        "A single DNS, gateway or route change cannot trigger this rule.",
        vec![
            "Confirm whether Wi-Fi, VPN, DHCP or administrator configuration changed".into(),
            "Compare the gateway and DNS values with the expected network".into(),
        ],
        vec![
            ("gateway_configuration".into(), "Gateway changed".into(), gateway.clone()),
            ("dns_configuration".into(), "DNS resolvers changed".into(), dns.clone()),
        ],
    ))
}

#[allow(clippy::too_many_arguments)]
fn base_match(
    rule_id: &'static str,
    rule_version: u32,
    trigger: &FactualEvent,
    entity_type: &str,
    entity_key: &str,
    correlation_key: &str,
    title: &'static str,
    summary: &'static str,
    severity: DetectionSeverity,
    confidence: DetectionConfidence,
    severity_reason: &'static str,
    confidence_reason: &'static str,
    what_happened: &'static str,
    why_flagged: &'static str,
    remediation_guidance: Vec<String>,
    evidence: Vec<(String, String, FactualEvent)>,
) -> RuleMatch {
    RuleMatch {
        rule_id,
        rule_version,
        baseline_id: trigger.baseline_id.clone(),
        entity_type: entity_type.into(),
        entity_key: entity_key.into(),
        dedup_key: stable_hash(&format!("{rule_id}|{entity_type}|{entity_key}")),
        correlation_key: correlation_key.into(),
        title,
        summary,
        severity,
        confidence,
        severity_reason,
        confidence_reason,
        what_happened,
        why_flagged,
        remediation_guidance,
        observed_at: trigger.observed_at.clone(),
        evidence,
    }
}

fn correlated_pair<'a>(
    trigger: &'a FactualEvent,
    related: &'a [FactualEvent],
    first_type: &str,
    second_type: &str,
    predicate: impl Fn(&FactualEvent, &FactualEvent) -> bool,
) -> Option<(&'a FactualEvent, &'a FactualEvent)> {
    let mut candidates = related.iter().collect::<Vec<_>>();
    if !candidates
        .iter()
        .any(|event| event.history_id == trigger.history_id)
    {
        candidates.push(trigger);
    }
    for first in candidates
        .iter()
        .copied()
        .filter(|event| event.event_type == first_type)
    {
        for second in candidates
            .iter()
            .copied()
            .filter(|event| event.event_type == second_type)
        {
            if predicate(first, second) {
                return Some((first, second));
            }
        }
    }
    None
}

fn upsert_detection(transaction: &Transaction<'_>, candidate: RuleMatch) -> Result<u32, String> {
    debug_assert!(candidate.severity <= DetectionSeverity::Medium);
    let detection_id = detection_identity(&candidate);
    let existing = load_existing_detection(transaction, &candidate)?;
    let explanation = json!({
        "whatHappened": candidate.what_happened,
        "whyFlagged": candidate.why_flagged,
        "severityReason": candidate.severity_reason,
        "confidenceReason": candidate.confidence_reason,
    });
    let explanation_json = serde_json::to_string(&explanation)
        .map_err(|_| "Unable to serialize detection explanation")?;
    let remediation_json = serde_json::to_string(&candidate.remediation_guidance)
        .map_err(|_| "Unable to serialize remediation guidance")?;
    let status = match existing.as_ref() {
        Some(value) if !value.condition_active && value.status == DetectionStatus::Resolved => {
            DetectionStatus::New
        }
        Some(value) => value.status,
        None => DetectionStatus::New,
    };
    let now = Utc::now().to_rfc3339();
    transaction
        .execute(
            "INSERT INTO detections(
                detection_id, dedup_key, correlation_key, rule_id, rule_version,
                baseline_id, entity_type, entity_key, title, summary, severity,
                confidence, severity_reason, confidence_reason,
                remediation_guidance_json, status, first_detected_at,
                last_detected_at, occurrence_count, condition_active,
                explanation_json, created_at, updated_at, schema_version
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                       ?13, ?14, ?15, ?16, ?17, ?17, 1, 1, ?18, ?19, ?19, ?20)
             ON CONFLICT(baseline_id, rule_id, rule_version, dedup_key) DO UPDATE SET
                last_detected_at = excluded.last_detected_at,
                occurrence_count = CASE
                    WHEN detections.condition_active = 0
                    THEN detections.occurrence_count + 1
                    ELSE detections.occurrence_count END,
                condition_active = 1,
                status = CASE
                    WHEN detections.condition_active = 0 AND detections.status = 'resolved'
                    THEN 'new' ELSE detections.status END,
                severity = excluded.severity,
                confidence = excluded.confidence,
                severity_reason = excluded.severity_reason,
                confidence_reason = excluded.confidence_reason,
                remediation_guidance_json = excluded.remediation_guidance_json,
                explanation_json = excluded.explanation_json,
                updated_at = excluded.updated_at",
            params![
                detection_id,
                candidate.dedup_key,
                candidate.correlation_key,
                candidate.rule_id,
                candidate.rule_version,
                candidate.baseline_id,
                candidate.entity_type,
                candidate.entity_key,
                candidate.title,
                candidate.summary,
                candidate.severity.as_str(),
                candidate.confidence.as_str(),
                candidate.severity_reason,
                candidate.confidence_reason,
                remediation_json,
                status.as_str(),
                candidate.observed_at,
                explanation_json,
                now,
                DETECTION_SCHEMA_VERSION
            ],
        )
        .map_err(|_| "Unable to persist detection")?;

    for (role, label, event) in &candidate.evidence {
        append_evidence(transaction, &detection_id, role, label, event)?;
    }

    let changed = match existing {
        None => append_detection_history(
            transaction,
            &detection_id,
            "first_detected",
            &candidate.observed_at,
            "Rule conditions were first satisfied",
            None,
            Some(DetectionStatus::New),
        )
        .map(|_| 1)?,
        Some(previous) if !previous.condition_active => append_detection_history(
            transaction,
            &detection_id,
            "reactivated",
            &candidate.observed_at,
            "Rule conditions became active again",
            Some(previous.status),
            Some(status),
        )
        .map(|_| 1)?,
        Some(previous)
            if previous.severity != candidate.severity
                || previous.confidence != candidate.confidence =>
        {
            append_detection_history(
                transaction,
                &detection_id,
                "classification_changed",
                &candidate.observed_at,
                "Corroborating evidence changed the current classification",
                Some(previous.status),
                Some(status),
            )
            .map(|_| 1)?
        }
        Some(_) => 0,
    };
    Ok(changed)
}

fn precedence(rule_id: &str) -> u8 {
    match rule_id {
        "EDY-NET-001" => 3,
        "EDY-PROC-002" => 2,
        "EDY-PROC-001" => 1,
        _ => 0,
    }
}

fn apply_rule_precedence(
    transaction: &Transaction<'_>,
    candidates: Vec<RuleMatch>,
) -> Result<Vec<RuleMatch>, String> {
    let strongest = candidates
        .iter()
        .fold(HashMap::new(), |mut values, candidate| {
            let rank = precedence(candidate.rule_id);
            if rank > 0 {
                values
                    .entry((
                        candidate.baseline_id.clone(),
                        candidate.correlation_key.clone(),
                    ))
                    .and_modify(|current: &mut u8| *current = (*current).max(rank))
                    .or_insert(rank);
            }
            values
        });
    candidates
        .into_iter()
        .filter_map(|candidate| {
            let rank = precedence(candidate.rule_id);
            if rank > 0
                && strongest
                    .get(&(
                        candidate.baseline_id.clone(),
                        candidate.correlation_key.clone(),
                    ))
                    .is_some_and(|strongest| *strongest > rank)
            {
                return None;
            }
            match has_active_higher_precedence(transaction, &candidate) {
                Ok(true) => None,
                Ok(false) => Some(Ok(candidate)),
                Err(error) => Some(Err(error)),
            }
        })
        .collect()
}

fn has_active_higher_precedence(
    transaction: &Transaction<'_>,
    candidate: &RuleMatch,
) -> Result<bool, String> {
    let rank = precedence(candidate.rule_id);
    if rank == 0 {
        return Ok(false);
    }
    let mut statement = transaction
        .prepare(
            "SELECT rule_id FROM detections
             WHERE baseline_id = ?1 AND correlation_key = ?2
               AND condition_active = 1
               AND rule_id IN ('EDY-PROC-001', 'EDY-PROC-002', 'EDY-NET-001')",
        )
        .map_err(|_| "Unable to prepare precedence query")?;
    let rules = statement
        .query_map(
            params![candidate.baseline_id, candidate.correlation_key],
            |row| row.get::<_, String>(0),
        )
        .map_err(|_| "Unable to query active rule precedence")?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Unable to read active rule precedence")?;
    Ok(rules.iter().any(|rule| precedence(rule) > rank))
}

fn deactivate_lower_precedence(
    transaction: &Transaction<'_>,
    candidate: &RuleMatch,
) -> Result<u32, String> {
    let rank = precedence(candidate.rule_id);
    if rank <= 1 {
        return Ok(0);
    }
    let mut statement = transaction
        .prepare(
            "SELECT detection_id, rule_id FROM detections
             WHERE baseline_id = ?1 AND correlation_key = ?2
               AND condition_active = 1
               AND rule_id IN ('EDY-PROC-001', 'EDY-PROC-002')",
        )
        .map_err(|_| "Unable to prepare lower-precedence detection query")?;
    let rows = statement
        .query_map(
            params![candidate.baseline_id, candidate.correlation_key],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .map_err(|_| "Unable to query lower-precedence detections")?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Unable to read lower-precedence detections")?;
    drop(statement);
    let observed_at = &candidate.observed_at;
    let mut changed = 0u32;
    for (detection_id, rule_id) in rows {
        if precedence(&rule_id) >= rank {
            continue;
        }
        transaction
            .execute(
                "UPDATE detections SET condition_active = 0, updated_at = ?1
                 WHERE detection_id = ?2",
                params![observed_at, detection_id],
            )
            .map_err(|_| "Unable to supersede lower-precedence detection")?;
        append_detection_history(
            transaction,
            &detection_id,
            "rule_superseded",
            observed_at,
            "A stronger correlated rule conclusion superseded this detection",
            None,
            None,
        )?;
        changed = changed.saturating_add(1);
    }
    Ok(changed)
}

fn append_evidence(
    transaction: &Transaction<'_>,
    detection_id: &str,
    role: &str,
    label: &str,
    event: &FactualEvent,
) -> Result<(), String> {
    let evidence_id = format!(
        "evidence-{}",
        &stable_hash(&format!(
            "detection-evidence-v1|{detection_id}|{}|{}|{role}",
            event.event_id, event.history_id
        ))[..32]
    );
    let evidence_json = serde_json::to_string(&json!({
        "label": label,
        "value": event.evidence,
    }))
    .map_err(|_| "Unable to serialize detection evidence")?;
    transaction
        .execute(
            "INSERT OR IGNORE INTO detection_evidence(
                evidence_id, detection_id, source_event_id, source_history_id,
                evidence_role, collector_source, captured_at,
                source_event_schema_version, source_observation_count,
                evidence_json, evidence_schema_version
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                evidence_id,
                detection_id,
                event.event_id,
                event.history_id,
                role,
                event.source,
                event.observed_at,
                event.event_schema_version,
                event.observation_count,
                evidence_json,
                EVIDENCE_SCHEMA_VERSION
            ],
        )
        .map(|_| ())
        .map_err(|_| "Unable to append detection evidence".into())
}

fn append_detection_history(
    transaction: &Transaction<'_>,
    detection_id: &str,
    transition: &str,
    observed_at: &str,
    reason: &str,
    previous_status: Option<DetectionStatus>,
    new_status: Option<DetectionStatus>,
) -> Result<(), String> {
    transaction
        .execute(
            "INSERT INTO detection_history(
                detection_id, transition, observed_at, recorded_at, reason,
                previous_status, new_status, severity, confidence,
                condition_active, occurrence_count, explanation_json,
                history_schema_version
             )
             SELECT detection_id, ?2, ?3, ?4, ?5, ?6, ?7, severity, confidence,
                    condition_active, occurrence_count, explanation_json, ?8
             FROM detections WHERE detection_id = ?1",
            params![
                detection_id,
                transition,
                observed_at,
                Utc::now().to_rfc3339(),
                reason,
                previous_status.map(DetectionStatus::as_str),
                new_status.map(DetectionStatus::as_str),
                HISTORY_SCHEMA_VERSION
            ],
        )
        .map(|_| ())
        .map_err(|_| "Unable to append detection history".into())
}

fn deactivate_for_source_event(
    transaction: &Transaction<'_>,
    event_id: &str,
    observed_at: &str,
) -> Result<u32, String> {
    let mut statement = transaction
        .prepare(
            "SELECT DISTINCT detection.detection_id
             FROM detections AS detection
             JOIN detection_evidence AS evidence
               ON evidence.detection_id = detection.detection_id
             WHERE evidence.source_event_id = ?1 AND detection.condition_active = 1",
        )
        .map_err(|_| "Unable to prepare affected detection query")?;
    let ids = statement
        .query_map([event_id], |row| row.get::<_, String>(0))
        .map_err(|_| "Unable to query affected detections")?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Unable to read affected detections")?;
    drop(statement);
    for detection_id in &ids {
        transaction
            .execute(
                "UPDATE detections SET condition_active = 0, updated_at = ?1
                 WHERE detection_id = ?2",
                params![Utc::now().to_rfc3339(), detection_id],
            )
            .map_err(|_| "Unable to deactivate detection")?;
        append_detection_history(
            transaction,
            detection_id,
            "inactive",
            observed_at,
            "A required factual condition is no longer active",
            None,
            None,
        )?;
    }
    Ok(ids.len() as u32)
}

fn deactivate_rule_detections(
    transaction: &Transaction<'_>,
    rule_id: &str,
    observed_at: &str,
    transition: &str,
) -> Result<(), String> {
    let mut statement = transaction
        .prepare(
            "SELECT detection_id FROM detections
             WHERE rule_id = ?1 AND condition_active = 1",
        )
        .map_err(|_| "Unable to prepare rule detection query")?;
    let ids = statement
        .query_map([rule_id], |row| row.get::<_, String>(0))
        .map_err(|_| "Unable to query rule detections")?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Unable to read rule detections")?;
    drop(statement);
    for detection_id in ids {
        transaction
            .execute(
                "UPDATE detections SET condition_active = 0, updated_at = ?1
                 WHERE detection_id = ?2",
                params![observed_at, detection_id],
            )
            .map_err(|_| "Unable to deactivate rule detection")?;
        append_detection_history(
            transaction,
            &detection_id,
            transition,
            observed_at,
            if transition == "rule_superseded" {
                "A newer immutable rule version became current"
            } else {
                "The local rule was disabled"
            },
            None,
            None,
        )?;
    }
    Ok(())
}

fn supersede_prior_rule_versions(
    transaction: &Transaction<'_>,
    rule_id: &str,
    current_version: u32,
    observed_at: &str,
) -> Result<(), String> {
    let mut statement = transaction
        .prepare(
            "SELECT detection_id FROM detections
             WHERE rule_id = ?1 AND rule_version <> ?2 AND condition_active = 1",
        )
        .map_err(|_| "Unable to prepare superseded detection query")?;
    let ids = statement
        .query_map(params![rule_id, current_version], |row| {
            row.get::<_, String>(0)
        })
        .map_err(|_| "Unable to query superseded detections")?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Unable to read superseded detections")?;
    drop(statement);
    for detection_id in ids {
        transaction
            .execute(
                "UPDATE detections SET condition_active = 0, updated_at = ?1
                 WHERE detection_id = ?2",
                params![observed_at, detection_id],
            )
            .map_err(|_| "Unable to deactivate superseded detection")?;
        append_detection_history(
            transaction,
            &detection_id,
            "rule_superseded",
            observed_at,
            "A newer immutable rule version became current",
            None,
            None,
        )?;
    }
    Ok(())
}

fn load_existing_detection(
    transaction: &Transaction<'_>,
    candidate: &RuleMatch,
) -> Result<Option<ExistingDetection>, String> {
    transaction
        .query_row(
            "SELECT status, severity, confidence, condition_active
             FROM detections
             WHERE baseline_id = ?1 AND rule_id = ?2 AND rule_version = ?3
               AND dedup_key = ?4",
            params![
                candidate.baseline_id,
                candidate.rule_id,
                candidate.rule_version,
                candidate.dedup_key
            ],
            map_existing_detection,
        )
        .optional()
        .map_err(|_| "Unable to inspect existing detection".into())
}

fn load_existing_detection_by_id(
    transaction: &Transaction<'_>,
    detection_id: &str,
) -> Result<Option<ExistingDetection>, String> {
    transaction
        .query_row(
            "SELECT status, severity, confidence, condition_active
             FROM detections WHERE detection_id = ?1",
            [detection_id],
            map_existing_detection,
        )
        .optional()
        .map_err(|_| "Unable to inspect detection".into())
}

fn map_existing_detection(row: &rusqlite::Row<'_>) -> rusqlite::Result<ExistingDetection> {
    let status: String = row.get(0)?;
    let severity: String = row.get(1)?;
    let confidence: String = row.get(2)?;
    Ok(ExistingDetection {
        status: DetectionStatus::from_persisted(&status).map_err(invalid_data)?,
        severity: DetectionSeverity::from_persisted(&severity).map_err(invalid_data)?,
        confidence: DetectionConfidence::from_persisted(&confidence).map_err(invalid_data)?,
        condition_active: row.get::<_, i64>(3)? != 0,
    })
}

fn invalid_data(message: String) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        0,
        rusqlite::types::Type::Text,
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            message,
        )),
    )
}

fn detection_identity(candidate: &RuleMatch) -> String {
    format!(
        "detection-{}",
        &stable_hash(&format!(
            "detection-v1|{}|{}|{}|{}",
            candidate.baseline_id, candidate.rule_id, candidate.rule_version, candidate.dedup_key
        ))[..32]
    )
}

fn merge_match_evidence(existing: &mut RuleMatch, candidate: &RuleMatch) {
    let identities = existing
        .evidence
        .iter()
        .map(|(_, _, event)| event.history_id)
        .collect::<HashSet<_>>();
    existing.evidence.extend(
        candidate
            .evidence
            .iter()
            .filter(|(_, _, event)| !identities.contains(&event.history_id))
            .cloned(),
    );
    if candidate.observed_at > existing.observed_at {
        existing.observed_at = candidate.observed_at.clone();
    }
}

fn evidence_text<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
}

fn paths_match(left: Option<&str>, right: Option<&str>) -> bool {
    left.zip(right)
        .is_some_and(|(left, right)| normalize_windows_path(left) == normalize_windows_path(right))
}

fn is_user_temp_path(path: &str) -> bool {
    normalize_windows_path(path).contains("\\appdata\\local\\temp\\")
}

fn is_user_writable_path(path: &str) -> bool {
    let path = normalize_windows_path(path);
    path.contains("\\users\\")
        && (path.contains("\\appdata\\")
            || path.contains("\\downloads\\")
            || path.contains("\\desktop\\"))
}

fn normalize_windows_path(path: &str) -> String {
    path.trim()
        .trim_matches('"')
        .replace('/', "\\")
        .to_lowercase()
}

fn is_loopback(ip: &str) -> bool {
    !is_eligible_remote_ip(ip)
}

fn is_eligible_remote_ip(ip: &str) -> bool {
    ip.parse::<std::net::IpAddr>()
        .is_ok_and(|value| match value {
            std::net::IpAddr::V4(value) => {
                !value.is_loopback() && !value.is_unspecified() && !value.is_multicast()
            }
            std::net::IpAddr::V6(value) => {
                !value.is_loopback() && !value.is_unspecified() && !value.is_multicast()
            }
        })
}

fn service_binary_executable_path(value: &str) -> Option<&str> {
    let value = value.trim();
    if let Some(quoted) = value.strip_prefix('"') {
        return quoted.split_once('"').map(|(path, _)| path);
    }
    let end = value.to_ascii_lowercase().find(".exe")? + 4;
    Some(&value[..end])
}

fn has_before_after(value: &Value) -> bool {
    value.get("before").is_some_and(|item| !item.is_null())
        && value.get("after").is_some_and(|item| !item.is_null())
}

fn is_privileged_service_account(account: &str) -> bool {
    matches!(
        account.trim().to_ascii_lowercase().as_str(),
        "localsystem" | "local system" | "nt authority\\system"
    )
}

fn baseline_has_path(transaction: &Transaction<'_>, baseline_id: &str, path: &str) -> bool {
    transaction
        .query_row(
            "SELECT EXISTS(
                SELECT 1 FROM baseline_executables
                WHERE baseline_id = ?1 AND normalized_path = ?2
            )",
            params![baseline_id, normalize_windows_path(path)],
            |row| row.get(0),
        )
        .unwrap_or(true)
}

fn parse_timestamp(value: &str) -> Result<DateTime<Utc>, String> {
    DateTime::parse_from_rfc3339(value)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|_| "Factual event timestamp is invalid".into())
}

fn validate_identifier(value: &str, field: &str) -> Result<(), String> {
    if value.trim().is_empty() || value.len() > 256 {
        Err(format!("{field} is invalid"))
    } else {
        Ok(())
    }
}

fn stable_hash(value: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(value.as_bytes());
    format!("{:x}", digest.finalize())
}

#[cfg(test)]
mod tests {
    use super::{
        evaluate_net_001, evaluate_net_002, evaluate_proc_001, evaluate_proc_002, evaluate_svc_001,
        evaluate_svc_002, is_user_temp_path, stable_hash, DetectionEngine, FactualEvent,
    };
    use crate::{
        models::{DetectionConfidence, DetectionSeverity, DetectionStatus, DetectionStatusInput},
        persistence::Database,
        rules::registry,
    };
    use rusqlite::params;
    use serde_json::json;

    fn fact(history_id: i64, event_type: &str, evidence: serde_json::Value) -> FactualEvent {
        FactualEvent {
            history_id,
            event_id: format!("event-{history_id}"),
            transition: "first_observed".into(),
            observed_at: "2026-08-16T10:01:00Z".into(),
            source: "fixture".into(),
            event_type: event_type.into(),
            entity_key: format!("entity-{history_id}"),
            baseline_id: "baseline-detection".into(),
            evidence,
            event_schema_version: 2,
            observation_count: 1,
            condition_active: true,
        }
    }

    fn seed_baseline(database: &Database) {
        database
            .analysis_transaction(|transaction| {
                transaction
                    .execute(
                        "INSERT INTO behavioral_baselines(
                            baseline_id, created_at, learning_started_at, version, host_id,
                            status, learning_period_seconds, updated_at
                         ) VALUES ('baseline-detection', '2026-08-16T10:00:00Z',
                                   '2026-08-16T10:00:00Z', 1, 'host-detection', 'ready',
                                   86400, '2026-08-16T10:00:00Z')",
                        [],
                    )
                    .map_err(|_| "seed baseline")?;
                Ok(())
            })
            .expect("baseline");
    }

    fn append_event(database: &Database, event_id: &str, transition: &str, active: bool) {
        let process_event_id = format!("{event_id}-process");
        database
            .analysis_transaction(|transaction| {
                transaction
                    .execute(
                        "INSERT INTO security_events(
                            id, event_type, occurred_at, source, payload_json, entity_type,
                            entity_key, title, first_seen_at, last_seen_at,
                            baseline_context_json, baseline_id, status, observation_count,
                            condition_active, schema_version
                         ) VALUES (?1, 'executable_first_seen', '2026-08-16T10:01:00Z',
                                   'processes', ?2, 'executable', 'exe-key', 'Executable',
                                   '2026-08-16T10:01:00Z', '2026-08-16T10:01:00Z', '{}',
                                   'baseline-detection', 'new', 1, ?3, 2)
                         ON CONFLICT(id) DO UPDATE SET condition_active = excluded.condition_active",
                        params![
                            event_id,
                            json!({
                                "process": "fixture.exe",
                                "path": "C:\\Users\\fixture\\AppData\\Local\\Temp\\fixture.exe",
                                "signatureStatus": "unsigned",
                                "executableKey": "exe-key"
                            })
                            .to_string(),
                            active as i64
                        ],
                    )
                    .map_err(|_| "seed event")?;
                transaction
                    .execute(
                        "INSERT INTO security_event_history(
                            event_id, transition, observed_at, recorded_at, source, event_type,
                            entity_type, entity_key, baseline_id, evidence_json,
                            baseline_context_json, event_schema_version,
                            history_schema_version, observation_count
                         ) VALUES (?1, ?2, '2026-08-16T10:01:00Z',
                                   '2026-08-16T10:01:00Z', 'processes',
                                   'executable_first_seen', 'executable', 'exe-key',
                                   'baseline-detection', ?3, '{}', 2, 1, 1)",
                        params![
                            event_id,
                            transition,
                            json!({
                                "process": "fixture.exe",
                                "path": "C:\\Users\\fixture\\AppData\\Local\\Temp\\fixture.exe",
                                "signatureStatus": "unsigned",
                                "executableKey": "exe-key"
                            })
                            .to_string()
                        ],
                    )
                    .map_err(|_| "seed history")?;
                transaction
                    .execute(
                        "INSERT INTO security_events(
                            id, event_type, occurred_at, source, payload_json, entity_type,
                            entity_key, title, first_seen_at, last_seen_at,
                            baseline_context_json, baseline_id, status, observation_count,
                            condition_active, schema_version
                         ) VALUES (?1, 'process_first_seen', '2026-08-16T10:01:00Z',
                                   'processes', ?2, 'process_pattern', 'pattern-key', 'Process',
                                   '2026-08-16T10:01:00Z', '2026-08-16T10:01:00Z', '{}',
                                   'baseline-detection', 'new', 1, ?3, 2)
                         ON CONFLICT(id) DO UPDATE SET condition_active = excluded.condition_active",
                        params![
                            process_event_id,
                            json!({
                                "process": "fixture.exe",
                                "path": "C:\\Users\\fixture\\AppData\\Local\\Temp\\fixture.exe",
                                "executableKey": "exe-key"
                            })
                            .to_string(),
                            active as i64
                        ],
                    )
                    .map_err(|_| "seed process event")?;
                transaction
                    .execute(
                        "INSERT INTO security_event_history(
                            event_id, transition, observed_at, recorded_at, source, event_type,
                            entity_type, entity_key, baseline_id, evidence_json,
                            baseline_context_json, event_schema_version,
                            history_schema_version, observation_count
                         ) VALUES (?1, ?2, '2026-08-16T10:01:00Z',
                                   '2026-08-16T10:01:00Z', 'processes',
                                   'process_first_seen', 'process_pattern', 'pattern-key',
                                   'baseline-detection', ?3, '{}', 2, 1, 1)",
                        params![
                            process_event_id,
                            transition,
                            json!({
                                "process": "fixture.exe",
                                "path": "C:\\Users\\fixture\\AppData\\Local\\Temp\\fixture.exe",
                                "executableKey": "exe-key"
                            })
                            .to_string()
                        ],
                    )
                    .map_err(|_| "seed process history")?;
                Ok(())
            })
            .expect("event");
    }

    #[test]
    fn path_policy_is_narrow_and_case_insensitive() {
        assert!(is_user_temp_path(
            "C:\\Users\\A\\AppData\\Local\\Temp\\sample.exe"
        ));
        assert!(!is_user_temp_path("C:\\Windows\\Temp\\sample.exe"));
        assert!(!is_user_temp_path("C:\\Tools\\sample.exe"));
    }

    #[test]
    fn proc_001_requires_execution_exact_unsigned_state_and_temp_path() {
        let database = Database::in_memory().expect("database");
        seed_baseline(&database);
        let executable = fact(
            1,
            "executable_first_seen",
            json!({"path":"C:\\Users\\fixture\\AppData\\Local\\Temp\\tool.exe","signatureStatus":"unsigned","executableKey":"exe-1"}),
        );
        let process = fact(
            2,
            "process_first_seen",
            json!({"path":"C:\\Users\\fixture\\AppData\\Local\\Temp\\tool.exe"}),
        );
        database
            .analysis_transaction(|transaction| {
                let detection =
                    evaluate_proc_001(transaction, &executable, std::slice::from_ref(&process), 1)
                        .expect("corroborated rule");
                assert_eq!(detection.severity, DetectionSeverity::Low);
                assert_eq!(detection.confidence, DetectionConfidence::High);
                let mut unknown = executable.clone();
                unknown.evidence["signatureStatus"] = json!("unknown");
                assert!(evaluate_proc_001(
                    transaction,
                    &unknown,
                    std::slice::from_ref(&process),
                    1
                )
                .is_none());
                assert!(evaluate_proc_001(transaction, &executable, &[], 1).is_none());
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn proc_002_requires_signed_known_parent_and_executed_child() {
        let database = Database::in_memory().expect("database");
        seed_baseline(&database);
        database
            .analysis_transaction(|transaction| {
                transaction
                    .execute(
                        "INSERT INTO baseline_executables(
                            baseline_id, executable_key, normalized_path, process_name,
                            signature_status, first_seen_at, last_seen_at
                         ) VALUES ('baseline-detection', 'parent-key',
                                   'c:\\program files\\vendor\\parent.exe', 'parent.exe',
                                   'signed', '2026-08-16T10:00:00Z', '2026-08-16T10:00:00Z')",
                        [],
                    )
                    .map_err(|_| "parent fixture")?;
                let relationship = fact(
                    1,
                    "parent_child_first_seen",
                    json!({"parentPath":"C:\\Program Files\\Vendor\\parent.exe","childPath":"C:\\Users\\fixture\\AppData\\Local\\Temp\\child.exe","childExecutableKey":"child-key"}),
                );
                let executable = fact(
                    2,
                    "executable_first_seen",
                    json!({"path":"C:\\Users\\fixture\\AppData\\Local\\Temp\\child.exe","signatureStatus":"unsigned","executableKey":"child-key"}),
                );
                let process = fact(
                    3,
                    "process_first_seen",
                    json!({"path":"C:\\Users\\fixture\\AppData\\Local\\Temp\\child.exe"}),
                );
                let evidence = vec![executable.clone(), process.clone()];
                let detection = evaluate_proc_002(transaction, &relationship, &evidence, 1)
                    .expect("parent-child rule");
                assert_eq!(detection.severity, DetectionSeverity::Medium);
                assert_eq!(detection.evidence.len(), 3);
                assert!(evaluate_proc_002(
                    transaction,
                    &relationship,
                    std::slice::from_ref(&executable),
                    1
                )
                .is_none());
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn net_001_rejects_loopback_and_requires_process_correlation() {
        let database = Database::in_memory().expect("database");
        seed_baseline(&database);
        let executable = fact(
            1,
            "executable_first_seen",
            json!({"process":"tool.exe","path":"C:\\Users\\fixture\\AppData\\Local\\Temp\\tool.exe","signatureStatus":"unsigned","executableKey":"exe-net"}),
        );
        let process = fact(
            2,
            "process_first_seen",
            json!({"process":"tool.exe","path":"C:\\Users\\fixture\\AppData\\Local\\Temp\\tool.exe"}),
        );
        let destination = fact(
            3,
            "destination_first_seen",
            json!({"process":"tool.exe","executableKey":"exe-net","association":"associated","remoteIp":"203.0.113.10"}),
        );
        database
            .analysis_transaction(|transaction| {
                let related = vec![executable.clone(), process.clone()];
                let detection =
                    evaluate_net_001(transaction, &destination, &related, 1).expect("network rule");
                assert_eq!(detection.severity, DetectionSeverity::Medium);
                assert_eq!(detection.confidence, DetectionConfidence::Medium);
                let mut loopback = destination.clone();
                loopback.evidence["remoteIp"] = json!("127.0.0.1");
                assert!(evaluate_net_001(transaction, &loopback, &related, 1).is_none());
                assert!(evaluate_net_001(
                    transaction,
                    &destination,
                    std::slice::from_ref(&executable),
                    1
                )
                .is_none());
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn svc_rules_require_persistent_correlated_configuration() {
        let service = fact(
            1,
            "service_first_seen",
            json!({"serviceKey":"svc-1","binaryPath":"\"C:\\Users\\fixture\\Downloads\\svc.exe\" --service","startupType":"Automatic","status":"Running","account":"LocalSystem"}),
        );
        let first = evaluate_svc_001(&service, 1).expect("service rule");
        assert_eq!(first.severity, DetectionSeverity::Medium);
        let mut manual = service.clone();
        manual.evidence["startupType"] = json!("Manual");
        assert!(evaluate_svc_001(&manual, 1).is_none());

        let binary = fact(
            2,
            "service_binary_changed",
            json!({"serviceKey":"svc-1","before":"C:\\old.exe","after":"C:\\new.exe"}),
        );
        let startup = fact(
            3,
            "service_startup_changed",
            json!({"serviceKey":"svc-1","before":"Manual","after":"Automatic"}),
        );
        let second = evaluate_svc_002(&binary, &[binary.clone(), startup], 1)
            .expect("correlated service changes");
        assert_eq!(second.confidence, DetectionConfidence::High);
        assert!(evaluate_svc_002(&binary, std::slice::from_ref(&binary), 1).is_none());
    }

    #[test]
    fn net_002_requires_two_complete_observations_and_no_route_change() {
        let mut gateway = fact(
            1,
            "gateway_changed",
            json!({"before":"192.0.2.1","after":"192.0.2.2","primaryInterfaceKey":"if-1"}),
        );
        gateway.observation_count = 2;
        let mut dns = fact(
            2,
            "dns_changed",
            json!({"before":["192.0.2.53"],"after":["203.0.113.53"],"primaryInterfaceKey":"if-1"}),
        );
        dns.observation_count = 2;
        let evidence = vec![gateway.clone(), dns.clone()];
        let detection = evaluate_net_002(&gateway, &evidence, 1).expect("network config rule");
        assert_eq!(detection.severity, DetectionSeverity::Low);
        let mut one_observation = dns.clone();
        one_observation.observation_count = 1;
        assert!(evaluate_net_002(&gateway, &[gateway.clone(), one_observation], 1).is_none());
        let route = fact(
            3,
            "primary_route_changed",
            json!({"before":"if-1","after":"if-2"}),
        );
        assert!(evaluate_net_002(&gateway, &[gateway.clone(), dns, route], 1).is_none());
    }

    #[test]
    fn registry_versions_are_immutable_and_local_enabled_state_is_separate() {
        let database = Database::in_memory().expect("database");
        let engine = DetectionEngine;
        let definition = registry()[0].clone();
        engine
            .initialize(&database, registry())
            .expect("initialize");
        engine
            .set_rule_enabled(&database, "EDY-PROC-001", false)
            .expect("disable");
        assert!(
            !engine
                .rule_definitions(&database)
                .unwrap()
                .into_iter()
                .find(|rule| rule.rule_id == "EDY-PROC-001")
                .expect("registered rule")
                .enabled
        );

        let mut changed = definition;
        changed.name = "Changed without version".into();
        assert!(engine.initialize(&database, &[changed]).is_err());
    }

    #[test]
    fn checkpoint_prevents_backfill_and_new_fact_deduplicates_and_reopens() {
        let database = Database::in_memory().expect("database");
        seed_baseline(&database);
        append_event(&database, "pre-v6-event", "first_observed", true);
        database
            .analysis_transaction(|transaction| {
                transaction
                    .execute(
                        "UPDATE analysis_checkpoints
                         SET last_security_event_history_id = (
                             SELECT MAX(history_id) FROM security_event_history
                         ) WHERE consumer_id = 'detection-engine-v1'",
                        [],
                    )
                    .map_err(|_| "cutover")?;
                Ok(())
            })
            .unwrap();
        let engine = DetectionEngine;
        engine.initialize(&database, registry()).unwrap();
        assert_eq!(
            engine.process_pending(&database).unwrap().processed_history,
            0
        );

        append_event(&database, "new-event", "first_observed", true);
        let first = engine.process_pending(&database).unwrap();
        assert_eq!(first.detections_changed, 1);
        append_event(&database, "new-event", "inactive", false);
        engine.process_pending(&database).unwrap();
        append_event(&database, "new-event", "reactivated", true);
        let second = engine.process_pending(&database).unwrap();
        assert_eq!(second.detections_changed, 1);

        let (count, detection_id): (u64, String) = database
            .analysis_read(|connection| {
                connection
                    .query_row(
                        "SELECT occurrence_count, detection_id FROM detections",
                        [],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .map_err(|_| "read detection".into())
            })
            .unwrap();
        assert_eq!(count, 2);
        engine
            .set_detection_status(
                &database,
                DetectionStatusInput {
                    detection_id: detection_id.clone(),
                    status: DetectionStatus::Resolved,
                },
            )
            .unwrap();

        append_event(&database, "new-event", "inactive", false);
        engine.process_pending(&database).unwrap();
        append_event(&database, "new-event", "reactivated", true);
        engine.process_pending(&database).unwrap();
        let status: String = database
            .analysis_read(|connection| {
                connection
                    .query_row(
                        "SELECT status FROM detections WHERE detection_id = ?1",
                        [detection_id],
                        |row| row.get(0),
                    )
                    .map_err(|_| "read status".into())
            })
            .unwrap();
        assert_eq!(status, "new");
    }

    #[test]
    fn detection_hash_is_stable() {
        assert_eq!(stable_hash("fixture"), stable_hash("fixture"));
    }
}
