CREATE TABLE IF NOT EXISTS process_observations (
    process_key TEXT PRIMARY KEY,
    pid INTEGER NOT NULL,
    process_name TEXT NOT NULL,
    parent_pid INTEGER,
    user_name TEXT,
    executable_path TEXT,
    start_time TEXT,
    first_seen_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    observation_count INTEGER NOT NULL CHECK (observation_count > 0),
    active INTEGER NOT NULL CHECK (active IN (0, 1)),
    access_status TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS connection_observations (
    connection_key TEXT PRIMARY KEY,
    protocol TEXT NOT NULL CHECK (protocol IN ('tcp', 'udp')),
    ip_version TEXT NOT NULL CHECK (ip_version IN ('ipv4', 'ipv6')),
    process_id INTEGER,
    local_address TEXT NOT NULL,
    local_port INTEGER NOT NULL,
    remote_address TEXT,
    remote_port INTEGER,
    tcp_state TEXT,
    first_seen_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    observation_count INTEGER NOT NULL CHECK (observation_count > 0),
    active INTEGER NOT NULL CHECK (active IN (0, 1))
);

CREATE TABLE IF NOT EXISTS service_observations (
    service_key TEXT PRIMARY KEY,
    service_name TEXT NOT NULL,
    display_name TEXT NOT NULL,
    binary_path TEXT,
    account_name TEXT,
    process_id INTEGER,
    first_seen_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    observation_count INTEGER NOT NULL CHECK (observation_count > 0),
    active INTEGER NOT NULL CHECK (active IN (0, 1)),
    status TEXT NOT NULL,
    startup_type TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS telemetry_events (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    subject_type TEXT NOT NULL,
    subject_key TEXT NOT NULL,
    message TEXT NOT NULL,
    occurred_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_process_observations_active_last_seen
    ON process_observations(active, last_seen_at DESC);
CREATE INDEX IF NOT EXISTS idx_connection_observations_active_last_seen
    ON connection_observations(active, last_seen_at DESC);
CREATE INDEX IF NOT EXISTS idx_service_observations_active_last_seen
    ON service_observations(active, last_seen_at DESC);
CREATE INDEX IF NOT EXISTS idx_telemetry_events_occurred_at
    ON telemetry_events(occurred_at DESC);
