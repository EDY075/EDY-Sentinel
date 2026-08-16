use crate::{
    models::{SecurityEventRecord, SecurityEventStatus},
    persistence::Database,
};
use chrono::DateTime;
use rusqlite::{params_from_iter, types::Value as SqlValue, Row};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_PAGE_SIZE: u32 = 50;
const MAX_PAGE_SIZE: u32 = 100;
const MAX_FILTER_VALUES: usize = 20;
const MAX_FILTER_LENGTH: usize = 512;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecurityEventCursor {
    pub last_seen_at: String,
    pub event_id: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SecurityEventQueryInput {
    pub statuses: Option<Vec<SecurityEventStatus>>,
    pub event_types: Option<Vec<String>>,
    pub entity_type: Option<String>,
    pub entity_key: Option<String>,
    pub baseline_id: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub cursor: Option<SecurityEventCursor>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityEventPage {
    pub items: Vec<SecurityEventRecord>,
    pub next_cursor: Option<SecurityEventCursor>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecurityEventHistoryCursor {
    pub observed_at: String,
    pub history_id: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityEventHistoryInput {
    pub entity_type: String,
    pub entity_key: String,
    pub event_type: Option<String>,
    pub baseline_id: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub cursor: Option<SecurityEventHistoryCursor>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityEventHistoryRecord {
    pub history_id: i64,
    pub event_id: String,
    pub transition: String,
    pub observed_at: String,
    pub recorded_at: String,
    pub source: String,
    pub event_type: String,
    pub entity_type: String,
    pub entity_key: String,
    pub baseline_id: Option<String>,
    pub rule_id: Option<String>,
    pub rule_version: Option<u32>,
    pub evidence: Value,
    pub baseline_context: Value,
    pub previous_status: Option<String>,
    pub new_status: Option<String>,
    pub event_schema_version: u32,
    pub history_schema_version: u32,
    pub observation_count: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityEventHistoryPage {
    pub items: Vec<SecurityEventHistoryRecord>,
    pub next_cursor: Option<SecurityEventHistoryCursor>,
    pub has_more: bool,
}

pub(crate) fn query_security_events(
    database: &Database,
    input: SecurityEventQueryInput,
) -> Result<SecurityEventPage, String> {
    let limit = validate_limit(input.limit)?;
    validate_optional_values(input.event_types.as_deref(), "event type")?;
    validate_optional_text(input.entity_type.as_deref(), "entity type")?;
    validate_optional_text(input.entity_key.as_deref(), "entity key")?;
    validate_optional_text(input.baseline_id.as_deref(), "baseline ID")?;
    validate_period(input.from.as_deref(), input.to.as_deref())?;
    if let Some(cursor) = input.cursor.as_ref() {
        validate_timestamp(&cursor.last_seen_at, "event cursor")?;
        validate_text(&cursor.event_id, "event cursor ID")?;
    }
    if input.statuses.as_ref().is_some_and(Vec::is_empty) {
        return Err("Status filter cannot be empty".into());
    }
    if input
        .statuses
        .as_ref()
        .is_some_and(|statuses| statuses.len() > MAX_FILTER_VALUES)
    {
        return Err("Too many status filters".into());
    }

    database.baseline_read(|connection| {
        let mut clauses = Vec::new();
        let mut values = Vec::<SqlValue>::new();

        if let Some(statuses) = input.statuses.as_ref() {
            clauses.push(format!("status IN ({})", placeholders(statuses.len())));
            values.extend(
                statuses
                    .iter()
                    .map(|status| SqlValue::Text(status.as_str().into())),
            );
        }
        if let Some(event_types) = input.event_types.as_ref() {
            clauses.push(format!(
                "event_type IN ({})",
                placeholders(event_types.len())
            ));
            values.extend(event_types.iter().cloned().map(SqlValue::Text));
        }
        add_equal_filter(&mut clauses, &mut values, "entity_type", input.entity_type);
        add_equal_filter(&mut clauses, &mut values, "entity_key", input.entity_key);
        add_equal_filter(&mut clauses, &mut values, "baseline_id", input.baseline_id);
        add_range_filters(
            &mut clauses,
            &mut values,
            "last_seen_at",
            input.from,
            input.to,
        );
        if let Some(cursor) = input.cursor {
            clauses.push("(last_seen_at < ? OR (last_seen_at = ? AND id < ?))".into());
            values.push(SqlValue::Text(cursor.last_seen_at.clone()));
            values.push(SqlValue::Text(cursor.last_seen_at));
            values.push(SqlValue::Text(cursor.event_id));
        }

        let mut sql = String::from(
            "SELECT id, event_type, entity_type, entity_key, title, occurred_at,
                    first_seen_at, last_seen_at, payload_json, baseline_context_json,
                    source, baseline_id, rule_id, rule_version, confidence, status,
                    observation_count, condition_active, schema_version
             FROM security_events",
        );
        append_where(&mut sql, &clauses);
        sql.push_str(" ORDER BY last_seen_at DESC, id DESC LIMIT ?");
        values.push(SqlValue::Integer(i64::from(limit + 1)));

        let mut statement = connection
            .prepare(&sql)
            .map_err(|_| "Unable to prepare paginated security event query")?;
        let rows = statement
            .query_map(params_from_iter(values.iter()), map_security_event)
            .map_err(|_| "Unable to query paginated security events")?;
        let mut items = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "Unable to read paginated security events")?;
        let has_more = items.len() > limit as usize;
        items.truncate(limit as usize);
        let next_cursor = has_more
            .then(|| {
                items.last().map(|event| SecurityEventCursor {
                    last_seen_at: event.last_seen.clone(),
                    event_id: event.event_id.clone(),
                })
            })
            .flatten();
        Ok(SecurityEventPage {
            items,
            next_cursor,
            has_more,
        })
    })
}

pub(crate) fn query_security_event_history(
    database: &Database,
    input: SecurityEventHistoryInput,
) -> Result<SecurityEventHistoryPage, String> {
    let limit = validate_limit(input.limit)?;
    validate_text(&input.entity_type, "entity type")?;
    validate_text(&input.entity_key, "entity key")?;
    validate_optional_text(input.event_type.as_deref(), "event type")?;
    validate_optional_text(input.baseline_id.as_deref(), "baseline ID")?;
    validate_period(input.from.as_deref(), input.to.as_deref())?;
    if let Some(cursor) = input.cursor.as_ref() {
        validate_timestamp(&cursor.observed_at, "history cursor")?;
        if cursor.history_id <= 0 {
            return Err("History cursor ID is invalid".into());
        }
    }

    database.baseline_read(|connection| {
        let mut clauses = vec!["entity_type = ?".into(), "entity_key = ?".into()];
        let mut values = vec![
            SqlValue::Text(input.entity_type),
            SqlValue::Text(input.entity_key),
        ];
        add_equal_filter(&mut clauses, &mut values, "event_type", input.event_type);
        add_equal_filter(&mut clauses, &mut values, "baseline_id", input.baseline_id);
        add_range_filters(
            &mut clauses,
            &mut values,
            "observed_at",
            input.from,
            input.to,
        );
        if let Some(cursor) = input.cursor {
            clauses.push("(observed_at < ? OR (observed_at = ? AND history_id < ?))".into());
            values.push(SqlValue::Text(cursor.observed_at.clone()));
            values.push(SqlValue::Text(cursor.observed_at));
            values.push(SqlValue::Integer(cursor.history_id));
        }

        let mut sql = String::from(
            "SELECT history_id, event_id, transition, observed_at, recorded_at, source,
                    event_type, entity_type, entity_key, baseline_id, rule_id, rule_version,
                    evidence_json, baseline_context_json, previous_status, new_status,
                    event_schema_version, history_schema_version, observation_count
             FROM security_event_history",
        );
        append_where(&mut sql, &clauses);
        sql.push_str(" ORDER BY observed_at DESC, history_id DESC LIMIT ?");
        values.push(SqlValue::Integer(i64::from(limit + 1)));

        let mut statement = connection
            .prepare(&sql)
            .map_err(|_| "Unable to prepare security event history query")?;
        let rows = statement
            .query_map(params_from_iter(values.iter()), map_history_record)
            .map_err(|_| "Unable to query security event history")?;
        let mut items = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "Unable to read security event history")?;
        let has_more = items.len() > limit as usize;
        items.truncate(limit as usize);
        let next_cursor = has_more
            .then(|| {
                items.last().map(|record| SecurityEventHistoryCursor {
                    observed_at: record.observed_at.clone(),
                    history_id: record.history_id,
                })
            })
            .flatten();
        Ok(SecurityEventHistoryPage {
            items,
            next_cursor,
            has_more,
        })
    })
}

fn map_security_event(row: &Row<'_>) -> rusqlite::Result<SecurityEventRecord> {
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
        baseline_context: serde_json::from_str(&baseline_context).unwrap_or(Value::Null),
        source: row.get(10)?,
        baseline_id: row.get(11)?,
        rule_id: row.get(12)?,
        rule_version: row.get(13)?,
        confidence: row.get(14)?,
        status: SecurityEventStatus::from_persisted(&row.get::<_, String>(15)?).map_err(
            |message| {
                rusqlite::Error::FromSqlConversionFailure(
                    15,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        message,
                    )),
                )
            },
        )?,
        observation_count: row.get(16)?,
        condition_active: row.get::<_, i64>(17)? != 0,
        schema_version: row.get(18)?,
    })
}

