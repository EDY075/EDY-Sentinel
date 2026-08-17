use crate::{
    models::{
        DetectionConfidence, DetectionSeverity, ProductVulnerabilityRisk, ScoreBreakdown,
        ScoreCoverage, ScoreState, SecurityScore, VulnerabilityCoverage, VulnerabilityCvssBands,
    },
    persistence::Database,
};
use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, OptionalExtension, Transaction};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    time::Instant,
};

const SCORE_FORMULA_VERSION: u32 = 2;
const SCORE_SCHEMA_VERSION: u32 = 2;
const SCORE_HEARTBEAT_MINUTES: i64 = 15;
const SCORE_RETENTION_DAYS: i64 = 365;
const PRODUCT_IMPACT_CAP: u32 = 7;
const VULNERABILITY_CATEGORY_CAP: u32 = 18;
const MARGINAL_BREADTH_CAP: f64 = 2.0;
const KEV_BOOST: f64 = 3.0;
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

#[derive(Debug)]
struct VulnerabilityRiskInput {
    canonical_vendor: String,
    canonical_product: String,
    display_name: String,
    software_id: String,
    installed_version: Option<String>,
    cve_id: String,
    match_state: String,
    confidence: String,
    cvss_score: Option<f64>,
    kev: bool,
    matching_engine_version: u32,
    identity_resolver_version: u32,
    nvd_source_version: Option<String>,
    kev_source_version: Option<String>,
    evaluated_at: String,
}

#[derive(Debug, Default)]
struct CveRiskAccumulator {
    confirmed_cvss: Option<Option<f64>>,
    possible: bool,
    kev: bool,
}

#[derive(Debug, Default)]
struct ProductRiskAccumulator {
    canonical_vendor: String,
    canonical_product: String,
    display_names: BTreeSet<String>,
    software_ids: BTreeSet<String>,
    installed_versions: BTreeSet<String>,
    cves: BTreeMap<String, CveRiskAccumulator>,
    matching_engine_versions: BTreeSet<u32>,
    identity_resolver_versions: BTreeSet<u32>,
    nvd_source_versions: BTreeSet<String>,
    kev_source_versions: BTreeSet<String>,
    evaluated_at: String,
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
                detection_penalty: 0,
                vulnerability_penalty: 0,
                vulnerability_coverage: unavailable_vulnerability_coverage(),
                product_vulnerability_risks: Vec::new(),
                reason,
            };

            let (vulnerability_coverage, product_risks, vulnerability_penalty) =
                load_vulnerability_component(transaction)?;
            score.vulnerability_coverage = vulnerability_coverage;
            score.product_vulnerability_risks = product_risks;
            score.vulnerability_penalty = vulnerability_penalty;
            if score.state == ScoreState::Available
                && score.vulnerability_coverage.status == "updating"
            {
                score.state = ScoreState::Limited;
                score.reason =
                    Some("Limited coverage — Vulnerability Intelligence is updating".into());
            }

            if matches!(state, ScoreState::Available | ScoreState::Limited) {
                let baseline_id = baseline_id
                    .as_deref()
                    .ok_or_else(|| "Score baseline is unavailable".to_string())?;
                let detections = load_active_detections(transaction, baseline_id)?;
                score.active_detection_count = detections.len() as u64;
                score.highest_severity = detections.iter().map(|value| value.severity).max();
                score.breakdown = grouped_breakdown(detections);
                score.detection_penalty = score
                    .breakdown
                    .iter()
                    .map(|item| item.penalty)
                    .sum::<u32>()
                    .min(100);
                let numeric = final_score(score.detection_penalty, score.vulnerability_penalty);
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
                            breakdown_json, reason, detection_penalty,
                            vulnerability_penalty, vulnerability_coverage_json,
                            product_vulnerability_risks_json
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
                            detection_penalty: row.get(9)?,
                            vulnerability_penalty: row.get(10)?,
                            vulnerability_coverage: serde_json::from_str(
                                &row.get::<_, String>(11)?,
                            )
                            .map_err(|error| invalid_data(error.to_string()))?,
                            product_vulnerability_risks: serde_json::from_str(
                                &row.get::<_, String>(12)?,
                            )
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

fn unavailable_vulnerability_coverage() -> VulnerabilityCoverage {
    VulnerabilityCoverage {
        status: "unavailable".into(),
        basis: "software_record_proxy".into(),
        total_software: 0,
        eligible_software: 0,
        resolved_eligible: 0,
        ambiguous_eligible: 0,
        unresolved_eligible: 0,
        not_mappable: 0,
        pending_evaluation: 0,
    }
}

fn load_vulnerability_component(
    transaction: &Transaction<'_>,
) -> Result<(VulnerabilityCoverage, Vec<ProductVulnerabilityRisk>, u32), String> {
    let coverage = load_vulnerability_coverage(transaction)?;
    let inputs = load_vulnerability_risk_inputs(transaction)?;
    let product_risks = build_product_risks(inputs);
    let penalty = vulnerability_penalty(&product_risks);
    Ok((coverage, product_risks, penalty))
}

fn vulnerability_penalty(product_risks: &[ProductVulnerabilityRisk]) -> u32 {
    product_risks
        .iter()
        .map(|product| product.impact)
        .sum::<u32>()
        .min(VULNERABILITY_CATEGORY_CAP)
}

fn final_score(detection_penalty: u32, vulnerability_penalty: u32) -> u32 {
    100u32.saturating_sub(
        detection_penalty
            .saturating_add(vulnerability_penalty)
            .min(100),
    )
}

fn load_vulnerability_coverage(
    transaction: &Transaction<'_>,
) -> Result<VulnerabilityCoverage, String> {
    let mut statement = transaction
        .prepare(
            "WITH ranked AS (
                SELECT evaluation_id, software_id,
                       ROW_NUMBER() OVER (
                           PARTITION BY software_id
                           ORDER BY completed_at DESC, evaluation_id DESC
                       ) AS position
                FROM software_vulnerability_evaluations
             )
             SELECT s.software_id, l.evaluation_id, i.resolver_status,
                    i.unresolved_reason, q.reason
             FROM installed_software s
             LEFT JOIN ranked l ON l.software_id = s.software_id AND l.position = 1
             LEFT JOIN software_identity_evaluations i ON i.evaluation_id = l.evaluation_id
             LEFT JOIN vulnerability_evaluation_queue q ON q.software_id = s.software_id
             WHERE s.active = 1",
        )
        .map_err(|_| "Unable to prepare vulnerability coverage query")?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        })
        .map_err(|_| "Unable to query vulnerability coverage")?;

    let mut coverage = unavailable_vulnerability_coverage();
    for row in rows {
        let (evaluation_id, resolver_status, unresolved_reason, queue_reason) =
            row.map_err(|_| "Unable to read vulnerability coverage")?;
        coverage.total_software += 1;
        let pending = evaluation_id.is_none() || queue_reason.is_some();
        if pending {
            coverage.pending_evaluation += 1;
        }
        let invalidated = queue_reason.as_deref().is_some_and(|reason| {
            matches!(
                reason,
                "initial" | "software_installed" | "software_version_changed" | "engine_updated"
            )
        });
        if !invalidated && unresolved_reason.as_deref() == Some("component_not_mappable") {
            coverage.not_mappable += 1;
            continue;
        }
        coverage.eligible_software += 1;
        match if invalidated {
            None
        } else {
            resolver_status.as_deref()
        } {
            Some("resolved") => coverage.resolved_eligible += 1,
            Some("ambiguous") => coverage.ambiguous_eligible += 1,
            _ => coverage.unresolved_eligible += 1,
        }
    }
    coverage.status = if coverage.eligible_software == 0 {
        "not_applicable"
    } else if coverage.pending_evaluation > 0 {
        "updating"
    } else if coverage.resolved_eligible == coverage.eligible_software {
        "complete"
    } else {
        "limited"
    }
    .into();
    Ok(coverage)
}

