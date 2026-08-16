# EDY Sentinel — Sprint 2A Report

Date: 2026-08-16

Scope: Behavioral Baseline & Security Event Foundation

Sprint 2A base commit: `57fe6e044b423a6e683dcce739d6ae5e0e56aa0a`

Sprint 2A implementation commit: `09d360fdc7bba6a2d4b9712f98f9b1574e73ca32`

Sprint 2A commit subject: `feat: add behavioral baseline and security event foundation`

Current hardening commit: the commit containing this report (`git rev-parse HEAD`)

## Outcome

Sprint 2A is complete. EDY Sentinel now learns a versioned local behavioral baseline
from real Windows telemetry and creates factual, evidence-backed security events after the
baseline becomes Ready. It does not assign threat severity, claim malware, call external
services, or calculate a numeric Security Score.

The implemented pipeline is:

`Collectors → Tracking → Baseline → Event Foundation`

## Baseline

`BaselineEngine` is a Rust application service managed by Tauri. Live process, connection,
and service facts are considered at a bounded 10-second cadence. System/network configuration
uses the existing 15-second overview cadence. All baseline writes use SQLite transactions.

Implemented states:

- `Not initialized`
- `Learning`
- `Ready`
- `Stale` after 24 hours without a successful observation
- `Error` for an invalid/unrecoverable persisted state

Learning periods are configurable at 1 minute (controlled development only), 1 hour,
6 hours, 24 hours (default), 3 days, or 7 days. Manual completion requires the exact
`COMPLETE BASELINE` phrase and at least one real observation. Reset/relearn requires an exact
strong phrase, creates a new version, and preserves prior versions as inactive history.

Persisted baseline facts:

- executable identity: normalized path, file size/mtime, process name, company metadata,
  signer when available, signature state, and architecture;
- process patterns: executable, parent executable, and user when available;
- parent-child relationships;
- process-associated remote IP, port, and protocol;
- service name/display name, state, startup type, binary path, and account. PID remains
  live runtime telemetry and may appear in factual event evidence when available;
- primary interface identity/type/address, default route metric, gateway, and DNS set.

Host identity is `host-v1-<opaque hash>`, derived from MachineGuid and the system-volume
serial. Raw identifiers and hostname are not stored in baseline metadata. Executable identity
hashes normalized metadata; EDY Sentinel does not continuously hash executable contents.

Crash recovery is database-driven: a Learning baseline reopens as Learning with its existing
facts and counts. A manually completed baseline that has not yet received the independent
network cycle seeds the first available network configuration without emitting a false change.

## Events

The central `SecurityEventRecord` contains stable ID/type/entity fields, UTC timestamps,
first/last seen, structured evidence, structured baseline context, source, optional baseline,
rule and correlation-confidence references, local workflow status, observation count,
condition activity, and event schema version.

Implemented factual categories:

- `process_first_seen`
- `executable_first_seen`
- `parent_child_first_seen`
- `destination_first_seen`
- `primary_route_changed`
- `gateway_changed`
- `dns_changed`
- `service_first_seen`
- `service_removed`
- `service_startup_changed`
- `service_binary_changed`
- `service_account_changed`

The stable event identity is `baseline_id + event_type + entity_key`. Repeated sightings update
`last_seen` and `observation_count`. Missing conditions become inactive. A resolved event that
reappears reopens as `New` under the same ID; an ignored event remains ignored. The supported
local states are New, Seen, Acknowledged, Resolved, and Ignored.

Every event has specific evidence. Confidence, when present, means confidence in the factual
entity correlation, never probability of malware. The UI explicitly says `Threat classification:
Not performed` and contains no severity column.

## UI

Overview now shows real baseline state, observation duration/cycles, learned fact counts,
last backend processing time, version, start/completion timestamps, guarded lifecycle actions,
and SQLite state. Security Score remains `Pending detection engine · no numeric score`.

The existing Reports navigation slot became Events, avoiding sidebar growth. The Security
Events view provides search, local status filters, a virtualized/keyboard-navigable table, an
empty state, and an evidence drawer with Summary, Entity, Evidence, Baseline, Timeline, and
Source sections. Command palette actions open Events/View baseline and start/reset a baseline.

