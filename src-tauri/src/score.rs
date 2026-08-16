use crate::{
    models::{
        DetectionConfidence, DetectionSeverity, ScoreBreakdown, ScoreCoverage, ScoreState,
        SecurityScore,
    },
    persistence::Database,
};
use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, OptionalExtension, Transaction};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{collections::HashMap, time::Instant};

const SCORE_FORMULA_VERSION: u32 = 1;
const SCORE_SCHEMA_VERSION: u32 = 1;
const SCORE_HEARTBEAT_MINUTES: i64 = 15;
const SCORE_RETENTION_DAYS: i64 = 365;
const REQUIRED_COVERAGE_COMPONENTS: [&str; 4] = ["system", "processes", "connections", "services"];

#[derive(Clone, Default)]
pub struct ScoreEngine;

#[derive(Debug)]
struct ActiveDetection {
    correlation_key: String,
    title: String,
    severity: DetectionSeverity,
    confidence: DetectionConfidence,
}

impl ScoreEngine {
    /// Calculates a conservative, explainable score over current detections.
    /// Missing baseline/collector coverage produces an unavailable snapshot,
    /// never a fabricated numeric value.
    pub fn calculate_and_persist(
        &self,
        database: &Database,
        mut coverage: Vec<ScoreCoverage>,
    ) -> Result<SecurityScore, String> {
        database.analysis_transaction(|transaction| {
            let started = Instant::now();
            coverage = merge_previous_coverage(transaction, coverage)?;
            coverage.sort_by(|left, right| left.component.cmp(&right.component));
            let active_baseline = transaction
                .query_row(
                    "SELECT baseline_id, status FROM behavioral_baselines WHERE active = 1",
                    [],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
                )
                .optional()
                .map_err(|_| "Unable to read baseline coverage")?;
            let enabled_rule_count: u64 = transaction
                .query_row(
                    "SELECT COUNT(*) FROM detection_rule_state WHERE enabled = 1",
                    [],
                    |row| row.get(0),
                )
                .map_err(|_| "Unable to read detection rule coverage")?;

            let (state, reason, baseline_id) = availability(
                active_baseline.as_ref().map(|(_, status)| status.as_str()),
                active_baseline
                    .as_ref()
                    .map(|(baseline_id, _)| baseline_id.clone()),
                enabled_rule_count,
                &coverage,
            );
            let mut score = SecurityScore {
                state,
                score: None,
                label: None,
                generated_at: Some(Utc::now().to_rfc3339()),
                formula_version: SCORE_FORMULA_VERSION,
                active_detection_count: 0,
                highest_severity: None,
                coverage,
                breakdown: Vec::new(),
                reason,
            };

            if matches!(state, ScoreState::Available | ScoreState::Limited) {
                let baseline_id = baseline_id
                    .as_deref()
                    .ok_or_else(|| "Score baseline is unavailable".to_string())?;
                let detections = load_active_detections(transaction, baseline_id)?;
                score.active_detection_count = detections.len() as u64;
                score.highest_severity = detections.iter().map(|value| value.severity).max();
                score.breakdown = grouped_breakdown(detections);
                let penalty = score
                    .breakdown
                    .iter()
                    .map(|item| item.penalty)
                    .sum::<u32>()
                    .min(100);
                let numeric = 100u32.saturating_sub(penalty);
                score.score = Some(numeric);
                score.label = Some(score_label(numeric).into());
            }

            persist_snapshot(
                transaction,
                baseline_id.as_deref(),
                &score,
                started.elapsed().as_millis() as u64,
            )?;
            enforce_score_retention(transaction)?;
            Ok(score)
        })
    }

