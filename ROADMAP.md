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

## Recommended Sprint 2

Build a local baseline and investigation timeline over the factual Sprint 1 events.
Add bounded event history queries, entity history views, and explainable change rules.
Do not add a composite Security Score, external reputation, automated response, or
service/process control until evidence calibration and a separate authorization model
exist.

## Later increments

1. Software inventory and conservative device discovery
2. Explainable detection rules, alerts, and event investigation
3. Reports and opt-in integrations
4. Vulnerability and threat-intelligence sources with secure credentials
5. Explainable Security Score after evidence, calibration, and tests exist

## Explicit non-goals through Sprint 1

VirusTotal, AbuseIPDB, HIBP, urlscan, NVD/CISA, geolocation, reputation,
advanced network scanning, CVE matching, AI, process/network blocking, firewall
management, and service control are not started.