The existing Sentinel Blue, Cyber Green, Terminal, and Spectrum tokens were preserved.
Dialogs, drawers, focus styles, active-descendant table navigation, screen-reader labels, and
reduced-motion behavior remain supported.

## Database

- Sprint 2A release ledger: **4**; verified pre-Sprint 2B hardening ledger: **5**
- Sprint 2A migration: `src-tauri/migrations/0004_behavioral_baseline.sql`
- Hardening migration: `src-tauri/migrations/0005_sprint2_hardening.sql`; migration 0004
  remains unchanged.
- Existing data is preserved; migration is additive and transactional.
- Real database: `C:\Users\<USER>\AppData\Roaming\com.edy.sentinel\sentinel.db`
- Sprint 2A validation size after the controlled scenario: **15,790,080 bytes**
- `PRAGMA integrity_check`: **ok**
- Controlled state: 3 preserved baseline versions and 526 factual events, including ambient
  host changes observed during the intentionally short development baseline.

Indexes cover baseline/entity identities, event type/time/baseline/status, executable identity,
and active-state queries. Inactive resolved/ignored security events are retained for 90 days;
other inactive events are bounded to 180 days; active conditions are preserved. Existing raw
observation/full-snapshot retention remains 7 days and Sprint 1 factual telemetry events 30 days.

## Controlled real smoke

The release executable was exercised through its real Tauri/WebView2 surface:

1. created a 1-minute development baseline;
2. observed real processes, connections, services, and Windows network configuration;
3. verified Learning generated zero security events on the first clean baseline;
4. manually completed the baseline and verified Ready;
5. launched `C:\Windows\System32\notepad.exe` after Ready;
6. observed factual executable/process/parent-child events with structured evidence;
7. observed repeated cycles using the same IDs with increased observation counts;
8. closed/reopened Notepad and verified the executable event reactivated under the same ID;
9. created one controlled local `node.exe → 127.0.0.1:45678` TCP connection and observed one
   factual `destination_first_seen` event;
10. stopped both test processes and changed no Windows service or sensitive configuration;
11. validated a second preserved baseline version and missing-network-family recovery without
    a false route/gateway/DNS event.

Service-change detection was validated with an in-memory integration test only; no real Windows
service was created, reconfigured, stopped, or removed.

## Performance

Both measurements used the real optimized Tauri executable on the same 12-logical-processor
host over approximately 15 seconds.

| Metric | Sprint 1.1 | Sprint 2A | Change |
| --- | ---: | ---: | ---: |
| CPU time | 0.688 s / 15.06 s | 0.703 s / 15.21 s | +0.015 s |
| Average CPU, one-core scale | 4.56% | 4.62% | +0.06 pp |
| Average total CPU capacity | ~0.38% | 0.385% | effectively flat |
| Working set | 43.00–46.16 MB | 42.33–46.61 MB | +0.45 MB peak |
| Private bytes | 20.18–23.84 MB | 20.28–25.09 MB | +1.25 MB peak |
| Threads | 23–24 | 22 | no growth |
| SQLite growth in window | 49,152 bytes | 24,576 bytes | -50% |

Observed steady collector durations were approximately system 92–149 ms, processes 54–92 ms,
connections 1–2 ms, and services 46–57 ms. Baseline/event processing was exposed by the backend
and observed at 3–13 ms in the integrated run (19–28 ms during initial bulk learning). No UI
stall, overlapping hydration, uncontrolled writer loop, or material regression was observed.

## Sprint 2A quality gates

| Gate | Result |
| --- | --- |
| `pnpm lint` | PASS |
| `pnpm typecheck` | PASS |
| `pnpm test` | PASS — 16/16 |
| `pnpm audit --audit-level high` | PASS — no known vulnerabilities |
| `pnpm build` | PASS — 1818 modules, JS 264.99 kB / 80.92 kB gzip, CSS 34.13 kB / 7.06 kB gzip |
| `cargo fmt --all -- --check` | PASS |
| `cargo check --all-targets --all-features` | PASS |
| `cargo clippy --all-targets --all-features -- -D warnings` | PASS |
| Rust automated tests | PASS — 31 passed, 1 manual ignored |
| Native Windows collector smoke | PASS when run separately |
| `cargo audit` 0.22.2 | PASS — zero vulnerabilities; 17 allowed transitive maintenance/unsound warnings |
| Tauri release | PASS |
| Release EXE | PASS and opened |
| MSI | PASS |
| NSIS | PASS |
| Real baseline/event smoke | PASS |
| SQLite `integrity_check` | PASS — `ok` |
| `git diff --check` | PASS (Windows LF→CRLF notices only) |