    pub fn current(&self, database: &Database) -> Result<SecurityScore, String> {
        database.analysis_read(|connection| {
            connection
                .query_row(
                    "SELECT calculated_at, score, availability, formula_version,
                            active_detection_count, highest_severity, coverage_json,
                            breakdown_json, reason
                     FROM security_score_snapshots
                     ORDER BY calculated_at DESC, snapshot_id DESC LIMIT 1",
                    [],
                    |row| {
                        let calculated_at: String = row.get(0)?;
                        let state_value: String = row.get(2)?;
                        let severity_value: Option<String> = row.get(5)?;
                        let score: Option<u32> = row.get(1)?;
                        Ok(SecurityScore {
                            state: ScoreState::from_persisted(&state_value)
                                .map_err(invalid_data)?,
                            score,
                            label: score.map(score_label).map(str::to_owned),
                            generated_at: Some(calculated_at),
                            formula_version: row.get(3)?,
                            active_detection_count: row.get(4)?,
                            highest_severity: severity_value
                                .as_deref()
                                .map(DetectionSeverity::from_persisted)
                                .transpose()
                                .map_err(invalid_data)?,
                            coverage: serde_json::from_str(&row.get::<_, String>(6)?)
                                .map_err(|error| invalid_data(error.to_string()))?,
                            breakdown: serde_json::from_str(&row.get::<_, String>(7)?)
                                .map_err(|error| invalid_data(error.to_string()))?,
                            reason: row.get(8)?,
                        })
                    },
                )
                .optional()
                .map_err(|_| "Unable to read Security Score")?
                .ok_or_else(|| "Security Score has not been calculated yet".into())
        })
    }
}

fn availability(
    baseline_status: Option<&str>,
    baseline_id: Option<String>,
    enabled_rule_count: u64,
    coverage: &[ScoreCoverage],
) -> (ScoreState, Option<String>, Option<String>) {
    if baseline_status != Some("ready") {
        let reason = match baseline_status {
            None => "Score unavailable — no behavioral baseline exists",
            Some("learning") => "Score unavailable — baseline still learning",
            Some("stale") => "Score unavailable — baseline coverage is stale",
            Some("error") => "Score unavailable — baseline is in Error",
            _ => "Score unavailable — baseline is not ready",
        };
        return (ScoreState::Unavailable, Some(reason.into()), baseline_id);
    }
    if enabled_rule_count == 0 {
        return (
            ScoreState::Unavailable,
            Some("Score unavailable — no detection rules are enabled".into()),
            baseline_id,
        );
    }
    if coverage.is_empty() {
        return (
            ScoreState::Unavailable,
            Some("Score unavailable — collector coverage has not been measured".into()),
            baseline_id,
        );
    }
    if REQUIRED_COVERAGE_COMPONENTS.iter().any(|required| {
        !coverage
            .iter()
            .any(|item| item.component.eq_ignore_ascii_case(required))
    }) {
        return (
            ScoreState::Unavailable,
            Some("Score unavailable — required collector coverage is incomplete".into()),
            baseline_id,
        );
    }
    if coverage.iter().any(|item| {
        matches!(
            item.status.to_ascii_lowercase().as_str(),
            "failed" | "unavailable" | "paused"
        )
    }) {
        return (
            ScoreState::Unavailable,
            Some("Score unavailable — required collector coverage is incomplete".into()),
            baseline_id,
        );
    }
    if coverage
        .iter()
        .any(|item| item.status.eq_ignore_ascii_case("degraded"))
    {
        return (
            ScoreState::Limited,
            Some("Limited coverage — restricted metadata may reduce rule visibility".into()),
            baseline_id,
        );
    }
    (ScoreState::Available, None, baseline_id)
}

fn merge_previous_coverage(
    transaction: &Transaction<'_>,
    current: Vec<ScoreCoverage>,
) -> Result<Vec<ScoreCoverage>, String> {
    let previous: Option<String> = transaction
        .query_row(
            "SELECT coverage_json FROM security_score_snapshots
             ORDER BY calculated_at DESC, snapshot_id DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "Unable to read previous score coverage")?;
    let mut merged = previous
        .as_deref()
        .and_then(|json| serde_json::from_str::<Vec<ScoreCoverage>>(json).ok())
        .unwrap_or_default()
        .into_iter()
        .map(|item| (item.component.clone(), item))
        .collect::<HashMap<_, _>>();
    for item in current {
        merged.insert(item.component.clone(), item);
    }
    Ok(merged.into_values().collect())
}

