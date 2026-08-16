ALTER TABLE process_observations ADD COLUMN company_name TEXT;
ALTER TABLE process_observations ADD COLUMN signature_status TEXT NOT NULL DEFAULT 'unknown';
ALTER TABLE process_observations ADD COLUMN signer_name TEXT;

ALTER TABLE connection_observations ADD COLUMN association_status TEXT NOT NULL DEFAULT 'unresolved';
ALTER TABLE connection_observations ADD COLUMN process_last_seen TEXT;

ALTER TABLE telemetry_events ADD COLUMN collector TEXT NOT NULL DEFAULT 'unknown';
ALTER TABLE telemetry_events ADD COLUMN factual_payload_json TEXT NOT NULL DEFAULT '{}';
ALTER TABLE telemetry_events ADD COLUMN schema_version INTEGER NOT NULL DEFAULT 1;

CREATE INDEX IF NOT EXISTS idx_telemetry_events_collector_time
    ON telemetry_events(collector, occurred_at DESC);
