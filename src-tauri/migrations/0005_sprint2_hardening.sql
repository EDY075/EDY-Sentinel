ALTER TABLE behavioral_baselines ADD COLUMN error_code TEXT;
ALTER TABLE behavioral_baselines ADD COLUMN updated_at TEXT;

UPDATE behavioral_baselines
SET updated_at = COALESCE(last_observed_at, learning_completed_at, created_at)
WHERE updated_at IS NULL;

CREATE TABLE security_events_v5 (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    occurred_at TEXT NOT NULL,
    source TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    entity_type TEXT NOT NULL DEFAULT 'unknown',
    entity_key TEXT NOT NULL DEFAULT 'unknown',
    title TEXT NOT NULL DEFAULT 'Observed change',
    first_seen_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    baseline_context_json TEXT NOT NULL DEFAULT '{}',
    baseline_id TEXT REFERENCES behavioral_baselines(baseline_id) ON DELETE RESTRICT,
    rule_id TEXT,
    rule_version INTEGER CHECK (rule_version IS NULL OR rule_version > 0),
    confidence TEXT,
    status TEXT NOT NULL DEFAULT 'new'
        CHECK (status IN ('new', 'seen', 'acknowledged', 'resolved', 'ignored')),
    observation_count INTEGER NOT NULL DEFAULT 1 CHECK (observation_count >= 1),
    condition_active INTEGER NOT NULL DEFAULT 1 CHECK (condition_active IN (0, 1)),
    schema_version INTEGER NOT NULL DEFAULT 1 CHECK (schema_version > 0)
);

INSERT INTO security_events_v5 (
    id, event_type, occurred_at, source, payload_json, entity_type, entity_key,
    title, first_seen_at, last_seen_at, baseline_context_json, baseline_id,
    rule_id, rule_version, confidence, status, observation_count, condition_active,
    schema_version
)
SELECT
    id, event_type, occurred_at, source, payload_json, entity_type, entity_key,
    title, COALESCE(first_seen_at, occurred_at), COALESCE(last_seen_at, occurred_at),
    baseline_context_json, baseline_id, rule_id, NULL, confidence, status,
    observation_count, condition_active, schema_version
FROM security_events;

DROP TABLE security_events;
ALTER TABLE security_events_v5 RENAME TO security_events;

CREATE INDEX idx_security_events_time
    ON security_events(last_seen_at DESC, id DESC);
CREATE INDEX idx_security_events_status_time
    ON security_events(status, last_seen_at DESC, id DESC);
CREATE INDEX idx_security_events_type_time
    ON security_events(event_type, last_seen_at DESC, id DESC);
CREATE INDEX idx_security_events_entity_time
    ON security_events(entity_type, entity_key, last_seen_at DESC, id DESC);
CREATE INDEX idx_security_events_baseline_time
    ON security_events(baseline_id, last_seen_at DESC, id DESC);
CREATE INDEX idx_security_events_active_cycle
    ON security_events(baseline_id, event_type, id)
    WHERE condition_active = 1;
CREATE INDEX idx_security_events_retention
    ON security_events(condition_active, last_seen_at, status);

CREATE TABLE security_event_history (
    history_id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id TEXT NOT NULL REFERENCES security_events(id) ON DELETE CASCADE,
    transition TEXT NOT NULL
        CHECK (transition IN ('first_observed', 'reactivated', 'inactive', 'status_changed')),
    observed_at TEXT NOT NULL,
    recorded_at TEXT NOT NULL,
    source TEXT NOT NULL,
    event_type TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_key TEXT NOT NULL,
    baseline_id TEXT REFERENCES behavioral_baselines(baseline_id) ON DELETE RESTRICT,
    rule_id TEXT,
    rule_version INTEGER CHECK (rule_version IS NULL OR rule_version > 0),
    evidence_json TEXT NOT NULL CHECK (json_valid(evidence_json)),
    baseline_context_json TEXT NOT NULL CHECK (json_valid(baseline_context_json)),
    previous_status TEXT
        CHECK (previous_status IS NULL OR previous_status IN ('new', 'seen', 'acknowledged', 'resolved', 'ignored')),
    new_status TEXT
        CHECK (new_status IS NULL OR new_status IN ('new', 'seen', 'acknowledged', 'resolved', 'ignored')),
    event_schema_version INTEGER NOT NULL CHECK (event_schema_version > 0),
    history_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (history_schema_version > 0),
    observation_count INTEGER NOT NULL CHECK (observation_count >= 1),
    CHECK (
        transition <> 'status_changed'
        OR (previous_status IS NOT NULL AND new_status IS NOT NULL)
    )
);

INSERT INTO security_event_history (
    event_id, transition, observed_at, recorded_at, source, event_type,
    entity_type, entity_key, baseline_id, rule_id, rule_version, evidence_json,
    baseline_context_json, previous_status, new_status, event_schema_version,
    history_schema_version, observation_count
)
SELECT
    id, 'first_observed', first_seen_at, first_seen_at, source, event_type,
    entity_type, entity_key, baseline_id, rule_id, rule_version,
    CASE WHEN json_valid(payload_json) THEN payload_json ELSE json_object('legacyRaw', payload_json) END,
    CASE WHEN json_valid(baseline_context_json) THEN baseline_context_json ELSE json_object('legacyRaw', baseline_context_json) END,
    NULL, NULL, schema_version, 1, observation_count
FROM security_events;

CREATE INDEX idx_security_event_history_entity_time
    ON security_event_history(entity_type, entity_key, observed_at DESC, history_id DESC);
CREATE INDEX idx_security_event_history_event_time
    ON security_event_history(event_id, observed_at DESC, history_id DESC);
CREATE INDEX idx_security_event_history_baseline_time
    ON security_event_history(baseline_id, observed_at DESC, history_id DESC);

CREATE TRIGGER security_event_history_no_update
BEFORE UPDATE ON security_event_history
BEGIN
    SELECT RAISE(ABORT, 'security event history is append-only');
END;