fn load_vulnerability_risk_inputs(
    transaction: &Transaction<'_>,
) -> Result<Vec<VulnerabilityRiskInput>, String> {
    let mut statement = transaction
        .prepare(
            "WITH ranked AS (
                SELECT evaluation_id, software_id,
                       ROW_NUMBER() OVER (
                           PARTITION BY software_id
                           ORDER BY completed_at DESC, evaluation_id DESC
                       ) AS position
                FROM software_vulnerability_evaluations
             )
             SELECT i.canonical_vendor, i.canonical_product, i.display_name,
                    i.software_id, COALESCE(m.installed_version, i.display_version,
                    i.normalized_version), m.cve_id, m.match_state, m.confidence,
                    n.cvss_score, CASE WHEN k.cve_id IS NULL THEN 0 ELSE 1 END,
                    e.matching_engine_version, i.identity_resolver_version,
                    e.nvd_source_version, e.kev_source_version, e.completed_at
             FROM ranked l
             JOIN installed_software s ON s.software_id = l.software_id AND s.active = 1
             JOIN software_vulnerability_evaluations e ON e.evaluation_id = l.evaluation_id
             JOIN software_identity_evaluations i ON i.evaluation_id = l.evaluation_id
             JOIN vulnerability_matches m ON m.evaluation_id = l.evaluation_id
             JOIN nvd_vulnerabilities n ON n.cve_id = m.cve_id
             LEFT JOIN cisa_kev_vulnerabilities k ON k.cve_id = m.cve_id
             LEFT JOIN vulnerability_evaluation_queue q ON q.software_id = l.software_id
             WHERE l.position = 1
               AND i.resolver_status = 'resolved'
               AND i.canonical_vendor IS NOT NULL
               AND i.canonical_product IS NOT NULL
               AND m.match_state IN ('confirmed', 'possible')
               AND (q.reason IS NULL OR q.reason IN ('nvd_updated', 'kev_updated'))",
        )
        .map_err(|_| "Unable to prepare vulnerability score query")?;
    let rows = statement
        .query_map([], |row| {
            Ok(VulnerabilityRiskInput {
                canonical_vendor: row.get(0)?,
                canonical_product: row.get(1)?,
                display_name: row.get(2)?,
                software_id: row.get(3)?,
                installed_version: row.get(4)?,
                cve_id: row.get(5)?,
                match_state: row.get(6)?,
                confidence: row.get(7)?,
                cvss_score: row.get(8)?,
                kev: row.get::<_, i64>(9)? != 0,
                matching_engine_version: row.get(10)?,
                identity_resolver_version: row.get(11)?,
                nvd_source_version: row.get(12)?,
                kev_source_version: row.get(13)?,
                evaluated_at: row.get(14)?,
            })
        })
        .map_err(|_| "Unable to query persisted vulnerability matches")?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Unable to read persisted vulnerability matches".into())
}

