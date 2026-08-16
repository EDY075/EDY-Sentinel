# Roadmap

## Sprint 0 — Foundation (complete)

- Tauri 2, React, TypeScript, and Rust workspace
- Real Windows system and network collection
- Typed IPC and honest partial/error states
- SQLite migrations, snapshots, and persisted theme
- Premium responsive shell and four themes
- Security, build, and contributor documentation

## Sprint 1 — Live endpoint observation (complete)

- Real Windows processes with cached executable metadata and signature status
- TCP/UDP IPv4/IPv6 owner-PID tables and process correlation
- Read-only Windows services with state and startup configuration
- Central live telemetry store, pause/resume, collector health, and virtualized tables
- First/last seen tracking, factual diffs, normalized SQLite observations, and retention
- Minimal frontend and Rust test foundations

## Sprint 1.1 — Telemetry accuracy and hardening (complete)

- Total-capacity process CPU semantics with core-equivalent diagnostic detail
- Collector execution health separated from observation coverage/restrictions
- PID-reuse-safe connection association and bounded recent-process correlation
- Native best-route selection and factual network interface classification
- Company metadata, signature verification, and signer identity kept distinct
- Versioned factual event payloads, monotonic tracking, deduplication, and close debounce
- SQLite schema v3 with integrity and retention validation

## Sprint 2A — Behavioral baseline and security event foundation (complete)

- Versioned, host-bound local baseline with Learning/Ready/Stale lifecycle and an Error
  value reserved in the schema for explicit persistence hardening
- Process, executable, parent-child, destination, service, route, gateway, DNS, and
  interface facts learned from real telemetry
- Crash-safe learning, guarded reset/relearn, and preserved baseline history
- Central factual security events with evidence, deduplication, reopen policy, and local
  New/Seen/Acknowledged/Resolved/Ignored workflow
- Security Events table and evidence drawer without fabricated severity
- SQLite schema v4 and bounded event retention

## Sprint 2B — Explainable detections and Security Score v1 (complete)

- Independent Rust Detection Engine consuming only factual-history deltas through a durable
  checkpoint; no continuous full-history scan and no retroactive pre-v6 classification
- Six enabled, immutable v1 rules with typed conditions, exclusions, evidence requirements,
  correlation windows, precedence, false-positive context, and safe remediation guidance
- Separate detection, evidence, and append-only history storage with exact rule provenance,
  deduplication, reopen, workflow, cursor pagination, and source-event retention protection
- Detections/Events workspace, structured evidence drawer, Rules view, real command-palette
  actions, bounded paging, in-app notification seeding, and accessible responsive behavior
- Conservative Security Score formula v1 with baseline/collector coverage gates, grouped
  penalties, complete breakdown, immutable snapshots, formula version, and 365-day retention
- SQLite schema v6; real EXE/MSI/NSIS smoke; controlled benign process fixture; no malware,
  service mutation, external reputation, shell response, or fabricated release data

`EDY-PROC-003` remains deferred: current executable identity is not stable enough to infer
tampering from metadata changes without bounded content identity. High/Critical rules,
production calibration, external intelligence, notifications native to Windows, automated
response, and long-term detection archival/downsampling remain future work.

## Later increments

1. Production calibration and broader legitimate-scenario coverage for the six-rule registry
2. Software inventory and conservative device discovery
3. Reports and opt-in integrations
4. Vulnerability and threat-intelligence sources with secure credentials
5. Carefully authorized response actions, each behind a separate security boundary

## Explicit non-goals through Sprint 2B

VirusTotal, AbuseIPDB, HIBP, urlscan, NVD/CISA, geolocation, reputation,
advanced network scanning, CVE matching, AI/ML classification, process/network blocking,
firewall management, service control, command-line persistence, and continuous executable
content hashing are not started.