fn load_active_detections(
    transaction: &Transaction<'_>,
    baseline_id: &str,
) -> Result<Vec<ActiveDetection>, String> {
    let mut statement = transaction
        .prepare(
            "SELECT detection.correlation_key, detection.title, detection.severity,
                    detection.confidence
             FROM detections AS detection
             JOIN detection_rule_state AS state
               ON state.rule_id = detection.rule_id
              AND state.rule_version = detection.rule_version
              AND state.enabled = 1
             WHERE detection.baseline_id = ?1
               AND detection.condition_active = 1
               AND detection.status NOT IN ('resolved', 'ignored')",
        )
        .map_err(|_| "Unable to prepare active detection score query")?;
    let rows = statement
        .query_map([baseline_id], |row| {
            let severity: String = row.get(2)?;
            let confidence: String = row.get(3)?;
            Ok(ActiveDetection {
                correlation_key: row.get(0)?,
                title: row.get(1)?,
                severity: DetectionSeverity::from_persisted(&severity).map_err(invalid_data)?,
                confidence: DetectionConfidence::from_persisted(&confidence)
                    .map_err(invalid_data)?,
            })
        })
        .map_err(|_| "Unable to query active detections for score")?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Unable to read active detections for score".into())
}

fn grouped_breakdown(detections: Vec<ActiveDetection>) -> Vec<ScoreBreakdown> {
    let mut groups = HashMap::<String, ScoreBreakdown>::new();
    for detection in detections {
        let penalty = detection_penalty(detection.severity, detection.confidence);
        let item = ScoreBreakdown {
            correlation_key: detection.correlation_key.clone(),
            title: detection.title,
            severity: detection.severity,
            confidence: detection.confidence,
            penalty,
        };
        groups
            .entry(detection.correlation_key)
            .and_modify(|current| {
                if item.penalty > current.penalty {
                    *current = item.clone();
                }
            })
            .or_insert(item);
    }
    let mut values = groups.into_values().collect::<Vec<_>>();
    values.sort_by(|left, right| {
        right
            .penalty
            .cmp(&left.penalty)
            .then(left.correlation_key.cmp(&right.correlation_key))
    });
    values
}

fn detection_penalty(severity: DetectionSeverity, confidence: DetectionConfidence) -> u32 {
    let base: f64 = match severity {
        DetectionSeverity::Informational => 0.0,
        DetectionSeverity::Low => 3.0,
        DetectionSeverity::Medium => 8.0,
        DetectionSeverity::High => 18.0,
        DetectionSeverity::Critical => 35.0,
    };
    let factor: f64 = match confidence {
        DetectionConfidence::Low => 0.5,
        DetectionConfidence::Medium => 0.75,
        DetectionConfidence::High => 1.0,
    };
    (base * factor).round() as u32
}

fn score_label(score: u32) -> &'static str {
    match score {
        90..=100 => "Excellent",
        75..=89 => "Good",
        50..=74 => "Attention",
        _ => "Elevated Risk",
    }
}