The recurring MSVC linker stdout line that reports creation of the import library remains a
localized toolchain warning already present in prior sprints; it is not a compiler, test, or
Clippy warning in project code.

`cargo audit` reported only inherited transitive advisories: GTK3 crates are unmaintained and
not used in the Windows release, several `unic-*`/`proc-macro-error` crates are unmaintained,
and `glib 0.18.5` has an allowed unsoundness warning. No RustSec vulnerability was found.

## Security review

- Tauri actions accept typed DTOs and exact/allowlisted values.
- All user-derived SQL values use parameters. The only formatted SQL identifiers are internal
  constant baseline table names.
- No arbitrary shell execution, command execution, writable path input, external HTTP/API,
  telemetry cloud, secret, token, `.env`, or analytics path was added.
- Process command lines remain live-only and are not persisted in baseline/event tables.
- Raw MachineGuid, volume serial, hostname, and username are not written to baseline metadata.
- Baseline/event evidence is bounded to factual fields already collected locally.
- Reset/relearn does not delete unrelated Sentinel history and no destructive response exists.

## Release artifacts

- EXE: `D:\CodexBuild\EDY-Sentinel-sprint2a\target\release\edy-sentinel.exe`
- MSI: `D:\CodexBuild\EDY-Sentinel-sprint2a\target\release\bundle\msi\EDY Sentinel_0.1.0_x64_en-US.msi`
- NSIS: `D:\CodexBuild\EDY-Sentinel-sprint2a\target\release\bundle\nsis\EDY Sentinel_0.1.0_x64-setup.exe`

## Captures

All files below are direct 1920×1080 PNG captures of the final real Tauri WebView2 surface:

1. `C:\Users\<USER>\Documents\Codex\2026-08-16\files-pasted-by-the-user-voc\outputs\EDY-Sentinel-sprint2a\01-overview-baseline-learning-1920x1080.png`
2. `C:\Users\<USER>\Documents\Codex\2026-08-16\files-pasted-by-the-user-voc\outputs\EDY-Sentinel-sprint2a\02-overview-baseline-ready-1920x1080.png`
3. `C:\Users\<USER>\Documents\Codex\2026-08-16\files-pasted-by-the-user-voc\outputs\EDY-Sentinel-sprint2a\03-security-events-1920x1080.png`
4. `C:\Users\<USER>\Documents\Codex\2026-08-16\files-pasted-by-the-user-voc\outputs\EDY-Sentinel-sprint2a\04-security-event-detail-1920x1080.png`
5. `C:\Users\<USER>\Documents\Codex\2026-08-16\files-pasted-by-the-user-voc\outputs\EDY-Sentinel-sprint2a\05-overview-baseline-details-1920x1080.png`
6. `C:\Users\<USER>\Documents\Codex\2026-08-16\files-pasted-by-the-user-voc\outputs\EDY-Sentinel-sprint2a\06-terminal-ready-1920x1080.png`

Screenshots remain outside the repository and are not tracked by Git.

## Limitations

- No numeric Security Score, threat classification, severity engine, or complete rule engine.
- No external reputation, CVE, geolocation, ASN, WHOIS, AI, or cloud telemetry.
- No firewall, process termination, service control, or other response action.
- SHA-256 executable content hashes and signer-subject enrichment are not on the live path;
  stable identity uses cached metadata and the already available signer/signature facts.
- A 1-minute manually completed baseline is deliberately incomplete compared with the 24-hour
  production default and can create many legitimate first-seen events as the host changes.
- The compatibility IPC still returns the newest 250 rows for the existing screen. Schema v5
  adds bounded cursor-pagination and entity-history APIs; wiring progressive loading into the
  current Events screen remains a Sprint 2B UI task.
