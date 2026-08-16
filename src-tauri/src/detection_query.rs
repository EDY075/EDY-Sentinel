use crate::{
    models::{
        DetectionConfidence, DetectionEvidenceRecord, DetectionExplanation, DetectionRecord,
        DetectionSeverity, DetectionStatus,
    },
    persistence::Database,
};
use chrono::DateTime;
use rusqlite::{params_from_iter, types::Value as SqlValue, Row};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_PAGE_SIZE: u32 = 50;
const MAX_PAGE_SIZE: u32 = 100;
const MAX_FILTER_VALUES: usize = 20;
const MAX_TEXT_LENGTH: usize = 512;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DetectionCursor {
    pub last_detected_at: String,
    pub detection_id: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DetectionQueryInput {
    pub statuses: Option<Vec<DetectionStatus>>,
    pub severities: Option<Vec<DetectionSeverity>>,
    pub categories: Option<Vec<String>>,
    pub rule_ids: Option<Vec<String>>,
    pub entity_type: Option<String>,
    pub entity_key: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub cursor: Option<DetectionCursor>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectionPage {
    pub items: Vec<DetectionRecord>,
    pub next_cursor: Option<DetectionCursor>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DetectionEvidenceCursor {
    pub observed_at: String,
    pub evidence_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectionEvidenceInput {
    pub detection_id: String,
    pub cursor: Option<DetectionEvidenceCursor>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectionEvidencePage {
    pub items: Vec<DetectionEvidenceRecord>,
    pub next_cursor: Option<DetectionEvidenceCursor>,
    pub has_more: bool,
}

pub(crate) fn query_detections(
    database: &Database,
    input: DetectionQueryInput,
) -> Result<DetectionPage, String> {
    let limit = validate_limit(input.limit)?;
    validate_enum_filters(input.statuses.as_deref(), "status")?;
    validate_enum_filters(input.severities.as_deref(), "severity")?;
    validate_text_filters(input.categories.as_deref(), "category")?;
    validate_text_filters(input.rule_ids.as_deref(), "rule")?;
    validate_optional_text(input.entity_type.as_deref(), "entity type")?;
    validate_optional_text(input.entity_key.as_deref(), "entity key")?;
    validate_period(input.from.as_deref(), input.to.as_deref())?;
    if let Some(cursor) = input.cursor.as_ref() {
        validate_timestamp(&cursor.last_detected_at, "detection cursor")?;
        validate_text(&cursor.detection_id, "detection cursor ID")?;
    }

    database.analysis_read(|connection| {
        let mut clauses = Vec::new();
        let mut values = Vec::<SqlValue>::new();
        if let Some(statuses) = input.statuses.as_ref() {
            clauses.push(format!(
                "detection.status IN ({})",
                placeholders(statuses.len())
            ));
            values.extend(
                statuses
                    .iter()
                    .map(|status| SqlValue::Text(status.as_str().into())),
            );
        }
        if let Some(severities) = input.severities.as_ref() {
            clauses.push(format!(
                "detection.severity IN ({})",
                placeholders(severities.len())
            ));
            values.extend(
                severities
                    .iter()
                    .map(|severity| SqlValue::Text(severity.as_str().into())),
            );
        }
        if let Some(categories) = input.categories.as_ref() {
            clauses.push(format!(
                "rule.category IN ({})",
                placeholders(categories.len())
            ));
            values.extend(categories.iter().cloned().map(SqlValue::Text));
        }
        if let Some(rule_ids) = input.rule_ids.as_ref() {
            clauses.push(format!(
                "detection.rule_id IN ({})",
                placeholders(rule_ids.len())
            ));
            values.extend(rule_ids.iter().cloned().map(SqlValue::Text));
        }
        add_equal_filter(
            &mut clauses,
            &mut values,
            "detection.entity_type",
            input.entity_type,
        );
        add_equal_filter(
            &mut clauses,
            &mut values,
            "detection.entity_key",
            input.entity_key,
        );
        add_range_filters(
            &mut clauses,
            &mut values,
            "detection.last_detected_at",
            input.from,
            input.to,
        );
        if let Some(cursor) = input.cursor {
            clauses.push(
                "(detection.last_detected_at < ? OR
                  (detection.last_detected_at = ? AND detection.detection_id < ?))"
                    .into(),
            );
            values.push(SqlValue::Text(cursor.last_detected_at.clone()));
            values.push(SqlValue::Text(cursor.last_detected_at));
            values.push(SqlValue::Text(cursor.detection_id));
        }

        let mut sql = String::from(
            "SELECT detection.detection_id, detection.rule_id, detection.rule_version,
                    detection.entity_type, detection.entity_key, detection.title,
                    detection.summary, detection.severity, detection.confidence,
                    detection.status, detection.first_detected_at,
                    detection.last_detected_at, detection.occurrence_count,
                    detection.baseline_id, detection.explanation_json,
                    detection.remediation_guidance_json, detection.condition_active,
                    detection.schema_version
             FROM detections AS detection
             JOIN detection_rule_versions AS rule
               ON rule.rule_id = detection.rule_id
              AND rule.rule_version = detection.rule_version",
        );
        append_where(&mut sql, &clauses);
        sql.push_str(
            " ORDER BY detection.last_detected_at DESC, detection.detection_id DESC LIMIT ?",
        );
        values.push(SqlValue::Integer(i64::from(limit + 1)));

        let mut statement = connection
            .prepare(&sql)
            .map_err(|_| "Unable to prepare paginated detection query")?;
        let rows = statement
            .query_map(params_from_iter(values.iter()), map_detection)
            .map_err(|_| "Unable to query detections")?;
        let mut items = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "Unable to read detections")?;
        let has_more = items.len() > limit as usize;
        items.truncate(limit as usize);
        let next_cursor = has_more
            .then(|| {
                items.last().map(|item| DetectionCursor {
                    last_detected_at: item.last_detected_at.clone(),
                    detection_id: item.detection_id.clone(),
                })
            })
            .flatten();
        Ok(DetectionPage {
            items,
            next_cursor,
            has_more,
        })
    })
}

pub(crate) fn query_detection_evidence(
    database: &Database,
    input: DetectionEvidenceInput,
) -> Result<DetectionEvidencePage, String> {
    validate_text(&input.detection_id, "detection ID")?;
    let limit = validate_limit(input.limit)?;
    if let Some(cursor) = input.cursor.as_ref() {
        validate_timestamp(&cursor.observed_at, "evidence cursor")?;
        validate_text(&cursor.evidence_id, "evidence cursor ID")?;
    }
    database.analysis_read(|connection| {
        let mut values = vec![SqlValue::Text(input.detection_id)];
        let mut cursor_clause = String::new();
        if let Some(cursor) = input.cursor {
            cursor_clause.push_str(
                " AND (evidence.captured_at < ? OR
                       (evidence.captured_at = ? AND evidence.evidence_id < ?))",
            );
            values.push(SqlValue::Text(cursor.observed_at.clone()));
            values.push(SqlValue::Text(cursor.observed_at));
            values.push(SqlValue::Text(cursor.evidence_id));
        }
        let sql = format!(
            "SELECT evidence.evidence_id, evidence.source_event_id,
                    evidence.evidence_role, evidence.evidence_json,
                    evidence.captured_at, evidence.collector_source,
                    event.event_type, event.entity_type, event.entity_key,
                    evidence.source_event_schema_version
             FROM detection_evidence AS evidence
             JOIN security_events AS event ON event.id = evidence.source_event_id
             WHERE evidence.detection_id = ?{cursor_clause}
             ORDER BY evidence.captured_at DESC, evidence.evidence_id DESC LIMIT ?"
        );
        values.push(SqlValue::Integer(i64::from(limit + 1)));
        let mut statement = connection
            .prepare(&sql)
            .map_err(|_| "Unable to prepare detection evidence query")?;
        let rows = statement
            .query_map(params_from_iter(values.iter()), map_evidence)
            .map_err(|_| "Unable to query detection evidence")?;
        let mut items = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "Unable to read detection evidence")?;
        let has_more = items.len() > limit as usize;
        items.truncate(limit as usize);
        let next_cursor = has_more
            .then(|| {
                items.last().map(|item| DetectionEvidenceCursor {
                    observed_at: item.observed_at.clone(),
                    evidence_id: item.evidence_id.clone(),
                })
            })
            .flatten();
        Ok(DetectionEvidencePage {
            items,
            next_cursor,
            has_more,
        })
    })
}

fn map_detection(row: &Row<'_>) -> rusqlite::Result<DetectionRecord> {
    let severity: String = row.get(7)?;
    let confidence: String = row.get(8)?;
    let status: String = row.get(9)?;
    let explanation_json: String = row.get(14)?;
    let remediation_json: String = row.get(15)?;
    Ok(DetectionRecord {
        detection_id: row.get(0)?,
        rule_id: row.get(1)?,
        rule_version: row.get(2)?,
        entity_type: row.get(3)?,
        entity_key: row.get(4)?,
        title: row.get(5)?,
        summary: row.get(6)?,
        severity: DetectionSeverity::from_persisted(&severity).map_err(invalid_data)?,
        confidence: DetectionConfidence::from_persisted(&confidence).map_err(invalid_data)?,
        status: DetectionStatus::from_persisted(&status).map_err(invalid_data)?,
        first_detected_at: row.get(10)?,
        last_detected_at: row.get(11)?,
        occurrence_count: row.get(12)?,
        baseline_id: row.get(13)?,
        explanation: serde_json::from_str::<DetectionExplanation>(&explanation_json)
            .map_err(|error| invalid_data(error.to_string()))?,
        remediation_guidance: serde_json::from_str(&remediation_json)
            .map_err(|error| invalid_data(error.to_string()))?,
        condition_active: row.get::<_, i64>(16)? != 0,
        schema_version: row.get(17)?,
    })
}

fn map_evidence(row: &Row<'_>) -> rusqlite::Result<DetectionEvidenceRecord> {
    let payload: String = row.get(3)?;
    let payload: Value =
        serde_json::from_str(&payload).map_err(|error| invalid_data(error.to_string()))?;
    Ok(DetectionEvidenceRecord {
        evidence_id: row.get(0)?,
        event_id: row.get(1)?,
        evidence_type: row.get(2)?,
        label: payload
            .get("label")
            .and_then(Value::as_str)
            .unwrap_or("Evidence")
            .into(),
        value: payload.get("value").cloned().unwrap_or(Value::Null),
        observed_at: row.get(4)?,
        source: row.get(5)?,
        event_type: row.get(6)?,
        entity_type: row.get(7)?,
        entity_key: row.get(8)?,
        event_schema_version: row.get(9)?,
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

fn validate_limit(limit: Option<u32>) -> Result<u32, String> {
    match limit.unwrap_or(DEFAULT_PAGE_SIZE) {
        0 => Err("Page size must be greater than zero".into()),
        value if value > MAX_PAGE_SIZE => Err(format!("Page size cannot exceed {MAX_PAGE_SIZE}")),
        value => Ok(value),
    }
}

fn validate_enum_filters<T>(values: Option<&[T]>, field: &str) -> Result<(), String> {
    if let Some(values) = values {
        if values.is_empty() || values.len() > MAX_FILTER_VALUES {
            return Err(format!("{field} filters are invalid"));
        }
    }
    Ok(())
}

fn validate_text_filters(values: Option<&[String]>, field: &str) -> Result<(), String> {
    validate_enum_filters(values, field)?;
    if let Some(values) = values {
        for value in values {
            validate_text(value, field)?;
        }
    }
    Ok(())
}

fn validate_optional_text(value: Option<&str>, field: &str) -> Result<(), String> {
    value.map(|value| validate_text(value, field)).transpose()?;
    Ok(())
}

fn validate_text(value: &str, field: &str) -> Result<(), String> {
    if value.trim().is_empty() || value.len() > MAX_TEXT_LENGTH {
        Err(format!("{field} is invalid"))
    } else {
        Ok(())
    }
}

fn validate_period(from: Option<&str>, to: Option<&str>) -> Result<(), String> {
    let from = from
        .map(|value| validate_timestamp(value, "period start"))
        .transpose()?;
    let to = to
        .map(|value| validate_timestamp(value, "period end"))
        .transpose()?;
    if from.zip(to).is_some_and(|(from, to)| from > to) {
        return Err("Period start must not be after period end".into());
    }
    Ok(())
}

fn validate_timestamp(value: &str, field: &str) -> Result<DateTime<chrono::FixedOffset>, String> {
    DateTime::parse_from_rfc3339(value)
        .map_err(|_| format!("{field} must be an RFC 3339 timestamp"))
}

fn placeholders(count: usize) -> String {
    vec!["?"; count].join(",")
}

fn add_equal_filter(
    clauses: &mut Vec<String>,
    values: &mut Vec<SqlValue>,
    column: &'static str,
    value: Option<String>,
) {
    if let Some(value) = value {
        clauses.push(format!("{column} = ?"));
        values.push(SqlValue::Text(value));
    }
}

fn add_range_filters(
    clauses: &mut Vec<String>,
    values: &mut Vec<SqlValue>,
    column: &'static str,
    from: Option<String>,
    to: Option<String>,
) {
    if let Some(from) = from {
        clauses.push(format!("{column} >= ?"));
        values.push(SqlValue::Text(from));
    }
    if let Some(to) = to {
        clauses.push(format!("{column} <= ?"));
        values.push(SqlValue::Text(to));
    }
}

fn append_where(sql: &mut String, clauses: &[String]) {
    if !clauses.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&clauses.join(" AND "));
    }
}

#[cfg(test)]
mod tests {
    use super::{validate_limit, validate_period};

    #[test]
    fn detection_query_limits_and_periods_are_bounded() {
        assert_eq!(validate_limit(None).unwrap(), 50);
        assert_eq!(validate_limit(Some(100)).unwrap(), 100);
        assert!(validate_limit(Some(101)).is_err());
        assert!(
            validate_period(Some("2026-08-16T10:00:00Z"), Some("2026-08-16T11:00:00Z")).is_ok()
        );
        assert!(
            validate_period(Some("2026-08-16T12:00:00Z"), Some("2026-08-16T11:00:00Z")).is_err()
        );
    }
}