fn map_history_record(row: &Row<'_>) -> rusqlite::Result<SecurityEventHistoryRecord> {
    let evidence: String = row.get(12)?;
    let baseline_context: String = row.get(13)?;
    Ok(SecurityEventHistoryRecord {
        history_id: row.get(0)?,
        event_id: row.get(1)?,
        transition: row.get(2)?,
        observed_at: row.get(3)?,
        recorded_at: row.get(4)?,
        source: row.get(5)?,
        event_type: row.get(6)?,
        entity_type: row.get(7)?,
        entity_key: row.get(8)?,
        baseline_id: row.get(9)?,
        rule_id: row.get(10)?,
        rule_version: row.get(11)?,
        evidence: serde_json::from_str(&evidence).unwrap_or(Value::Null),
        baseline_context: serde_json::from_str(&baseline_context).unwrap_or(Value::Null),
        previous_status: row.get(14)?,
        new_status: row.get(15)?,
        event_schema_version: row.get(16)?,
        history_schema_version: row.get(17)?,
        observation_count: row.get(18)?,
    })
}

fn validate_limit(limit: Option<u32>) -> Result<u32, String> {
    match limit.unwrap_or(DEFAULT_PAGE_SIZE) {
        0 => Err("Page size must be greater than zero".into()),
        value if value > MAX_PAGE_SIZE => Err(format!("Page size cannot exceed {MAX_PAGE_SIZE}")),
        value => Ok(value),
    }
}