fn build_product_risks(inputs: Vec<VulnerabilityRiskInput>) -> Vec<ProductVulnerabilityRisk> {
    let mut products = BTreeMap::<String, ProductRiskAccumulator>::new();
    for input in inputs {
        let product_risk_key = format!(
            "{}/{}",
            input.canonical_vendor.to_ascii_lowercase(),
            input.canonical_product.to_ascii_lowercase()
        );
        let product = products.entry(product_risk_key).or_default();
        product.canonical_vendor = input.canonical_vendor;
        product.canonical_product = input.canonical_product;
        product.display_names.insert(input.display_name);
        product.software_ids.insert(input.software_id);
        if let Some(version) = input
            .installed_version
            .filter(|value| !value.trim().is_empty())
        {
            product.installed_versions.insert(version);
        }
        product
            .matching_engine_versions
            .insert(input.matching_engine_version);
        product
            .identity_resolver_versions
            .insert(input.identity_resolver_version);
        if let Some(version) = input.nvd_source_version {
            product.nvd_source_versions.insert(version);
        }
        if let Some(version) = input.kev_source_version {
            product.kev_source_versions.insert(version);
        }
        if input.evaluated_at > product.evaluated_at {
            product.evaluated_at = input.evaluated_at;
        }
        let cve = product.cves.entry(input.cve_id).or_default();
        if input.match_state == "confirmed" && input.confidence == "high" {
            cve.confirmed_cvss = Some(preferred_cvss(
                cve.confirmed_cvss.flatten(),
                input.cvss_score,
            ));
            cve.kev |= input.kev;
            cve.possible = false;
        } else if input.match_state == "possible" && cve.confirmed_cvss.is_none() {
            cve.possible = true;
        }
    }

    let mut risks = products
        .into_iter()
        .map(|(product_risk_key, product)| product_risk(product_risk_key, product))
        .collect::<Vec<_>>();
    risks.sort_by(|left, right| {
        right
            .impact
            .cmp(&left.impact)
            .then(left.product_risk_key.cmp(&right.product_risk_key))
    });
    risks
}

fn preferred_cvss(current: Option<f64>, candidate: Option<f64>) -> Option<f64> {
    match (current, candidate) {
        (Some(current), Some(candidate)) => Some(current.max(candidate)),
        (Some(current), None) => Some(current),
        (None, candidate) => candidate,
    }
}

fn product_risk(
    product_risk_key: String,
    product: ProductRiskAccumulator,
) -> ProductVulnerabilityRisk {
    let mut confirmed = product
        .cves
        .iter()
        .filter_map(|(cve_id, value)| {
            value
                .confirmed_cvss
                .map(|cvss| (cve_id.clone(), cvss, value.kev))
        })
        .collect::<Vec<_>>();
    confirmed.sort_by(|left, right| left.0.cmp(&right.0));
    let possible_count = product
        .cves
        .values()
        .filter(|value| value.confirmed_cvss.is_none() && value.possible)
        .count() as u64;
    let possible_cve_ids = product
        .cves
        .iter()
        .filter(|(_, value)| value.confirmed_cvss.is_none() && value.possible)
        .map(|(cve_id, _)| cve_id.clone())
        .collect::<Vec<_>>();
    let mut weights = confirmed
        .iter()
        .map(|(_, cvss, _)| cvss_weight(*cvss))
        .collect::<Vec<_>>();
    weights.sort_by(|left, right| right.total_cmp(left));
    let severity_anchor = weights.first().copied().unwrap_or(0.0);
    let marginal_breadth = (weights.iter().skip(1).sum::<f64>() * 0.25).min(MARGINAL_BREADTH_CAP);
    let confirmed_kev_count = confirmed.iter().filter(|(_, _, kev)| *kev).count() as u64;
    let kev_boost = if confirmed_kev_count > 0 {
        KEV_BOOST
    } else {
        0.0
    };
    let uncapped_impact = severity_anchor + marginal_breadth + kev_boost;
    let impact = ((uncapped_impact + 0.5).floor() as u32).min(PRODUCT_IMPACT_CAP);
    let mut cvss_bands = VulnerabilityCvssBands::default();
    for (_, cvss, _) in &confirmed {
        match cvss {
            None => cvss_bands.unrated += 1,
            Some(value) if *value < 4.0 => cvss_bands.low += 1,
            Some(value) if *value < 7.0 => cvss_bands.medium += 1,
            Some(value) if *value < 9.0 => cvss_bands.high += 1,
            Some(_) => cvss_bands.critical += 1,
        }
    }
    ProductVulnerabilityRisk {
        product_risk_key,
        canonical_vendor: product.canonical_vendor,
        canonical_product: product.canonical_product,
        display_names: product.display_names.into_iter().collect(),
        software_ids: product.software_ids.into_iter().collect(),
        installed_versions: product.installed_versions.into_iter().collect(),
        confirmed_cve_ids: confirmed.iter().map(|(id, _, _)| id.clone()).collect(),
        possible_cve_ids,
        confirmed_count: confirmed.len() as u64,
        possible_count,
        cvss_bands,
        highest_cvss: confirmed
            .iter()
            .filter_map(|(_, cvss, _)| *cvss)
            .max_by(f64::total_cmp),
        confirmed_kev_count,
        severity_anchor,
        marginal_breadth,
        kev_boost,
        uncapped_impact,
        impact,
        matching_engine_versions: product.matching_engine_versions.into_iter().collect(),
        identity_resolver_versions: product.identity_resolver_versions.into_iter().collect(),
        nvd_source_versions: product.nvd_source_versions.into_iter().collect(),
        kev_source_versions: product.kev_source_versions.into_iter().collect(),
        evaluated_at: product.evaluated_at,
    }
}