fn persist_snapshot(
    transaction: &Transaction<'_>,
    baseline_id: Option<&str>,
    score: &SecurityScore,
    calculation_duration_ms: u64,
) -> Result<(), String> {
    let coverage_json =
        serde_json::to_string(&score.coverage).map_err(|_| "Unable to serialize score coverage")?;
    let breakdown_json = serde_json::to_string(&score.breakdown)
        .map_err(|_| "Unable to serialize score breakdown")?;
    let fingerprint = stable_hash(
        &json!({
            "formulaVersion": SCORE_FORMULA_VERSION,
            "state": score.state,
            "score": score.score,
            "activeDetectionCount": score.active_detection_count,
            "highestSeverity": score.highest_severity,
            "coverage": score.coverage,
            "breakdown": score.breakdown,
            "reason": score.reason,
        })
        .to_string(),
    );
    let previous: Option<(String, String)> = transaction
        .query_row(
            "SELECT input_fingerprint, calculated_at
             FROM security_score_snapshots
             ORDER BY calculated_at DESC, snapshot_id DESC LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|_| "Unable to inspect previous score snapshot")?;
    let heartbeat_due = match previous.as_ref() {
        None => true,
        Some((_, calculated_at)) => match parse_timestamp(calculated_at) {
            None => true,
            Some(timestamp) => {
                Utc::now().signed_duration_since(timestamp)
                    >= Duration::minutes(SCORE_HEARTBEAT_MINUTES)
            }
        },
    };
    if previous
        .as_ref()
        .is_some_and(|(previous, _)| previous == &fingerprint)
        && !heartbeat_due
    {
        return Ok(());
    }
    let calculated_at = Utc::now().to_rfc3339();
    let snapshot_id = format!(
        "score-{}",
        &stable_hash(&format!("score-v1|{calculated_at}|{fingerprint}"))[..32]
    );
    transaction
        .execute(
            "INSERT INTO security_score_snapshots(
                snapshot_id, calculated_at, score, availability, baseline_id,
                formula_version, active_detection_count, highest_severity,
                coverage_json, breakdown_json, reason, input_fingerprint,
                calculation_duration_ms, schema_version
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                snapshot_id,
                calculated_at,
                score.score,
                score.state.as_str(),
                baseline_id,
                SCORE_FORMULA_VERSION,
                score.active_detection_count,
                score.highest_severity.map(DetectionSeverity::as_str),
                coverage_json,
                breakdown_json,
                score.reason,
                fingerprint,
                calculation_duration_ms,
                SCORE_SCHEMA_VERSION
            ],
        )
        .map(|_| ())
        .map_err(|_| "Unable to persist Security Score snapshot".into())
}

fn enforce_score_retention(transaction: &Transaction<'_>) -> Result<(), String> {
    let cutoff = (Utc::now() - Duration::days(SCORE_RETENTION_DAYS)).to_rfc3339();
    transaction
        .execute(
            "DELETE FROM security_score_snapshots WHERE calculated_at < ?1",
            [cutoff],
        )
        .map(|_| ())
        .map_err(|_| "Unable to enforce Security Score retention".into())
}

fn parse_timestamp(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|value| value.with_timezone(&Utc))
}