fn validate_period(from: Option<&str>, to: Option<&str>) -> Result<(), String> {
    let from = from
        .map(|value| validate_timestamp(value, "period start"))
        .transpose()?;
    let to = to
        .map(|value| validate_timestamp(value, "period end"))
        .transpose()?;
    if from.zip(to).is_some_and(|(start, end)| start > end) {
        return Err("Period start must not be after period end".into());
    }
    Ok(())
}

fn validate_timestamp(value: &str, field: &str) -> Result<DateTime<chrono::FixedOffset>, String> {
    DateTime::parse_from_rfc3339(value)
        .map_err(|_| format!("{field} must be an RFC 3339 timestamp"))
}

fn validate_optional_values(values: Option<&[String]>, field: &str) -> Result<(), String> {
    if let Some(values) = values {
        if values.is_empty() {
            return Err(format!("{field} filter cannot be empty"));
        }
        if values.len() > MAX_FILTER_VALUES {
            return Err(format!("Too many {field} filters"));
        }
        for value in values {
            validate_text(value, field)?;
        }
    }
    Ok(())
}

fn validate_optional_text(value: Option<&str>, field: &str) -> Result<(), String> {
    value
        .map(|value| validate_text(value, field))
        .transpose()
        .map(|_| ())
}

fn validate_text(value: &str, field: &str) -> Result<(), String> {
    if value.is_empty() || value.len() > MAX_FILTER_LENGTH {
        Err(format!("{field} is invalid"))
    } else {
        Ok(())
    }
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
    use super::{
        query_security_event_history, query_security_events, validate_limit, validate_period,
        SecurityEventHistoryInput, SecurityEventQueryInput,
    };
    use crate::{models::SecurityEventStatus, persistence::Database};

    fn database_with_events() -> Database {
        let database = Database::in_memory().expect("database");
        database
            .baseline_read(|connection| {
                connection
                    .execute(
                        "INSERT INTO behavioral_baselines(
                            baseline_id, created_at, learning_started_at, version, host_id,
                            status, learning_period_seconds, updated_at
                         ) VALUES ('baseline-test', '2026-08-16T09:00:00Z',
                                   '2026-08-16T09:00:00Z', 1, 'host-test', 'ready', 86400,
                                   '2026-08-16T09:00:00Z')",
                        [],
                    )
                    .map_err(|_| "Unable to seed baseline")?;
                for (id, status) in [("event-1", "new"), ("event-2", "seen"), ("event-3", "new")] {
                    connection
                        .execute(
                            "INSERT INTO security_events(
                                id, event_type, occurred_at, source, payload_json, entity_type,
                                entity_key, title, first_seen_at, last_seen_at,
                                baseline_context_json, baseline_id, status, observation_count,
                                condition_active, schema_version
                             ) VALUES (?1, 'process_first_seen', '2026-08-16T10:00:00Z',
                                       'process_collector', '{}', 'process', 'entity-test',
                                       'Observed process', '2026-08-16T10:00:00Z',
                                       '2026-08-16T10:00:00Z', '{}', 'baseline-test', ?2, 1, 1, 1)",
                            rusqlite::params![id, status],
                        )
                        .map_err(|_| "Unable to seed security event")?;
                }
                Ok(())
            })
            .expect("seed events");
        database
    }

    #[test]
    fn page_limits_are_bounded() {
        assert_eq!(validate_limit(None).expect("default"), 50);
        assert_eq!(validate_limit(Some(100)).expect("maximum"), 100);
        assert!(validate_limit(Some(0)).is_err());
        assert!(validate_limit(Some(101)).is_err());
    }

    #[test]
    fn periods_are_ordered_and_rfc3339() {
        assert!(
            validate_period(Some("2026-08-16T10:00:00Z"), Some("2026-08-16T11:00:00Z")).is_ok()
        );
        assert!(
            validate_period(Some("2026-08-16T12:00:00Z"), Some("2026-08-16T11:00:00Z")).is_err()
        );
        assert!(validate_period(Some("not-a-date"), None).is_err());
    }

    #[test]
    fn statuses_have_closed_database_values() {
        assert_eq!(SecurityEventStatus::New.as_str(), "new");
        assert_eq!(SecurityEventStatus::Acknowledged.as_str(), "acknowledged");
        assert_eq!(SecurityEventStatus::Ignored.as_str(), "ignored");
    }

    #[test]
    fn event_cursor_is_stable_when_timestamps_are_equal() {
        let database = database_with_events();
        let first = query_security_events(
            &database,
            SecurityEventQueryInput {
                limit: Some(2),
                ..SecurityEventQueryInput::default()
            },
        )
        .expect("first page");
        assert_eq!(
            first
                .items
                .iter()
                .map(|event| event.event_id.as_str())
                .collect::<Vec<_>>(),
            ["event-3", "event-2"]
        );
        assert!(first.has_more);

        let second = query_security_events(
            &database,
            SecurityEventQueryInput {
                cursor: first.next_cursor,
                limit: Some(2),
                ..SecurityEventQueryInput::default()
            },
        )
        .expect("second page");
        assert_eq!(second.items.len(), 1);
        assert_eq!(second.items[0].event_id, "event-1");
        assert!(!second.has_more);
    }

    #[test]
    fn event_filters_and_entity_history_are_bound_and_paginated() {
        let database = database_with_events();
        let filtered = query_security_events(
            &database,
            SecurityEventQueryInput {
                statuses: Some(vec![SecurityEventStatus::Seen]),
                entity_type: Some("process".into()),
                entity_key: Some("entity-test".into()),
                baseline_id: Some("baseline-test".into()),
                ..SecurityEventQueryInput::default()
            },
        )
        .expect("filtered events");
        assert_eq!(filtered.items.len(), 1);
        assert_eq!(filtered.items[0].event_id, "event-2");

        database
            .baseline_read(|connection| {
                for (event_id, transition, observed_at) in [
                    ("event-1", "first_observed", "2026-08-16T10:00:00Z"),
                    ("event-1", "inactive", "2026-08-16T10:01:00Z"),
                    ("event-1", "reactivated", "2026-08-16T10:02:00Z"),
                ] {
                    connection
                        .execute(
                            "INSERT INTO security_event_history(
                                event_id, transition, observed_at, recorded_at, source,
                                event_type, entity_type, entity_key, baseline_id, evidence_json,
                                baseline_context_json, event_schema_version,
                                history_schema_version, observation_count
                             ) VALUES (?1, ?2, ?3, ?3, 'process_collector',
                                       'process_first_seen', 'process', 'entity-test',
                                       'baseline-test', '{}', '{}', 1, 1, 1)",
                            rusqlite::params![event_id, transition, observed_at],
                        )
                        .map_err(|_| "Unable to seed event history")?;
                }
                Ok(())
            })
            .expect("seed history");

        let first = query_security_event_history(
            &database,
            SecurityEventHistoryInput {
                entity_type: "process".into(),
                entity_key: "entity-test".into(),
                event_type: None,
                baseline_id: None,
                from: None,
                to: None,
                cursor: None,
                limit: Some(2),
            },
        )
        .expect("first history page");
        assert_eq!(first.items.len(), 2);
        assert_eq!(first.items[0].transition, "reactivated");
        assert!(first.has_more);

        let second = query_security_event_history(
            &database,
            SecurityEventHistoryInput {
                entity_type: "process".into(),
                entity_key: "entity-test".into(),
                event_type: None,
                baseline_id: None,
                from: None,
                to: None,
                cursor: first.next_cursor,
                limit: Some(2),
            },
        )
        .expect("second history page");
        assert_eq!(second.items.len(), 1);
        assert_eq!(second.items[0].transition, "first_observed");
    }
}