fn cvss_weight(cvss: Option<f64>) -> f64 {
    match cvss {
        None => 0.5,
        Some(value) if value < 4.0 => 0.5,
        Some(value) if value < 7.0 => 1.0,
        Some(value) if value < 9.0 => 2.0,
        Some(_) => 3.0,
    }
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
    let vulnerability_coverage_json = serde_json::to_string(&score.vulnerability_coverage)
        .map_err(|_| "Unable to serialize vulnerability coverage")?;
    let product_vulnerability_risks_json =
        serde_json::to_string(&score.product_vulnerability_risks)
            .map_err(|_| "Unable to serialize product vulnerability risks")?;
    let fingerprint = stable_hash(
        &json!({
            "formulaVersion": SCORE_FORMULA_VERSION,
            "state": score.state,
            "score": score.score,
            "activeDetectionCount": score.active_detection_count,
            "highestSeverity": score.highest_severity,
            "coverage": score.coverage,
            "breakdown": score.breakdown,
            "detectionPenalty": score.detection_penalty,
            "vulnerabilityPenalty": score.vulnerability_penalty,
            "vulnerabilityCoverage": score.vulnerability_coverage,
            "productVulnerabilityRisks": score.product_vulnerability_risks,
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
        &stable_hash(&format!("score-v2|{calculated_at}|{fingerprint}"))[..32]
    );
    transaction
        .execute(
            "INSERT INTO security_score_snapshots(
                snapshot_id, calculated_at, score, availability, baseline_id,
                formula_version, active_detection_count, highest_severity,
                coverage_json, breakdown_json, reason, input_fingerprint,
                calculation_duration_ms, schema_version, detection_penalty,
                vulnerability_penalty, vulnerability_coverage_json,
                product_vulnerability_risks_json
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                       ?14, ?15, ?16, ?17, ?18)",
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
                SCORE_SCHEMA_VERSION,
                score.detection_penalty,
                score.vulnerability_penalty,
                vulnerability_coverage_json,
                product_vulnerability_risks_json
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
    use super::{
        availability, build_product_risks, cvss_weight, detection_penalty, final_score,
        grouped_breakdown, load_vulnerability_component, load_vulnerability_coverage,
        vulnerability_penalty, ActiveDetection, ScoreEngine, VulnerabilityRiskInput,
    };
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

    fn risk_input(
        product: &str,
        cve_id: &str,
        match_state: &str,
        confidence: &str,
        cvss_score: Option<f64>,
        kev: bool,
    ) -> VulnerabilityRiskInput {
        VulnerabilityRiskInput {
            canonical_vendor: "fixture-vendor".into(),
            canonical_product: product.into(),
            display_name: product.into(),
            software_id: format!("software-{product}"),
            installed_version: Some("1.0".into()),
            cve_id: cve_id.into(),
            match_state: match_state.into(),
            confidence: confidence.into(),
            cvss_score,
            kev,
            matching_engine_version: 1,
            identity_resolver_version: 1,
            nvd_source_version: Some("nvd-fixture".into()),
            kev_source_version: Some("kev-fixture".into()),
            evaluated_at: "2026-08-17T12:00:00Z".into(),
        }
    }

    fn confirmed_bands(
        product: &str,
        bands: &[(usize, f64)],
        kev_first: bool,
    ) -> Vec<VulnerabilityRiskInput> {
        let mut inputs = Vec::new();
        let mut index = 0;
        for (count, cvss) in bands {
            for _ in 0..*count {
                inputs.push(risk_input(
                    product,
                    &format!("CVE-2026-{index:04}"),
                    "confirmed",
                    "high",
                    Some(*cvss),
                    kev_first && index == 0,
                ));
                index += 1;
            }
        }
        inputs
    }

    fn seed_inventory(transaction: &rusqlite::Transaction<'_>) {
        transaction
            .execute(
                "INSERT OR IGNORE INTO software_inventory_snapshots(
                    snapshot_id, collected_at, duration_ms, raw_entry_count,
                    software_count, source_count
                 ) VALUES ('score-v2-inventory', '2026-08-17T12:00:00Z', 1, 0, 0, 1)",
                [],
            )
            .unwrap();
    }

    fn seed_software_evaluation(
        transaction: &rusqlite::Transaction<'_>,
        software_id: &str,
        evaluation_suffix: &str,
        active: bool,
        resolver_status: &str,
        unresolved_reason: Option<&str>,
        completed_at: &str,
    ) -> String {
        seed_inventory(transaction);
        transaction
            .execute(
                "INSERT OR IGNORE INTO installed_software(
                    software_id, display_name, display_version, publisher, architecture,
                    install_scope, sources_json, registry_identities_json, normalized_vendor,
                    normalized_product, normalized_version, identity_status, first_seen_at,
                    last_seen_at, active, last_snapshot_id
                 ) VALUES (?1, ?1, '1.0', 'Fixture Vendor', 'x64', 'machine', '[]', '[]',
                           'fixture-vendor', ?1, '1.0', 'resolved', '2026-08-17T12:00:00Z',
                           '2026-08-17T12:00:00Z', ?2, 'score-v2-inventory')",
                rusqlite::params![software_id, active as i64],
            )
            .unwrap();
        let evaluation_id = format!("evaluation-{software_id}-{evaluation_suffix}");
        transaction
            .execute(
                "INSERT INTO software_vulnerability_evaluations(
                    evaluation_id, software_id, software_fingerprint,
                    matching_engine_version, outcome, candidate_count, confirmed_count,
                    possible_count, unresolved_count, not_affected_count, started_at,
                    completed_at, duration_ms
                 ) VALUES (?1, ?2, ?1, 1, 'no_confirmed', 0, 0, 0, 0, 0, ?3, ?3, 1)",
                rusqlite::params![evaluation_id, software_id, completed_at],
            )
            .unwrap();
        let (canonical_vendor, canonical_product, confidence) = if resolver_status == "resolved" {
            (Some("fixture-vendor"), Some(software_id), Some("high"))
        } else {
            (None, None, None)
        };
        transaction
            .execute(
                "INSERT INTO software_identity_evaluations(
                    evaluation_id, software_id, display_name, display_version, publisher,
                    architecture, install_scope, install_source_json, registry_identity_json,
                    normalized_vendor, normalized_product, normalized_version, identity_status,
                    evidence_json, identity_resolver_version, resolver_status, canonical_vendor,
                    canonical_product, resolver_confidence, unresolved_reason, provenance_json
                 ) VALUES (?1, ?2, ?2, '1.0', 'Fixture Vendor', 'x64', 'machine', '[]', '[]',
                           'fixture-vendor', ?2, '1.0', 'strong', '{}', 1, ?3, ?4, ?5, ?6, ?7, '[]')",
                rusqlite::params![
                    evaluation_id,
                    software_id,
                    resolver_status,
                    canonical_vendor,
                    canonical_product,
                    confidence,
                    unresolved_reason
                ],
            )
            .unwrap();
        evaluation_id
    }

    fn seed_match(
        transaction: &rusqlite::Transaction<'_>,
        evaluation_id: &str,
        software_id: &str,
        cve_id: &str,
        match_state: &str,
        confidence: &str,
        cvss: f64,
    ) {
        transaction
            .execute(
                "INSERT OR IGNORE INTO nvd_vulnerabilities(
                    cve_id, published_at, last_modified_at, description, cvss_score,
                    weaknesses_json, references_json, applicability_json, repository_updated_at
                 ) VALUES (?1, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z',
                           'Fixture', ?2, '[]', '[]', '[]', '2026-08-17T12:00:00Z')",
                rusqlite::params![cve_id, cvss],
            )
            .unwrap();
        transaction
            .execute(
                "INSERT INTO vulnerability_matches(
                    match_id, evaluation_id, software_id, candidate_id, cve_id,
                    match_state, confidence, installed_version, affected_range,
                    comparison_result, matching_engine_version, last_evaluated_at
                 ) VALUES (?1, ?2, ?3, NULL, ?4, ?5, ?6, '1.0', '= 1.0',
                           'fixture', 1, '2026-08-17T12:00:00Z')",
                rusqlite::params![
                    format!("match-{evaluation_id}-{cve_id}"),
                    evaluation_id,
                    software_id,
                    cve_id,
                    match_state,
                    confidence
                ],
            )
            .unwrap();
    }

    #[test]
    fn cvss_boundaries_have_the_approved_weights() {
        let cases = [
            (None, 0.5),
            (Some(0.0), 0.5),
            (Some(3.9), 0.5),
            (Some(4.0), 1.0),
            (Some(6.9), 1.0),
            (Some(7.0), 2.0),
            (Some(8.9), 2.0),
            (Some(9.0), 3.0),
            (Some(10.0), 3.0),
        ];
        for (cvss, expected) in cases {
            assert_eq!(cvss_weight(cvss), expected);
        }
    }

    #[test]
    fn final_score_combines_detection_and_vulnerability_components_once() {
        assert_eq!(final_score(8, 14), 78);
        assert_eq!(final_score(0, 0), 100);
        assert_eq!(final_score(90, 18), 0);
    }

    #[test]
    fn real_calibration_is_jre_6_virtualbox_4_python_4_and_score_86() {
        let mut inputs = confirmed_bands("jre", &[(1, 8.0), (5, 3.0)], true);
        inputs.extend(confirmed_bands(
            "virtualbox",
            &[(5, 8.0), (7, 5.0), (3, 3.0)],
            false,
        ));
        inputs.extend(confirmed_bands(
            "python",
            &[(2, 8.0), (5, 5.0), (3, 3.0)],
            false,
        ));
        inputs.push(risk_input(
            "python",
            "CVE-2026-9999",
            "possible",
            "medium",
            Some(6.5),
            false,
        ));
        let risks = build_product_risks(inputs);
        let impacts = risks
            .iter()
            .map(|risk| (risk.canonical_product.as_str(), risk.impact))
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(impacts.get("jre"), Some(&6));
        assert_eq!(impacts.get("virtualbox"), Some(&4));
        assert_eq!(impacts.get("python"), Some(&4));
        assert_eq!(
            risks
                .iter()
                .find(|risk| risk.canonical_product == "python")
                .unwrap()
                .possible_count,
            1
        );
        assert_eq!(vulnerability_penalty(&risks), 14);
        assert_eq!(100 - vulnerability_penalty(&risks), 86);
    }

    #[test]
    fn possible_low_confidence_and_unresolved_inputs_never_reduce_score() {
        let inputs = vec![
            risk_input(
                "possible",
                "CVE-2026-0001",
                "possible",
                "high",
                Some(10.0),
                true,
            ),
            risk_input(
                "low-confidence",
                "CVE-2026-0002",
                "confirmed",
                "medium",
                Some(10.0),
                true,
            ),
            risk_input(
                "unresolved",
                "CVE-2026-0003",
                "unresolved",
                "high",
                Some(10.0),
                true,
            ),
        ];
        assert_eq!(vulnerability_penalty(&build_product_risks(inputs)), 0);
    }

    #[test]
    fn duplicate_cves_are_counted_once_and_kev_requires_confirmed_high_confidence() {
        let duplicate = risk_input(
            "product",
            "CVE-2026-0001",
            "confirmed",
            "high",
            Some(8.0),
            false,
        );
        let mut duplicate_second = risk_input(
            "product",
            "CVE-2026-0001",
            "confirmed",
            "high",
            Some(8.0),
            false,
        );
        duplicate_second.software_id = "duplicate-registry-record".into();
        let possible_kev = risk_input(
            "product",
            "CVE-2026-0002",
            "possible",
            "high",
            Some(10.0),
            true,
        );
        let risks = build_product_risks(vec![duplicate, duplicate_second, possible_kev]);
        assert_eq!(risks[0].confirmed_count, 1);
        assert_eq!(risks[0].possible_count, 1);
        assert_eq!(risks[0].confirmed_kev_count, 0);
        assert_eq!(risks[0].impact, 2);
    }

    #[test]
    fn diminishing_returns_respect_product_and_category_caps() {
        let mut inputs = confirmed_bands("crowded", &[(100, 8.0)], true);
        for product in ["second", "third", "fourth"] {
            inputs.extend(confirmed_bands(product, &[(1, 10.0)], true));
        }
        let risks = build_product_risks(inputs);
        assert_eq!(
            risks
                .iter()
                .find(|risk| risk.canonical_product == "crowded")
                .unwrap()
                .impact,
            7
        );
        assert_eq!(vulnerability_penalty(&risks), 18);
    }

    #[test]
    fn coverage_separates_not_mappable_and_reports_pending_without_penalty() {
        let database = Database::in_memory().expect("database");
        database
            .analysis_transaction(|transaction| {
                seed_software_evaluation(
                    transaction,
                    "resolved",
                    "one",
                    true,
                    "resolved",
                    None,
                    "2026-08-17T12:00:00Z",
                );
                seed_software_evaluation(
                    transaction,
                    "ambiguous",
                    "one",
                    true,
                    "ambiguous",
                    Some("multiple_candidates"),
                    "2026-08-17T12:00:00Z",
                );
                seed_software_evaluation(
                    transaction,
                    "unresolved",
                    "one",
                    true,
                    "unresolved",
                    Some("missing_version"),
                    "2026-08-17T12:00:00Z",
                );
                seed_software_evaluation(
                    transaction,
                    "not-mappable",
                    "one",
                    true,
                    "unresolved",
                    Some("component_not_mappable"),
                    "2026-08-17T12:00:00Z",
                );
                transaction
                    .execute(
                        "INSERT INTO installed_software(
                            software_id, display_name, display_version, publisher, architecture,
                            install_scope, sources_json, registry_identities_json,
                            normalized_vendor, normalized_product, normalized_version,
                            identity_status, first_seen_at, last_seen_at, active, last_snapshot_id
                         ) VALUES ('pending', 'Pending', '1.0', 'Fixture', 'x64', 'machine',
                                   '[]', '[]', '', '', '', 'unresolved',
                                   '2026-08-17T12:00:00Z', '2026-08-17T12:00:00Z', 1,
                                   'score-v2-inventory')",
                        [],
                    )
                    .unwrap();
                transaction
                    .execute(
                        "INSERT INTO vulnerability_evaluation_queue(software_id, reason, requested_at)
                         VALUES ('pending', 'initial', '2026-08-17T12:00:00Z')",
                        [],
                    )
                    .unwrap();
                let coverage = load_vulnerability_coverage(transaction)?;
                assert_eq!(coverage.total_software, 5);
                assert_eq!(coverage.eligible_software, 4);
                assert_eq!(coverage.resolved_eligible, 1);
                assert_eq!(coverage.ambiguous_eligible, 1);
                assert_eq!(coverage.unresolved_eligible, 2);
                assert_eq!(coverage.not_mappable, 1);
                assert_eq!(coverage.pending_evaluation, 1);
                assert_eq!(coverage.status, "updating");
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn persisted_matches_react_to_removal_version_queue_not_affected_provider_and_kev() {
        let database = Database::in_memory().expect("database");
        database
            .analysis_transaction(|transaction| {
                let kept = seed_software_evaluation(
                    transaction,
                    "kept",
                    "one",
                    true,
                    "resolved",
                    None,
                    "2026-08-17T12:00:00Z",
                );
                seed_match(
                    transaction,
                    &kept,
                    "kept",
                    "CVE-2026-1001",
                    "confirmed",
                    "high",
                    8.0,
                );
                let removed = seed_software_evaluation(
                    transaction,
                    "removed",
                    "one",
                    false,
                    "resolved",
                    None,
                    "2026-08-17T12:00:00Z",
                );
                seed_match(
                    transaction,
                    &removed,
                    "removed",
                    "CVE-2026-1002",
                    "confirmed",
                    "high",
                    10.0,
                );
                let updating = seed_software_evaluation(
                    transaction,
                    "updating",
                    "one",
                    true,
                    "resolved",
                    None,
                    "2026-08-17T12:00:00Z",
                );
                seed_match(
                    transaction,
                    &updating,
                    "updating",
                    "CVE-2026-1003",
                    "confirmed",
                    "high",
                    10.0,
                );
                transaction
                    .execute(
                        "INSERT INTO vulnerability_evaluation_queue(software_id, reason, requested_at)
                         VALUES ('updating', 'software_version_changed', '2026-08-17T12:01:00Z')",
                        [],
                    )
                    .unwrap();
                let provider = seed_software_evaluation(
                    transaction,
                    "provider-refresh",
                    "one",
                    true,
                    "resolved",
                    None,
                    "2026-08-17T12:00:00Z",
                );
                seed_match(
                    transaction,
                    &provider,
                    "provider-refresh",
                    "CVE-2026-1004",
                    "confirmed",
                    "high",
                    8.0,
                );
                transaction
                    .execute(
                        "INSERT INTO vulnerability_evaluation_queue(software_id, reason, requested_at)
                         VALUES ('provider-refresh', 'nvd_updated', '2026-08-17T12:01:00Z')",
                        [],
                    )
                    .unwrap();
                let old = seed_software_evaluation(
                    transaction,
                    "not-affected",
                    "old",
                    true,
                    "resolved",
                    None,
                    "2026-08-17T11:00:00Z",
                );
                seed_match(
                    transaction,
                    &old,
                    "not-affected",
                    "CVE-2026-1005",
                    "confirmed",
                    "high",
                    10.0,
                );
                let latest = seed_software_evaluation(
                    transaction,
                    "not-affected",
                    "new",
                    true,
                    "resolved",
                    None,
                    "2026-08-17T13:00:00Z",
                );
                seed_match(
                    transaction,
                    &latest,
                    "not-affected",
                    "CVE-2026-1005",
                    "not_affected",
                    "high",
                    10.0,
                );

                let (_, risks, penalty) = load_vulnerability_component(transaction)?;
                assert_eq!(penalty, 4);
                assert_eq!(risks.len(), 2);
                assert!(risks.iter().any(|risk| risk.canonical_product == "kept"));
                assert!(risks
                    .iter()
                    .any(|risk| risk.canonical_product == "provider-refresh"));
                assert!(!risks.iter().any(|risk| risk.canonical_product == "removed"));
                assert!(!risks.iter().any(|risk| risk.canonical_product == "updating"));
                assert!(!risks
                    .iter()
                    .any(|risk| risk.canonical_product == "not-affected"));

                transaction
                    .execute(
                        "INSERT INTO cisa_kev_vulnerabilities(
                            cve_id, vendor_project, product, vulnerability_name, date_added,
                            required_action, cwes_json, repository_updated_at
                         ) VALUES ('CVE-2026-1001', 'Fixture', 'Kept', 'Fixture KEV',
                                   '2026-08-17', 'Update', '[]', '2026-08-17T12:00:00Z')",
                        [],
                    )
                    .unwrap();
                let (_, risks_with_kev, penalty_with_kev) =
                    load_vulnerability_component(transaction)?;
                assert_eq!(penalty_with_kev, 7);
                assert_eq!(
                    risks_with_kev
                        .iter()
                        .find(|risk| risk.canonical_product == "kept")
                        .unwrap()
                        .confirmed_kev_count,
                    1
                );
                Ok(())
            })
            .unwrap();
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
    fn updating_vulnerability_coverage_qualifies_the_global_score_as_limited() {
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
                         ) VALUES ('updating-baseline', '2026-08-17T10:00:00Z',
                                   '2026-08-17T10:00:00Z', 1, 'updating-host', 'ready',
                                   86400, '2026-08-17T10:00:00Z', 1)",
                        [],
                    )
                    .unwrap();
                seed_inventory(transaction);
                transaction
                    .execute(
                        "INSERT INTO installed_software(
                            software_id, display_name, display_version, publisher, architecture,
                            install_scope, sources_json, registry_identities_json,
                            normalized_vendor, normalized_product, normalized_version,
                            identity_status, first_seen_at, last_seen_at, active, last_snapshot_id
                         ) VALUES ('pending-score', 'Pending Score', '1.0', 'Fixture', 'x64',
                                   'machine', '[]', '[]', '', '', '', 'unresolved',
                                   '2026-08-17T12:00:00Z', '2026-08-17T12:00:00Z', 1,
                                   'score-v2-inventory')",
                        [],
                    )
                    .unwrap();
                transaction
                    .execute(
                        "INSERT INTO vulnerability_evaluation_queue(software_id, reason, requested_at)
                         VALUES ('pending-score', 'initial', '2026-08-17T12:00:00Z')",
                        [],
                    )
                    .unwrap();
                Ok(())
            })
            .unwrap();
        let score = ScoreEngine
            .calculate_and_persist(&database, healthy_coverage())
            .expect("limited score");
        assert_eq!(score.state, ScoreState::Limited);
        assert_eq!(score.score, Some(100));
        assert_eq!(score.vulnerability_penalty, 0);
        assert_eq!(score.vulnerability_coverage.status, "updating");
        assert_eq!(
            score.reason.as_deref(),
            Some("Limited coverage — Vulnerability Intelligence is updating")
        );
        database
            .analysis_transaction(|transaction| {
                transaction
                    .execute(
                        "DELETE FROM vulnerability_evaluation_queue
                         WHERE software_id = 'pending-score'",
                        [],
                    )
                    .unwrap();
                seed_software_evaluation(
                    transaction,
                    "pending-score",
                    "completed",
                    true,
                    "resolved",
                    None,
                    "2026-08-17T13:00:00Z",
                );
                Ok(())
            })
            .unwrap();
        let recovered = ScoreEngine
            .calculate_and_persist(&database, Vec::new())
            .expect("recovered score");
        assert_eq!(recovered.state, ScoreState::Available);
        assert_eq!(recovered.score, Some(100));
        assert_eq!(recovered.vulnerability_coverage.status, "complete");
        assert_eq!(recovered.vulnerability_coverage.pending_evaluation, 0);
    }

    #[test]
    fn historical_formula_v1_snapshot_remains_readable_and_immutable() {
        let database = Database::in_memory().expect("database");
        database
            .analysis_transaction(|transaction| {
                transaction
                    .execute(
                        "INSERT INTO security_score_snapshots(
                            snapshot_id, calculated_at, score, availability, baseline_id,
                            formula_version, active_detection_count, highest_severity,
                            coverage_json, breakdown_json, reason, input_fingerprint,
                            calculation_duration_ms, schema_version
                         ) VALUES ('historical-v1', '2026-08-16T10:00:00Z', 100,
                                   'available', NULL, 1, 0, NULL, '[]', '[]', NULL,
                                   ?1, 1, 1)",
                        ["0".repeat(64)],
                    )
                    .map_err(|_| "historical score fixture")?;
                Ok(())
            })
            .unwrap();
        let historical = ScoreEngine.current(&database).expect("historical score");
        assert_eq!(historical.formula_version, 1);
        assert_eq!(historical.score, Some(100));
        assert_eq!(historical.detection_penalty, 0);
        assert_eq!(historical.vulnerability_penalty, 0);
        assert_eq!(historical.vulnerability_coverage.status, "unavailable");
        assert_eq!(
            historical.vulnerability_coverage.basis,
            "software_record_proxy"
        );
        assert!(historical.product_vulnerability_risks.is_empty());
        let update = database.analysis_transaction(|transaction| {
            transaction
                .execute(
                    "UPDATE security_score_snapshots SET score = 99
                     WHERE snapshot_id = 'historical-v1'",
                    [],
                )
                .map(|_| ())
                .map_err(|error| error.to_string())
        });
        assert!(update.is_err());
    }

    #[test]
    fn score_history_distinguishes_formula_v1_and_v2_snapshots() {
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
                         ) VALUES ('history-baseline', '2026-08-17T10:00:00Z',
                                   '2026-08-17T10:00:00Z', 1, 'history-host', 'ready',
                                   86400, '2026-08-17T10:00:00Z', 1)",
                        [],
                    )
                    .unwrap();
                transaction
                    .execute(
                        "INSERT INTO security_score_snapshots(
                            snapshot_id, calculated_at, score, availability, baseline_id,
                            formula_version, active_detection_count, highest_severity,
                            coverage_json, breakdown_json, reason, input_fingerprint,
                            calculation_duration_ms, schema_version
                         ) VALUES ('history-v1', '2026-08-17T10:01:00Z', 100,
                                   'available', 'history-baseline', 1, 0, NULL, '[]', '[]',
                                   NULL, ?1, 1, 1)",
                        ["1".repeat(64)],
                    )
                    .unwrap();
                Ok(())
            })
            .unwrap();
        let v2 = ScoreEngine
            .calculate_and_persist(&database, healthy_coverage())
            .expect("v2 snapshot");
        assert_eq!(v2.formula_version, 2);
        let history = database
            .analysis_transaction(|transaction| {
                let mut statement = transaction
                    .prepare(
                        "SELECT snapshot_id, formula_version, score
                         FROM security_score_snapshots ORDER BY calculated_at",
                    )
                    .map_err(|error| error.to_string())?;
                let rows = statement
                    .query_map([], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, u32>(1)?,
                            row.get::<_, Option<u32>>(2)?,
                        ))
                    })
                    .map_err(|error| error.to_string())?
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|error| error.to_string())?;
                Ok(rows)
            })
            .unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0], ("history-v1".into(), 1, Some(100)));
        assert_eq!(history[1].1, 2);
        assert_eq!(history[1].2, Some(100));
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
        let current = engine.current(&database).unwrap();
        assert_eq!(current.formula_version, 2);
        assert_eq!(current.detection_penalty, 0);
        assert_eq!(current.vulnerability_penalty, 0);
    }

    #[test]
    #[ignore = "manual calibration and performance probe against the native Windows database"]
    fn native_windows_vulnerability_score_v2_calibration_and_benchmark() {
        use std::{path::PathBuf, time::Instant};

        let app_data = std::env::var_os("APPDATA").expect("APPDATA is available");
        let database = Database::open(
            PathBuf::from(app_data)
                .join("com.edy.sentinel")
                .join("sentinel.db"),
        )
        .expect("application database");
        let query_started = Instant::now();
        let inputs = database
            .analysis_transaction(super::load_vulnerability_risk_inputs)
            .expect("persisted vulnerability inputs");
        let query_ms = query_started.elapsed().as_secs_f64() * 1_000.0;
        let calculation_started = Instant::now();
        let risks = build_product_risks(inputs);
        let penalty = vulnerability_penalty(&risks);
        let calculation_ms = calculation_started.elapsed().as_secs_f64() * 1_000.0;

        let impacts = risks
            .iter()
            .map(|risk| (risk.canonical_product.as_str(), risk.impact))
            .collect::<std::collections::BTreeMap<_, _>>();
        println!("native calibrated product impacts: {impacts:?}");
        assert_eq!(impacts.get("jre"), Some(&6));
        assert_eq!(impacts.get("vm_virtualbox"), Some(&4));
        assert_eq!(impacts.get("python"), Some(&4));
        assert_eq!(
            risks.iter().map(|risk| risk.confirmed_count).sum::<u64>(),
            31
        );
        assert_eq!(risks.iter().map(|risk| risk.possible_count).sum::<u64>(), 1);
        assert_eq!(
            risks
                .iter()
                .map(|risk| risk.confirmed_kev_count)
                .sum::<u64>(),
            1
        );
        assert_eq!(penalty, 14);

        let persistence_started = Instant::now();
        let score = ScoreEngine
            .calculate_and_persist(&database, Vec::new())
            .expect("formula v2 score snapshot");
        let end_to_end_ms = persistence_started.elapsed().as_secs_f64() * 1_000.0;
        assert_eq!(score.formula_version, 2);
        assert_eq!(score.vulnerability_penalty, 14);
        if score.detection_penalty == 0 {
            assert_eq!(score.score, Some(86));
        }
        println!(
            "vulnerability score v2 benchmark: query_ms={query_ms:.3}, calculation_ms={calculation_ms:.3}, end_to_end_persistence_ms={end_to_end_ms:.3}"
        );
    }
}