- Service change coverage on the real host was not mutated; it is covered by transactional tests.
- Stale is computed from the last successful persisted observation, not wall-clock UI polling.

## SPRINT 2B VERIFIED HANDOFF

This section supersedes the previous Sprint 2B handoff. The code and schema are the source
of truth; no alias APIs were added to preserve incorrect documentation.

### Verification anchor

- Previous commit: `09d360fdc7bba6a2d4b9712f98f9b1574e73ca32`
- Hardening commit: the commit containing this verified handoff (`git rev-parse HEAD`)
- Intended subject: `fix: harden sprint 2 handoff and baseline contracts`
- Branch: `main`
- SQLite ledger: v5; baseline schema: v1; factual-event schema: v1; provenance schema: v1
- Migration 0004 is immutable. Future Sprint 2B schema work starts at migration 0006.
- This hardening does not implement a Detection Engine, operational rules, severity,
  a numeric Security Score, external APIs, or response actions.

### Hardening verification

The pre-Sprint 2B hardening was validated independently from the historical Sprint 2A gates:

| Gate | Hardening result |
| --- | --- |
| `pnpm lint` / `pnpm typecheck` | PASS |
| `pnpm test` | PASS — 17/17 |
| `pnpm audit --audit-level high` | PASS — no known vulnerabilities |
| `pnpm build` | PASS — 1818 modules, JS 264.99 kB / 80.92 kB gzip, CSS 34.13 kB / 7.06 kB gzip |
| `cargo fmt --all -- --check` | PASS |
| `cargo check --all-targets --all-features` | PASS |
| `cargo clippy --all-targets --all-features -- -D warnings` | PASS |
| Rust automated tests | PASS — 46 passed, 1 manual smoke ignored in the normal suite |
| Native Windows collector smoke | PASS — 1/1 when run explicitly |
| `cargo audit` 0.22.2 | PASS — zero vulnerabilities; 17 documented transitive warnings |
| Tauri release / EXE / MSI / NSIS | PASS |
| SQLite v4→v5 preservation | PASS — 526 aggregate events and 526 origin-history rows preserved in the validation copy |
| Real SQLite after controlled native launch | PASS — ledger v5, `integrity_check=ok`, zero FK violations |

The controlled 12.11-second optimized native sample used 1.125 CPU-seconds: 9.288% on a
one-core scale, or 0.774% of total capacity on the 12-logical-processor host. Working set was
41.14–42.43 MB, private bytes 18.59–20.21 MB, and the process remained at 26 threads. The
sample includes startup, migration, hydration, and active collection, so it is intentionally
not presented as a steady-state replacement for the longer Sprint 2A comparison.

Before applying v5 to the real database, the exact v4 database was copied to
`archive/pre-sprint2b-hardening/sentinel-v4-pre-hardening.db`. The archive is ignored by Git
and remains locally recoverable. After the controlled native launch, the real database had
3 baseline versions, 661 aggregate factual events, 792 append-only history rows, zero null
event timestamps, zero orphan baseline references, and no baseline in Error. The additional
events are factual ambient host observations from the deliberately short development baseline,
not detections or threat findings.

Hardening release artifacts were generated under
`D:\CodexBuild\EDY-Sentinel-hardening\target\release`; these build outputs remain outside the
repository and are not part of the commit.

### Current architecture

`Windows collectors → TelemetryEngine tracking/last-good → persistence → BaselineEngine → factual security_events → typed Tauri IPC → TelemetryProvider → React`

Rust owns collection, baseline state, factual comparison, provenance, queries, validation,
and SQLite. React remains presentation-only. `TelemetryProvider` is the single frontend
hydration owner and prevents overlapping collection. Do not move baseline/detection logic
into React and do not add a second polling loop.

There is no separate Repository trait or EventService. `persistence::Database` is the sole
concrete SQLite repository. `BaselineEngine` currently owns factual-event query, upsert,
deduplication, condition activity, reopen, workflow status, provenance transitions, and
retention. Future detection ownership must be a separate Rust service/module rather than
being placed in React or silently merged into factual observations.

### Schema and migrations

