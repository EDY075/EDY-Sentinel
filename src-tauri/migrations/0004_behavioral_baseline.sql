CREATE TABLE IF NOT EXISTS behavioral_baselines (
    baseline_id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    learning_started_at TEXT NOT NULL,
    learning_completed_at TEXT,
    version INTEGER NOT NULL CHECK (version > 0),
    host_id TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('learning', 'ready', 'stale', 'error')),
    observation_count INTEGER NOT NULL DEFAULT 0 CHECK (observation_count >= 0),
    schema_version INTEGER NOT NULL DEFAULT 1,
    learning_period_seconds INTEGER NOT NULL CHECK (learning_period_seconds > 0),
    last_observed_at TEXT,
    error_message TEXT,
    active INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    UNIQUE(host_id, version)
);

CREATE TABLE IF NOT EXISTS baseline_executables (
    baseline_id TEXT NOT NULL REFERENCES behavioral_baselines(baseline_id) ON DELETE CASCADE,
    executable_key TEXT NOT NULL,
    normalized_path TEXT,
    process_name TEXT NOT NULL,
    company_name TEXT,
    signer_name TEXT,
    signature_status TEXT NOT NULL,
    architecture TEXT,
    file_size INTEGER,
    file_modified_at TEXT,
    first_seen_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    observation_count INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (baseline_id, executable_key)
);

CREATE TABLE IF NOT EXISTS baseline_process_patterns (
    baseline_id TEXT NOT NULL REFERENCES behavioral_baselines(baseline_id) ON DELETE CASCADE,
    pattern_key TEXT NOT NULL,
    executable_key TEXT NOT NULL,
    process_name TEXT NOT NULL,
    parent_executable_key TEXT,
    parent_process_name TEXT,
    user_name TEXT,
    first_seen_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    observation_count INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (baseline_id, pattern_key)
);

CREATE TABLE IF NOT EXISTS baseline_parent_child_relationships (
    baseline_id TEXT NOT NULL REFERENCES behavioral_baselines(baseline_id) ON DELETE CASCADE,
    relationship_key TEXT NOT NULL,
    parent_executable_key TEXT NOT NULL,
    parent_process_name TEXT NOT NULL,
    child_executable_key TEXT NOT NULL,
    child_process_name TEXT NOT NULL,
    first_seen_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    observation_count INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (baseline_id, relationship_key)
);

CREATE TABLE IF NOT EXISTS baseline_network_destinations (
    baseline_id TEXT NOT NULL REFERENCES behavioral_baselines(baseline_id) ON DELETE CASCADE,
    destination_key TEXT NOT NULL,
    executable_key TEXT,
    process_name TEXT,
    remote_ip TEXT NOT NULL,
    remote_port INTEGER NOT NULL,
    protocol TEXT NOT NULL,
    first_seen_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    observation_count INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (baseline_id, destination_key)
);

CREATE TABLE IF NOT EXISTS baseline_services (
    baseline_id TEXT NOT NULL REFERENCES behavioral_baselines(baseline_id) ON DELETE CASCADE,
    service_key TEXT NOT NULL,
    service_name TEXT NOT NULL,
    display_name TEXT NOT NULL,
    status TEXT NOT NULL,
    startup_type TEXT NOT NULL,
    binary_path TEXT,
    account_name TEXT,
    first_seen_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    observation_count INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (baseline_id, service_key)
);

CREATE TABLE IF NOT EXISTS baseline_network_configurations (
    baseline_id TEXT PRIMARY KEY REFERENCES behavioral_baselines(baseline_id) ON DELETE CASCADE,
    primary_interface_key TEXT,
    primary_interface_type TEXT,
    primary_ipv4 TEXT,
    gateway TEXT,
    dns_json TEXT NOT NULL,
    route_metric INTEGER,
    interfaces_json TEXT NOT NULL,
    first_seen_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    observation_count INTEGER NOT NULL DEFAULT 1
);

ALTER TABLE security_events ADD COLUMN entity_type TEXT NOT NULL DEFAULT 'unknown';
ALTER TABLE security_events ADD COLUMN entity_key TEXT NOT NULL DEFAULT 'unknown';
ALTER TABLE security_events ADD COLUMN title TEXT NOT NULL DEFAULT 'Observed change';
ALTER TABLE security_events ADD COLUMN first_seen_at TEXT;
ALTER TABLE security_events ADD COLUMN last_seen_at TEXT;
ALTER TABLE security_events ADD COLUMN baseline_context_json TEXT NOT NULL DEFAULT '{}';
ALTER TABLE security_events ADD COLUMN baseline_id TEXT;
ALTER TABLE security_events ADD COLUMN rule_id TEXT;
ALTER TABLE security_events ADD COLUMN confidence TEXT;
ALTER TABLE security_events ADD COLUMN status TEXT NOT NULL DEFAULT 'new';
ALTER TABLE security_events ADD COLUMN observation_count INTEGER NOT NULL DEFAULT 1;
ALTER TABLE security_events ADD COLUMN condition_active INTEGER NOT NULL DEFAULT 1;
ALTER TABLE security_events ADD COLUMN schema_version INTEGER NOT NULL DEFAULT 1;

UPDATE security_events
SET first_seen_at = COALESCE(first_seen_at, occurred_at),
    last_seen_at = COALESCE(last_seen_at, occurred_at);

CREATE INDEX IF NOT EXISTS idx_baselines_host_version
    ON behavioral_baselines(host_id, version DESC);
CREATE INDEX IF NOT EXISTS idx_baseline_executables_identity
    ON baseline_executables(executable_key, baseline_id);
CREATE INDEX IF NOT EXISTS idx_baseline_process_patterns_executable
    ON baseline_process_patterns(executable_key, baseline_id);
CREATE INDEX IF NOT EXISTS idx_baseline_destinations_identity
    ON baseline_network_destinations(destination_key, baseline_id);
CREATE INDEX IF NOT EXISTS idx_baseline_services_identity
    ON baseline_services(service_key, baseline_id);
CREATE INDEX IF NOT EXISTS idx_security_events_entity
    ON security_events(entity_key, last_seen_at DESC);
CREATE INDEX IF NOT EXISTS idx_security_events_type_time
    ON security_events(event_type, last_seen_at DESC);
CREATE INDEX IF NOT EXISTS idx_security_events_baseline
    ON security_events(baseline_id, last_seen_at DESC);
CREATE INDEX IF NOT EXISTS idx_security_events_status_time
    ON security_events(status, last_seen_at DESC);
