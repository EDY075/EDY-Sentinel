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

## Sprint 2C — English and Português (Brasil) localization (complete)

- Sixteen namespaces and 969 parity-checked leaf keys for each official locale
- Runtime language switching with durable SQLite preference and rollback on persistence failure
- Initial locale priority: saved preference, native Windows user locale, then English fallback
- Locale-aware pluralization, numbers, percentages, dates, timestamps, and relative time
- Localized shell, settings, accessibility, operational telemetry, baseline, security events,
  detections, rules, and Security Score presentation
- Display labels localized without changing technical values, Detection Engine behavior, rule
  definitions, evidence, correlation, deduplication, or Security Score semantics
- Real Tauri review in both languages across five desktop resolutions and all four themes
- Eight release screenshots kept outside Git and complete frontend/Rust/release quality gates

## Sprint 3 Part 1 — Inventory and vulnerability data foundations (complete)

- Native on-demand installed-software inventory from HKLM/HKCU 64/32-bit Registry views
- Conservative normalized identity, strict product-code/scope deduplication, and raw-value retention
- Virtualized bilingual Inventory UI with search, exact filters, sorting, and software details
- SQLite schema v7 inventory snapshots and factual installed/removed/version-changed events
- Dedicated NVD provider with public rate limiting, pagination, resumable initial sync,
  last-modified incremental windows, bounded retry/timeout, cancellation, and local cache
- Separate CISA KEV provider and authoritative local catalog
- Provider lifecycle/status in Settings and offline reuse after successful synchronization
- No software-to-CVE match, vulnerability Detection, or Security Score penalty

## Sprint 3 Parts 2–3 — Conservative matching and Security Score v2 (complete)

- Versioned product identity/CPE resolution and conservative NVD range matching
- Auditable Confirmed/Possible/Unresolved/Not affected evidence with KEV as post-match priority
- Security Score formula v2 with bounded product and vulnerability caps; Possible impact remains zero
- Bilingual Inventory, Product Identity, CVE Detail and score explanations

## Sprint 4A — Adversarial product hardening (complete)

- Startup/cache failure isolation, provider-worker recovery and safe external CVE references
- Real desktop keyboard/focus QA and command-palette lifecycle fixes
- Full architecture, database, dependency, artifact and performance audit

## Sprint 4B — Release blockers and final hardening (technical work complete)

- Versioned incremental vulnerability-history retention with active evidence protection
- Single-owner security analysis in the joint polling cycle
- Authenticode/timestamp pipeline prepared for a legitimate production credential
- Clean-install/uninstall procedure, final desktop QA and signed-public-release gate

The codebase is technically ready for the v1 review checkpoint. Public distribution remains
blocked until a legitimate Authenticode identity signs and timestamps EXE, MSI and NSIS, and the
manual clean-install plus remaining desktop visual matrix are completed in the release environment.

`EDY-PROC-003` remains deferred: current executable identity is not stable enough to infer
tampering from metadata changes without bounded content identity. High/Critical rules,
production calibration, external intelligence, notifications native to Windows, automated
response, and long-term detection archival/downsampling remain future work.

## Later increments

1. Production calibration and broader legitimate-scenario coverage for the six-rule registry
2. Evidence-based software-to-CVE matching using product identity, CPE, and version ranges
3. Conservative device discovery
4. Reports and opt-in integrations
5. Carefully authorized response actions, each behind a separate security boundary

## Explicit non-goals through Sprint 3 Part 1

VirusTotal, AbuseIPDB, HIBP, urlscan, geolocation, reputation, advanced network scanning,
software-to-CVE matching, AI/ML classification, process/network blocking,
firewall management, service control, command-line persistence, and continuous executable
content hashing are not started.