- `src-tauri/migrations/0004_behavioral_baseline.sql`: released Sprint 2A baseline/event
  foundation; never edit it.
- `src-tauri/migrations/0005_sprint2_hardening.sql`: adds baseline `error_code`/`updated_at`,
  rebuilds `security_events` safely, and creates append-only `security_event_history`.
- `security_events.first_seen_at` and `last_seen_at` are now `NOT NULL` after backfill.
- Event status, activity, observation count, schema version, and optional rule version have
  SQLite constraints aligned with the Rust domain.
- `security_events.baseline_id` uses a non-cascading `ON DELETE RESTRICT` foreign key. Baseline
  history cannot be deleted while factual evidence refers to it.
- The low-selectivity `condition_active` field has no isolated index. A partial active-cycle
  index supports the real lifecycle query, while a retention index supports actual cleanup.
- `security_event_history` stores only meaningful transitions: first observation,
  reactivation, inactivity, and workflow status change. Continuous refreshes update the
  aggregate event without appending history rows.
- History rows are append-only. Event retention may cascade to its provenance only when the
  aggregate event itself is legitimately removed; baseline deletion never cascades events.

### ACTUAL SPRINT 2B INTERFACES

#### Rust modules

- `src-tauri/src/baseline.rs` — `BaselineEngine`, persisted lifecycle, factual comparison,
  event aggregation, provenance transitions, status workflow, and retention.
- `src-tauri/src/telemetry.rs` — `TelemetryEngine::collect`, last-good tracking, and Sprint 1
  factual telemetry transitions.
- `src-tauri/src/collectors/{system,network,processes,connections,services}.rs` — Windows facts.
- `src-tauri/src/event_query.rs` — bounded cursor pagination and entity-history queries.
- `src-tauri/src/host_identity.rs` — real Windows system-volume discovery and opaque host ID.
- `src-tauri/src/rules.rs` — versioned `RuleDefinition` contract only; no evaluator/registry.
- `src-tauri/src/commands.rs` — narrow typed Tauri boundary.
- `src-tauri/src/models.rs` — Rust DTOs and closed status enums.
- `src-tauri/src/persistence/mod.rs` — concrete SQLite repository, migration runner, transaction
  boundaries, and controlled persisted Error marker.
- `src-tauri/src/lib.rs` — managed `Database`, `TelemetryEngine`, and `BaselineEngine` wiring.

#### Rust structs and enums

- Baseline/event DTOs: `BaselineStatus`, `BaselineEntityCounts`, `BaselineSummary`,
  `SecurityEventStatus`, `SecurityEventRecord`, `BaselineActionInput`,
  `SecurityEventStatusInput`.
- Query DTOs: `SecurityEventCursor`, `SecurityEventQueryInput`, `SecurityEventPage`,
  `SecurityEventHistoryCursor`, `SecurityEventHistoryInput`,
  `SecurityEventHistoryRecord`, `SecurityEventHistoryPage`.
- Rule contract: `RuleDefinition`, `RuleCondition`, `RuleOperator`, `EvidenceRequirement`,
  and `RulePolicy`.
- Live DTOs remain `LiveTelemetrySnapshot`, `ProcessRecord`, `ConnectionRecord`,
  `ServiceRecord`, `TelemetryEvent`, `CollectorHealth`, and `CollectorStatus`.

#### Public/application Rust functions

- `TelemetryEngine::collect(&self) -> Result<LiveTelemetrySnapshot, String>`
- `BaselineEngine::summary(&self, &Database) -> Result<BaselineSummary, String>`
- `BaselineEngine::security_events(&self, &Database) -> Result<Vec<SecurityEventRecord>, String>`
- `BaselineEngine::observe_live(&self, &Database, &LiveTelemetrySnapshot) -> Result<(), String>`
- `BaselineEngine::observe_network(&self, &Database, &SystemOverview) -> Result<(), String>`
- `BaselineEngine::{start_new_baseline, reset_baseline, complete_learning}`
- `BaselineEngine::set_event_status`
- `event_query::{query_security_events, query_security_event_history}`
- `rules::RuleDefinition::validate`
- Repository boundary: `Database::{open, save_snapshot, persist_live_telemetry, get_theme,
  set_theme, status, baseline_read, baseline_transaction, baseline_engine_transaction,
  mark_active_baseline_error}`.

