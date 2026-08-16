# Changelog

All notable changes follow Keep a Changelog principles.

## [0.2.0] — 2026-08-16

### Added

- Native Windows process telemetry with CPU warm-up, memory, ownership, thread,
  architecture, executable metadata, and cached signature status
- Native TCP/UDP IPv4/IPv6 connection telemetry with owner-PID correlation
- Read-only Service Control Manager telemetry for service state and startup settings
- Central live telemetry, pause/resume, collector health, factual diffs, and tracking
- Virtualized process/connection/service tables, keyboard navigation, and detail drawers
- SQLite schema v2 with normalized observations, event persistence, and retention
- Focused Vitest and Rust tests for filters, sorting, health, parsing, correlation, diffs,
  migrations, and persistence

### Security

- No PowerShell, external enrichment, fabricated severity, continuous hashing, or
  process command-line persistence in the live pipeline

### Deferred

- Detection/risk scoring, threat intelligence, geolocation, process/network response,
  firewall management, service control, and a composite Security Score

## [0.1.0] — 2026-08-16

### Added

- Tauri 2, React 19, TypeScript, and Rust foundation
- Real Windows system, hardware, storage, and network collectors
- SQLite schema v1, migration ledger, snapshots, and theme persistence
- Sentinel Blue, Cyber Green, Terminal, and Spectrum themes
- Responsive shell, compact sidebar, command palette, toasts, and feedback states
- Restrictive CSP, least-privilege capability, documentation, and verification commands

### Deferred

- Analysis/Security Score engine and all Sprint 1+ collectors and integrations
