CREATE TABLE detection_rule_versions (
    rule_id TEXT NOT NULL CHECK (length(trim(rule_id)) > 0),
    rule_version INTEGER NOT NULL CHECK (rule_version > 0),
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    description TEXT NOT NULL CHECK (length(trim(description)) > 0),
    category TEXT NOT NULL CHECK (length(trim(category)) > 0),
    definition_json TEXT NOT NULL CHECK (json_valid(definition_json)),
    definition_sha256 TEXT NOT NULL CHECK (length(definition_sha256) = 64),
    registered_at TEXT NOT NULL,
    schema_version INTEGER NOT NULL DEFAULT 1 CHECK (schema_version > 0),
    PRIMARY KEY (rule_id, rule_version)
);

CREATE TABLE detection_rule_state (
    rule_id TEXT PRIMARY KEY CHECK (length(trim(rule_id)) > 0),
    rule_version INTEGER NOT NULL CHECK (rule_version > 0),
    enabled INTEGER NOT NULL CHECK (enabled IN (0, 1)),
    updated_at TEXT NOT NULL,
    FOREIGN KEY (rule_id, rule_version)
        REFERENCES detection_rule_versions(rule_id, rule_version) ON DELETE RESTRICT
);

CREATE TABLE detections (
    detection_id TEXT PRIMARY KEY CHECK (length(trim(detection_id)) > 0),
    dedup_key TEXT NOT NULL CHECK (length(trim(dedup_key)) > 0),
    correlation_key TEXT NOT NULL CHECK (length(trim(correlation_key)) > 0),
    rule_id TEXT NOT NULL,
    rule_version INTEGER NOT NULL CHECK (rule_version > 0),
    baseline_id TEXT NOT NULL REFERENCES behavioral_baselines(baseline_id) ON DELETE RESTRICT,
    entity_type TEXT NOT NULL CHECK (length(trim(entity_type)) > 0),
    entity_key TEXT NOT NULL CHECK (length(trim(entity_key)) > 0),
    title TEXT NOT NULL CHECK (length(trim(title)) > 0),
    summary TEXT NOT NULL CHECK (length(trim(summary)) > 0),
    severity TEXT NOT NULL
        CHECK (severity IN ('informational', 'low', 'medium', 'high', 'critical')),
    confidence TEXT NOT NULL CHECK (confidence IN ('low', 'medium', 'high')),
    severity_reason TEXT NOT NULL CHECK (length(trim(severity_reason)) > 0),
    confidence_reason TEXT NOT NULL CHECK (length(trim(confidence_reason)) > 0),
    remediation_guidance_json TEXT NOT NULL CHECK (json_valid(remediation_guidance_json)),
    status TEXT NOT NULL DEFAULT 'new'
        CHECK (status IN ('new', 'investigating', 'acknowledged', 'resolved', 'ignored')),
    first_detected_at TEXT NOT NULL,
    last_detected_at TEXT NOT NULL,
    occurrence_count INTEGER NOT NULL DEFAULT 1 CHECK (occurrence_count >= 1),
    condition_active INTEGER NOT NULL DEFAULT 1 CHECK (condition_active IN (0, 1)),
    explanation_json TEXT NOT NULL CHECK (json_valid(explanation_json)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    schema_version INTEGER NOT NULL DEFAULT 1 CHECK (schema_version > 0),
    FOREIGN KEY (rule_id, rule_version)
        REFERENCES detection_rule_versions(rule_id, rule_version) ON DELETE RESTRICT,
    UNIQUE (baseline_id, rule_id, rule_version, dedup_key)
);

CREATE TABLE detection_evidence (
    evidence_id TEXT PRIMARY KEY CHECK (length(trim(evidence_id)) > 0),
    detection_id TEXT NOT NULL REFERENCES detections(detection_id) ON DELETE RESTRICT,
    source_event_id TEXT NOT NULL REFERENCES security_events(id) ON DELETE RESTRICT,
    source_history_id INTEGER
        REFERENCES security_event_history(history_id) ON DELETE RESTRICT,
    evidence_role TEXT NOT NULL CHECK (length(trim(evidence_role)) > 0),
    collector_source TEXT NOT NULL CHECK (length(trim(collector_source)) > 0),
    captured_at TEXT NOT NULL,
    source_event_schema_version INTEGER NOT NULL CHECK (source_event_schema_version > 0),
    source_observation_count INTEGER NOT NULL CHECK (source_observation_count >= 1),
    evidence_json TEXT NOT NULL CHECK (json_valid(evidence_json)),
    evidence_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (evidence_schema_version > 0)
);

CREATE TABLE detection_history (
    history_id INTEGER PRIMARY KEY AUTOINCREMENT,
    detection_id TEXT NOT NULL REFERENCES detections(detection_id) ON DELETE RESTRICT,
    transition TEXT NOT NULL CHECK (transition IN (
        'first_detected', 'reactivated', 'inactive', 'status_changed',
        'classification_changed', 'rule_disabled', 'rule_superseded'
    )),
    observed_at TEXT NOT NULL,
    recorded_at TEXT NOT NULL,
    reason TEXT NOT NULL CHECK (length(trim(reason)) > 0),
    previous_status TEXT
        CHECK (previous_status IS NULL OR previous_status IN (
            'new', 'investigating', 'acknowledged', 'resolved', 'ignored'
        )),
    new_status TEXT
        CHECK (new_status IS NULL OR new_status IN (
            'new', 'investigating', 'acknowledged', 'resolved', 'ignored'
        )),
    severity TEXT NOT NULL
        CHECK (severity IN ('informational', 'low', 'medium', 'high', 'critical')),
    confidence TEXT NOT NULL CHECK (confidence IN ('low', 'medium', 'high')),
    condition_active INTEGER NOT NULL CHECK (condition_active IN (0, 1)),
    occurrence_count INTEGER NOT NULL CHECK (occurrence_count >= 1),
    explanation_json TEXT NOT NULL CHECK (json_valid(explanation_json)),
    history_schema_version INTEGER NOT NULL DEFAULT 1 CHECK (history_schema_version > 0),
    CHECK (
        transition <> 'status_changed'
        OR (previous_status IS NOT NULL AND new_status IS NOT NULL)
    )
);

CREATE TABLE security_score_snapshots (
    snapshot_id TEXT PRIMARY KEY CHECK (length(trim(snapshot_id)) > 0),
    calculated_at TEXT NOT NULL,
    score INTEGER CHECK (score IS NULL OR score BETWEEN 0 AND 100),
    availability TEXT NOT NULL CHECK (availability IN ('available', 'limited', 'unavailable')),
    baseline_id TEXT REFERENCES behavioral_baselines(baseline_id) ON DELETE RESTRICT,
    formula_version INTEGER NOT NULL CHECK (formula_version > 0),
    active_detection_count INTEGER NOT NULL CHECK (active_detection_count >= 0),
    highest_severity TEXT
        CHECK (highest_severity IS NULL OR highest_severity IN (
            'informational', 'low', 'medium', 'high', 'critical'
        )),
    coverage_json TEXT NOT NULL CHECK (json_valid(coverage_json)),
    breakdown_json TEXT NOT NULL CHECK (json_valid(breakdown_json)),
    reason TEXT,
    input_fingerprint TEXT NOT NULL CHECK (length(input_fingerprint) = 64),
    calculation_duration_ms INTEGER NOT NULL CHECK (calculation_duration_ms >= 0),
    schema_version INTEGER NOT NULL DEFAULT 1 CHECK (schema_version > 0),
    CHECK (
        (availability = 'unavailable' AND score IS NULL)
        OR (availability IN ('available', 'limited') AND score IS NOT NULL)
    )
);

CREATE TABLE analysis_checkpoints (
    consumer_id TEXT PRIMARY KEY CHECK (length(trim(consumer_id)) > 0),
    cutover_security_event_history_id INTEGER NOT NULL DEFAULT 0
        CHECK (cutover_security_event_history_id >= 0),
    last_security_event_history_id INTEGER NOT NULL DEFAULT 0
        CHECK (last_security_event_history_id >= 0),
    updated_at TEXT NOT NULL
);

INSERT INTO analysis_checkpoints(
    consumer_id, cutover_security_event_history_id,
    last_security_event_history_id, updated_at
)
SELECT
    'detection-engine-v1', COALESCE(MAX(history_id), 0),
    COALESCE(MAX(history_id), 0),
    strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
FROM security_event_history;

CREATE INDEX idx_detection_rules_category
    ON detection_rule_versions(category, rule_id, rule_version DESC);
CREATE INDEX idx_detections_time
    ON detections(last_detected_at DESC, detection_id DESC);
CREATE INDEX idx_detections_status_time
    ON detections(status, last_detected_at DESC, detection_id DESC);
CREATE INDEX idx_detections_severity_time
    ON detections(severity, last_detected_at DESC, detection_id DESC);
CREATE INDEX idx_detections_rule_time
    ON detections(rule_id, rule_version, last_detected_at DESC, detection_id DESC);
CREATE INDEX idx_detections_entity_time
    ON detections(entity_type, entity_key, last_detected_at DESC, detection_id DESC);
CREATE INDEX idx_detections_active_identity
    ON detections(baseline_id, rule_id, rule_version, correlation_key)
    WHERE condition_active = 1;
CREATE INDEX idx_detection_evidence_detection
    ON detection_evidence(detection_id, captured_at, evidence_id);
CREATE INDEX idx_detection_evidence_event
    ON detection_evidence(source_event_id, source_history_id);
CREATE INDEX idx_detection_history_detection_time
    ON detection_history(detection_id, observed_at DESC, history_id DESC);
CREATE INDEX idx_score_snapshots_time
    ON security_score_snapshots(calculated_at DESC, snapshot_id DESC);

CREATE TRIGGER detection_rule_versions_no_update
BEFORE UPDATE ON detection_rule_versions
BEGIN
    SELECT RAISE(ABORT, 'detection rule versions are immutable');
END;

CREATE TRIGGER detection_rule_versions_no_delete
BEFORE DELETE ON detection_rule_versions
BEGIN
    SELECT RAISE(ABORT, 'detection rule versions are immutable');
END;

CREATE TRIGGER detection_evidence_no_update
BEFORE UPDATE ON detection_evidence
BEGIN
    SELECT RAISE(ABORT, 'detection evidence is immutable');
END;

CREATE TRIGGER detection_evidence_no_delete
BEFORE DELETE ON detection_evidence
BEGIN
    SELECT RAISE(ABORT, 'detection evidence is immutable');
END;

CREATE TRIGGER detection_evidence_history_matches_event
BEFORE INSERT ON detection_evidence
WHEN NEW.source_history_id IS NOT NULL
     AND NOT EXISTS (
         SELECT 1 FROM security_event_history
         WHERE history_id = NEW.source_history_id
           AND event_id = NEW.source_event_id
     )
BEGIN
    SELECT RAISE(ABORT, 'detection evidence history/event mismatch');
END;

CREATE TRIGGER detection_evidence_baseline_matches_detection
BEFORE INSERT ON detection_evidence
WHEN NOT EXISTS (
    SELECT 1
    FROM detections AS detection
    JOIN security_events AS event
      ON event.id = NEW.source_event_id
     AND event.baseline_id = detection.baseline_id
    WHERE detection.detection_id = NEW.detection_id
)
BEGIN
    SELECT RAISE(ABORT, 'detection evidence baseline mismatch');
END;

CREATE TRIGGER detection_history_no_update
BEFORE UPDATE ON detection_history
BEGIN
    SELECT RAISE(ABORT, 'detection history is append-only');
END;

CREATE TRIGGER score_snapshots_no_update
BEFORE UPDATE ON security_score_snapshots
BEGIN
    SELECT RAISE(ABORT, 'security score snapshots are immutable');
END;
