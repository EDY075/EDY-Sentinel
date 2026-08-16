# Changelog

All notable changes follow Keep a Changelog principles.

## [0.4.0] — 2026-08-16

### Added

- SQLite schema v5 constraints, non-cascading baseline references, bounded cursor pagination,
  entity history, and append-only factual-event transition provenance
- Versioned Rust/TypeScript `RuleDefinition` contract without an evaluator or operational rules
- Controlled persisted baseline Error metadata with sanitized messages and explicit recovery
- SQLite schema v6 with immutable rule versions, local enabled state, Detections, evidence,
  append-only Detection history, Security Score snapshots, and a durable analysis checkpoint
- Independent Rust Detection Engine with six versioned rules, typed negative conditions,
  bounded delta processing, correlation windows, precedence, deduplication, and reopen policy
- Explainable Security Score formula v1 with Ready/coverage gates, grouped penalties,
  formula version, complete breakdown, snapshot deduplication, and 365-day retention
- Cursor-paginated Detections and factual Events workspace, structured provenance drawer,
  Rules view, score breakdown, real command-palette actions, and in-app Detection notices

### Fixed

- Host identity now resolves the actual Windows installation volume instead of assuming `C:\`
- Sprint 2B handoff now lists the real Rust, TypeScript, IPC, context, page, drawer, and action APIs
- Service PID is documented and implemented as runtime evidence rather than persistent identity
- Detection evidence now renders as structured factual fields instead of opaque JSON

### Security

- Factual evidence history cannot be overwritten, baseline references cannot cascade-delete
  events, and event status/activity values are constrained at both Rust and SQLite boundaries
- Detection analysis uses a transaction/checkpoint independent from baseline Error semantics;
  retained factual events referenced by Detection evidence cannot be removed
- Rule/score inputs are parameterized and allowlisted; no external API, shell, PowerShell,
  command-line persistence, destructive response, or arbitrary rule editing was introduced

### Calibration

- All v1 rules are capped at Medium; `unknown` and `restricted` never mean unsigned
- `EDY-PROC-003` remains deferred until executable identity can be established reliably
- Controlled native smoke produced only the expected process-rule findings; normal host use did
  not create an unrelated Detection flood

## [0.3.0] — 2026-08-16

### Added

- Local Rust Behavioral Baseline Engine with versioned host-bound Learning, Ready, Stale,
  Error, reset, relearn, manual test completion, and crash recovery semantics
- Factual executable, process-pattern, parent-child, destination, service, route, gateway,
  DNS, and interface baseline entities
- Central security-event foundation with evidence, baseline context, stable identity,
  deduplication, reopen policy, workflow status, and bounded retention
- Security Events operational table, factual evidence drawer, baseline Overview panel,
  strong confirmation dialogs, and real command-palette actions
- SQLite schema v4 and focused Rust/TypeScript baseline and event tests

### Security

- Host baseline identity stores only an opaque hash derived from Windows installation and
  system-volume identifiers; raw identifiers and hostname are not persisted in metadata
- Executable identity hashes metadata only; no continuous content hashing, cloud telemetry,
  external reputation, threat severity, or numeric Security Score was introduced

### Deferred

- Rule-engine severity, composite Security Score, threat intelligence, CVE enrichment,
  automated response, firewall, process termination, and service control

## [0.2.1] — 2026-08-16

### Changed

- Normalized process CPU to total logical-processor capacity while retaining an
  explicitly labeled core-equivalent diagnostic value and honest warm-up state
- Separated collector execution health from metadata coverage; restricted records
  are counted without falsely degrading a successful collector
- Added detailed collector attempt/success timestamps, duration, observations,
  restrictions, and native failure diagnostics
- Hardened connection correlation with a bounded recent-process cache, PID-reuse
  guard, explicit association states, and two-snapshot close debounce
- Selected the primary network interface from the native Windows best route and
  exposed interface class, gateway, source address, and route metric
- Split executable Company metadata, native signature result, and certificate signer
- Versioned the factual event schema and made tracking/deduplication monotonic

### Security

- Signature verification and signer extraction use local Windows trust APIs only,
  with bounded enrichment and cache invalidation on executable changes
- Process command lines remain live-only; no external reputation or hashing service
  was added

### Fixed

- Eliminated misleading collector degradation caused solely by access-restricted
  process/service metadata
- Prevented stale PID correlation from attributing a connection to a reused PID
- Prevented a single transient network miss from emitting a false close event

## [0.2.0] — 2026-08-16

### Added

- Native Windows process telemetry with CPU warm-up, memory, ownership, thread,
  architecture, executable metadata, and cached signature status
- Native TCP/UDP IPv4/IPv6 connection telemetry with owner-PID correlation
- Read-only Service Control Manager telemetry for service state and startup settings
- Central live telemetry, pause/resume, collector health, factual diffs, and tracking
- Virtualized process/connection/service tables and keyboard navigation, with process and
  connection detail drawers
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
