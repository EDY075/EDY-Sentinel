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

- Versioned, host-bound local baseline with Learning/Ready/Stale/Error lifecycle
- Process, executable, parent-child, destination, service, route, gateway, DNS, and
  interface facts learned from real telemetry
- Crash-safe learning, guarded reset/relearn, and preserved baseline history
- Central factual security events with evidence, deduplication, reopen policy, and local
  New/Seen/Acknowledged/Resolved/Ignored workflow
- Security Events table and evidence drawer without fabricated severity
- SQLite schema v4 and bounded event retention

## Recommended Sprint 2B

Build a small explainable rule engine over the factual Sprint 2A event contract. Add
versioned rule definitions, evidence requirements, calibrated severity, entity history,
and investigation workflow. Do not calculate a composite Security Score until rules and
calibration are validated. External reputation, automated response, and process/service
control remain separate future authorization boundaries.

## Later increments

1. Software inventory and conservative device discovery
2. Explainable detection rules, alerts, and event investigation
3. Reports and opt-in integrations
4. Vulnerability and threat-intelligence sources with secure credentials
5. Explainable Security Score after evidence, calibration, and tests exist

## Explicit non-goals through Sprint 2A

VirusTotal, AbuseIPDB, HIBP, urlscan, NVD/CISA, geolocation, reputation,
advanced network scanning, CVE matching, AI, process/network blocking, firewall
management, and service control are not started.