fn stable_hash(value: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(value.as_bytes());
    format!("{:x}", digest.finalize())
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

#[cfg(test)]
mod tests {
    use super::{availability, detection_penalty, grouped_breakdown, ActiveDetection, ScoreEngine};
    use crate::{
        detection::DetectionEngine,
        models::{DetectionConfidence, DetectionSeverity, ScoreCoverage, ScoreState},
        persistence::Database,
        rules::registry,
    };

    fn healthy_coverage() -> Vec<ScoreCoverage> {
        ["system", "processes", "connections", "services"]
            .into_iter()
            .map(|component| ScoreCoverage {
                component: component.into(),
                status: "healthy".into(),
                detail: "fixture".into(),
            })
            .collect()
    }

    #[test]
    fn formula_is_conservative_and_does_not_stack_one_correlation_group() {
        assert_eq!(
            detection_penalty(DetectionSeverity::Medium, DetectionConfidence::High),
            8
        );
        assert_eq!(
            detection_penalty(DetectionSeverity::Medium, DetectionConfidence::Medium),
            6
        );
        let breakdown = grouped_breakdown(vec![
            ActiveDetection {
                correlation_key: "same-executable".into(),
                title: "Low".into(),
                severity: DetectionSeverity::Low,
                confidence: DetectionConfidence::High,
            },
            ActiveDetection {
                correlation_key: "same-executable".into(),
                title: "Medium".into(),
                severity: DetectionSeverity::Medium,
                confidence: DetectionConfidence::High,
            },
        ]);
        assert_eq!(breakdown.len(), 1);
        assert_eq!(breakdown[0].penalty, 8);
        assert_eq!(breakdown[0].title, "Medium");
    }

    #[test]
    fn score_availability_fails_closed_for_baseline_and_coverage() {
        let (state, reason, _) = availability(None, None, 6, &healthy_coverage());
        assert_eq!(state, ScoreState::Unavailable);
        assert!(reason.unwrap().contains("no behavioral baseline"));
        let (state, _, _) = availability(
            Some("learning"),
            Some("baseline".into()),
            6,
            &healthy_coverage(),
        );
        assert_eq!(state, ScoreState::Unavailable);
        let (state, _, _) = availability(Some("ready"), Some("baseline".into()), 6, &[]);
        assert_eq!(state, ScoreState::Unavailable);
        let mut incomplete = healthy_coverage();
        incomplete.retain(|item| item.component != "services");
        let (state, _, _) = availability(Some("ready"), Some("baseline".into()), 6, &incomplete);
        assert_eq!(state, ScoreState::Unavailable);
        let mut degraded = healthy_coverage();
        degraded[0].status = "degraded".into();
        let (state, _, _) = availability(Some("ready"), Some("baseline".into()), 6, &degraded);
        assert_eq!(state, ScoreState::Limited);
    }

    #[test]
    fn current_score_excludes_resolved_and_ignored_but_keeps_acknowledged_active() {
        let database = Database::in_memory().expect("database");
        DetectionEngine
            .initialize(&database, registry())
            .expect("rules");
        database
            .analysis_transaction(|transaction| {
                transaction
                    .execute(
                        "INSERT INTO behavioral_baselines(
                            baseline_id, created_at, learning_started_at, version, host_id,
                            status, learning_period_seconds, updated_at, active
                         ) VALUES ('score-baseline', '2026-08-16T10:00:00Z',
                                   '2026-08-16T10:00:00Z', 1, 'score-host', 'ready',
                                   86400, '2026-08-16T10:00:00Z', 1)",
                        [],
                    )
                    .map_err(|_| "baseline fixture")?;
                transaction
                    .execute(
                        "INSERT INTO detections(
                            detection_id, dedup_key, correlation_key, rule_id, rule_version,
                            baseline_id, entity_type, entity_key, title, summary, severity,
                            confidence, severity_reason, confidence_reason,
                            remediation_guidance_json, status, first_detected_at,
                            last_detected_at, occurrence_count, condition_active,
                            explanation_json, created_at, updated_at
                         ) VALUES ('score-detection', 'dedup', 'entity-group', 'EDY-PROC-002', 1,
                                   'score-baseline', 'process', 'entity', 'Fixture detection',
                                   'Fixture summary', 'medium', 'high', 'corroborated facts',
                                   'complete evidence', '[]', 'acknowledged',
                                   '2026-08-16T10:01:00Z', '2026-08-16T10:02:00Z', 50, 1,
                                   '{}', '2026-08-16T10:01:00Z', '2026-08-16T10:02:00Z')",
                        [],
                    )
                    .map_err(|_| "detection fixture")?;
                Ok(())
            })
            .unwrap();
        let engine = ScoreEngine;
        let acknowledged = engine
            .calculate_and_persist(&database, healthy_coverage())
            .expect("acknowledged score");
        assert_eq!(acknowledged.score, Some(92));
        assert_eq!(acknowledged.active_detection_count, 1);

        for status in ["resolved", "ignored"] {
            database
                .analysis_transaction(|transaction| {
                    transaction
                        .execute(
                            "UPDATE detections SET status = ?1 WHERE detection_id = 'score-detection'",
                            [status],
                        )
                        .map_err(|_| "status fixture")?;
                    Ok(())
                })
                .unwrap();
            let score = engine
                .calculate_and_persist(&database, Vec::new())
                .expect("closed score");
            assert_eq!(score.score, Some(100));
            assert_eq!(score.active_detection_count, 0);
        }
        assert_eq!(engine.current(&database).unwrap().formula_version, 1);
    }
}