#### Tauri commands

- Collection/state: `get_system_overview`, `get_live_telemetry`, `get_database_status`,
  `get_theme`, `set_theme`, `get_capabilities`.
- Baseline: `get_baseline_summary`, `start_new_baseline`, `reset_baseline`,
  `complete_baseline_learning`.
- Events: `get_security_events` (compatibility newest-250 view),
  `get_security_events_page`, `get_security_event_history`, `set_security_event_status`.

#### TypeScript context, hook, and actions

`src/features/telemetry/TelemetryProvider.tsx` exports `TelemetryProvider`, `useTelemetry`,
and `telemetryIntervals`. Its real `TelemetryContextValue` contains:

- state: `live`, `loading`, `refreshing`, `overview`, `database`, `snapshot`, `error`,
  `health`, `baseline`, `securityEvents`, `securityError`;
- actions: `setLive`, `refresh`, `refreshSecurity`, `startBaseline`, `resetBaseline`,
  `completeBaseline`, `setEventStatus`.

`refreshSecurityFoundation` does not exist. `runBaselineAction` is a private `App.tsx`
dispatcher, not a context method. `updateSecurityEventStatus` is an IPC wrapper; the context
action is `setEventStatus`.

#### TypeScript IPC wrappers and types

- `src/lib/tauri.ts`: `getBaselineSummary`, `getSecurityEvents`, `getSecurityEventsPage`,
  `getSecurityEventHistory`, `startNewBaseline`, `resetBaseline`,
  `completeBaselineLearning`, `updateSecurityEventStatus`, plus system/theme wrappers.
- `src/types/baseline.ts`: baseline/event workflow, cursor page, entity history, transition,
  and action contracts.
- `src/types/rules.ts`: `RuleDefinition`, conditions, evidence requirements, policies, and
  operators. These types do not execute a rule.
- `src/types/telemetry.ts` and `src/types/system.ts`: live telemetry/system contracts.

#### Pages, drawers, and real UI actions

- `src/App.tsx` owns page routing and action composition.
- `src/features/overview/Overview.tsx`: Overview plus the
  `src/features/baseline/BaselinePanel.tsx` summary/actions.
- `src/features/processes/ProcessesView.tsx`: Processes page and inline process detail drawer.
- `src/features/connections/ConnectionsView.tsx`: Network page and inline connection drawer.
- `src/features/services/ServicesView.tsx`: Services page; there is no service detail drawer.
- `src/features/events/SecurityEventsView.tsx`: Events page and inline factual event drawer.
- `src/features/baseline/BaselineActionDialog.tsx`: guarded start, reset/relearn, and complete
  learning actions.
- `src/features/events/SecurityEventsView.tsx`: mark Seen on first open, Acknowledge, Resolve,
  and Ignore actions.
- The existing Events screen still consumes the compatibility newest-250 wrapper. Cursor APIs
  are ready for progressive UI loading without changing 250 to an arbitrary larger cap.

### Hardening decisions

#### Service PID semantics

`baseline_services` intentionally has no PID column. PID is runtime telemetry in
`ServiceRecord`/`service_observations`, changes across executions, and is not persistent
service identity. Current PID may appear in factual event evidence when Windows provides it.

#### Persisted Error semantics

`Error` is terminal for one baseline version. After an already persisted baseline is loaded,
a durable state-processing failure rolls back its operation and is marked in a second,
controlled write with an allowlisted code, fixed sanitized message, and UTC `updated_at`.
Reset/relearn creates a new version and preserves the Error version. Validation mistakes,
missing baselines, partial/restricted collectors, throttling, UI errors, and transaction
start/commit failures return normally and do not create or poison a baseline. Stack traces,
SQL, paths, evidence payloads, raw identifiers, and personal data are never persisted in the
Error message.

#### Host identity

