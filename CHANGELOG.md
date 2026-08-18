# Changelog

All notable changes follow Keep a Changelog principles.

## [Unreleased]

No public release has been published.

## [1.0.0] — 2026-08-17 (final local preparation)

### Added

- Real Windows system, process, network/connection, service, route, adapter, DNS, and installed-
  software collectors with explicit partial/restricted states
- Versioned behavioral baseline, factual Security Events, six explainable Detection rules, evidence,
  deduplication, reopen behavior, workflow history, and Security Score formula v2
- Conservative Product Identity/CPE resolution, NVD applicability matching, CISA KEV enrichment,
  offline evidence, and isolated reconstructible vulnerability cache
- English and Português (Brasil), four semantic-token themes, keyboard/focus contracts, responsive
  layouts, user guide, release notes, and final local validation report

### Changed

- Promoted npm, Cargo, Tauri, Settings, MSI, NSIS, executable, and documentation metadata from
  `1.0.0-rc.1` to final local version `1.0.0`
- Final unsigned local output is marked `UNSIGNED BUILD`; authenticated public distribution remains
  fail-closed until legitimate Authenticode signing and trusted timestamp verification succeed

### Security

- Preserved local-first storage, typed Tauri IPC, standard-user runtime, HTTPS-only NVD/CISA
  providers, no persisted process command lines or file contents, and no product analytics/cloud
  telemetry
- Preserved 31 Confirmed CVEs, 1 Possible at impact zero, 1 Confirmed KEV, score 86 with Detection
  contribution 0 and Vulnerability contribution 14, formula version 2, and six enabled v1 rules

### Validation

- Final frontend, Rust, dependency, database, installer, performance, secret, artifact, localization,
  theme, and Authenticode fail-closed gates are recorded in `SPRINT5B_FINAL_REPORT.md`
- EXE, MSI, and NSIS are local unsigned review artifacts; no tag, push, public installer, or release
  was created

## [1.0.0-rc.1] — 2026-08-17 (technical release candidate)

### Added

- Consistent v1 release-candidate version and Windows product metadata across npm, Cargo, Tauri,
  Settings, MSI, and NSIS packaging
- Professional user guide and explicit technical/public release checklist

- Security Score formula v2 with a bounded Vulnerability Intelligence contribution, canonical
  product aggregation, KEV post-match prioritization, explicit coverage state, and immutable
  formula-versioned history
- Explainable pt-BR/English score card, category math, per-product risk details, Possible impact
  zero, KEV context, and navigation to existing Inventory/CVE evidence
- Migration 0010 and lifecycle tests for version changes, removals, Not affected, stale
  fingerprints, duplicates, caps, queue recovery, and v1/v2 history
- Migration 0011 with versioned, bounded vulnerability-history retention state and incremental
  preservation tests
- Production Authenticode build/verification infrastructure and explicit unsigned-development
  trust marker without a repository-held certificate or private key
- Explicit `UNSIGNED RELEASE CANDIDATE` marker for unsigned prerelease artifacts

### Changed

- A valid numeric score remains visible while vulnerability evaluation is updating, with an
  explicit limited-coverage qualification instead of hiding the current observable result
- Software identity coverage and CVE counts are reported as separate metrics

### Fixed

- A missing, unavailable, or corrupt reconstructible vulnerability cache no longer aborts Tauri
  startup during interrupted-sync recovery
- The vulnerability-provider sync lock is now released after the worker join even when the worker
  terminates exceptionally
- CVE Detail and new NVD ingestion now expose only credential-free HTTPS advisory references;
  legacy plaintext HTTP references remain stored evidence but are not clickable
- The 15-second joint polling cycle no longer starts duplicate Detection Engine and Security Score
  analysis; live telemetry is the single analysis owner and collector cadences are unchanged
- Obsolete vulnerability evaluation families are pruned in bounded daily maintenance batches while
  current Confirmed evidence and audit checkpoints remain intact
- Dialog focus is no longer reset by telemetry-driven parent rerenders, so command-palette keyboard
  input and selection remain stable while live data refreshes

### Security

- Sprint 4A dependency audits report no applicable known npm or Rust vulnerability in the Windows
  runtime graph; unmaintained transitive dependency warnings remain tracked
- EXE, MSI, and NSIS artifacts remain unsigned development outputs and must not be treated as
  publicly distributable Release v1.0 artifacts

### Validation

- Clean NSIS installation and silent uninstall succeeded in a temporary install directory; upgrade
  from 0.1.0 preserved the same install location and both application-database hashes
- Real Tauri RC validated English/pt-BR and Sentinel Blue, Cyber Green, Terminal, and Spectrum at
  configured 1440x900; remaining physical resolutions and CVE Detail capture stay manual gates
- Score regression remained 86 = 100 − 0 − 14 with 31 Confirmed, 1 Possible at impact zero,
  1 KEV Confirmed, formula v2, and six enabled v1 rules

- Real release: score 86 = base 100 − Detections 0 − Vulnerabilities 14; JRE/VirtualBox/Python
  impacts 6/4/4; 31 Confirmed, 1 KEV Confirmed, and 1 Possible with impact zero
- Runtime queue lifecycle: 81 pending → 0, global score Limited → Good, while unresolved identity
  coverage remains independently Limited
- Automated pt-BR/English catalogs and runtime contracts passed. Real release screenshots validated
  English at 1440x900 in Cyber Green; the remaining locale/theme/resolution matrix is an explicit
  manual release-environment acceptance item, not claimed as completed visual evidence

## [0.4.0] — 2026-08-16

### Added

- Complete English and Português (Brasil) localization across the shell, operational telemetry,
  baseline, security events, detections, rules, Security Score, settings, errors, and accessibility
- Runtime language switching with SQLite persistence, native Windows user-locale initialization,
  English fallback, locale-aware pluralization, dates, times, relative time, numbers, and percentages
- Sixteen domain namespaces with 969 parity-checked leaf keys per locale and focused i18n tests
- Display-only localization for stable entity types, evidence/baseline field names, collector
  sources, statuses, severities, confidence levels, and all six rule IDs
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

- Remaining raw presentation labels for known entity types, evidence fields, collector sources,
  operational states, connection states, service startup values, and score coverage components
- Localized accessible names, drawer controls, table navigation, live regions, dialogs, and feedback
- Host identity now resolves the actual Windows installation volume instead of assuming `C:\`
- Sprint 2B handoff now lists the real Rust, TypeScript, IPC, context, page, drawer, and action APIs
- Service PID is documented and implemented as runtime evidence rather than persistent identity
- Detection evidence now renders as structured factual fields instead of opaque JSON

### Security

- Translation remains a presentation-only boundary: rule evaluation, Detection Engine inputs,
  Security Score semantics, IDs, hashes, paths, IPs, processes, services, and evidence values are unchanged
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