The code uses `GetSystemWindowsDirectoryW → GetVolumePathNameW → GetVolumeInformationW` and
does not assume `C:\`. The existing v1 hash remains byte-for-byte stable when MachineGuid and
the system-volume serial are available. Partial-signal IDs are domain-separated; an existing
opaque persisted host ID is reused during transient signal loss. Hostname is never a fallback,
and raw MachineGuid/serial values are not stored.

#### Company, signer, and SHA-256

CompanyName is descriptive file metadata, never cryptographic signer identity. Signature
state and signer remain separate facts. Executable content SHA-256 stays outside the live
loop; future enrichment must be on demand or bounded/cached background work, never hundreds
of hashes per refresh.

### Event query and provenance contract

Event pagination is keyset/cursor based on `(last_seen_at DESC, id DESC)`, defaults to 50,
and is capped at 100. Bound filters support status, event type, entity type/key, baseline,
and RFC3339 period. Entity history uses `(observed_at DESC, history_id DESC)` and supports
event type, baseline, and period filters. SQL fragments are internal/allowlisted; all values
are bound parameters.

Factual provenance retains collector, baseline, timestamp, event/entity identity, evidence,
baseline context, optional future rule ID/version, event schema, transition, and observation
count. Sprint 2B must create detections separately and must never overwrite factual history.

### Versioned RuleDefinition contract

The Rust and TypeScript contracts include `rule_id`, version, name, description, category,
enabled state, conditions, required evidence, versioned severity/confidence policies, and
remediation guidance. Validation requires nonzero rule/policy versions and explicit evidence.
No registry, evaluator, operational rule, detection, or severity assignment exists yet.
Any future detection must store both `rule_id` and `rule_version` so historical results remain
explainable after rule changes.

### Future detection policy

- Observation is not Detection; Detection is not Threat.
- Learning creates facts and never novelty events.
- No prior comparable fact means seed the baseline, not emit a change.
- Severity cannot come from novelty alone. `executable_first_seen`, unsigned/unknown metadata,
  new destinations/ports/services, or restricted metadata cannot individually produce
  High/Critical. A rule must require explicit corroborating facts.
- Confidence measures evidence availability and correlation quality. It is not malware
  probability unless a future calibrated model explicitly defines it that way.
- Failed service enumeration is unavailable/error, not `Stopped`. Protected/restricted
  metadata absence is not proof of maliciousness.
- Network destinations are dynamic. An isolated endpoint or unresolved/kernel association
  cannot receive elevated severity without corroborating evidence.
- Security Score remains nonnumeric until coverage, calibrated rules, known denominators,
  category impact, explanations, and anti-misleading safeguards are validated.

### Development/test baseline warning

The active one-minute baseline and approximately 526 factual events are a controlled
development/test configuration. They are not a production baseline and must never be used as
a production calibration or false-positive dataset. The production default remains 24 hours.

### Known risks

- Short baselines generate legitimate novelty volume.
- Dynamic network endpoints and PID correlation can be temporarily unresolved.
- Signer data may be unavailable; CompanyName cannot substitute for it.
- Protected process/service metadata may be restricted without indicating a threat.
- Transitive Tauri/GTK/unicode RustSec maintenance warnings remain dependency-upgrade work.
- The compatibility Events screen still shows only its newest 250 rows until the prepared
  cursor API is connected to progressive loading.
- Provenance before migration v5 can only be backfilled from the final v4 aggregate evidence;
  overwritten earlier evidence cannot be reconstructed retroactively.

### Exact Sprint 2B TODOs

1. Keep migrations 0004 and 0005 immutable; begin new schema changes at 0006.
2. Implement a small Rust Detection Engine over factual events/history; do not place logic in React.
3. Create only a small, explainable, versioned rule set with corroborating evidence and
   calibration fixtures; keep detections separate from factual observations.
4. Persist `rule_id` and `rule_version` on future detections and retain evidence provenance.
5. Connect cursor pagination/entity history to progressive Events/investigation UI loading.
6. Add false-positive regression, rule-version, provenance, and evidence-coverage tests before
   exposing operational severity.
7. Add a detection detail section only after a rule fires; preserve the factual event drawer.
8. Keep Security Score pending until rule coverage/calibration and denominators are measurable.
9. Repeat native smoke, performance comparison, SQLite integrity/retention/FK checks, all four
   themes, security audits, packaging, and real screenshots for any relevant UI change.

Sprint 2B detection work was not started by this hardening delivery.
