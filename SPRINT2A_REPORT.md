# EDY Sentinel — Sprint 2A Report

Date: 2026-08-16

Scope: Behavioral Baseline & Security Event Foundation

Base commit: `57fe6e044b423a6e683dcce739d6ae5e0e56aa0a`

Final commit: the commit containing this report (`git rev-parse HEAD`)

Commit subject: `feat: add behavioral baseline and security event foundation`

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
- service name/display name, state, startup type, binary path, account, and PID context;
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

- Migration ledger version: **4**
- New migration: `src-tauri/migrations/0004_behavioral_baseline.sql`
- Existing data is preserved; migration is additive and transactional.
- Real database: `C:\Users\<USER>\AppData\Roaming\com.edy.sentinel\sentinel.db`
- Final validation size after the controlled scenario: **15,790,080 bytes**
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

## Quality gates

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
- Event IPC currently returns the newest 250 rows; pagination/entity history is a Sprint 2B task.
- Service change coverage on the real host was not mutated; it is covered by transactional tests.
- Stale is computed from the last successful persisted observation, not wall-clock UI polling.

## HANDOFF FOR SPRINT 2B

### Current architecture

React has one `TelemetryProvider`, one non-overlapping hydration path, and typed IPC in
`src/lib/tauri.ts`. Rust owns real collectors, `TelemetryEngine` tracking/last-good state,
`BaselineEngine` learning/comparison, and SQLite repositories. The dependency direction is:

`Windows collectors → TelemetryEngine tracking → BaselineEngine → security_events → typed IPC → React`

Do not move baseline/detection logic into React and do not add a second frontend polling loop.

### Git and schema

- Base: `57fe6e044b423a6e683dcce739d6ae5e0e56aa0a`
- Final: commit containing this report (`git rev-parse HEAD`)
- Subject: `feat: add behavioral baseline and security event foundation`
- Branch: `main`
- SQLite migration ledger: v4
- Baseline model schema: v1
- Security event schema: v1

### Main files

- `src-tauri/src/baseline.rs` — engine, identities, learning, comparison, event lifecycle/tests
- `src-tauri/src/telemetry.rs` — real telemetry tracking and Sprint 1 factual events
- `src-tauri/src/commands.rs` — Tauri boundary and baseline/event commands
- `src-tauri/src/models.rs` — Rust DTO contracts
- `src-tauri/src/persistence/mod.rs` — migration runner and transaction boundary
- `src-tauri/migrations/0004_behavioral_baseline.sql` — schema v4
- `src/types/baseline.ts` — frontend baseline/event contracts
- `src/features/telemetry/TelemetryProvider.tsx` — single frontend store/hydration owner
- `src/features/baseline/*` — Overview baseline UI and guarded actions
- `src/features/events/*` — event table/filter/detail drawer
- `src/lib/tauri.ts` — typed IPC wrapper
- `docs/adr/0006-behavioral-baseline-and-factual-events.md` — decisions and boundaries

### Important Rust interfaces

- `BaselineEngine::summary(&Database) -> Result<BaselineSummary, String>`
- `BaselineEngine::observe_live(&Database, &LiveTelemetrySnapshot)`
- `BaselineEngine::observe_network(&Database, &SystemOverview)`
- `BaselineEngine::security_events(&Database) -> Result<Vec<SecurityEventRecord>, String>`
- `BaselineEngine::{start_new_baseline, reset_baseline, complete_learning}`
- `BaselineEngine::set_event_status`
- Tauri commands: `get_baseline_summary`, `get_security_events`, `start_new_baseline`,
  `reset_baseline`, `complete_baseline_learning`, `set_security_event_status`

### Important TypeScript interfaces

- `BaselineSummary`, `BaselineEntityCounts`, `BaselineState`, `BaselineAction`
- `SecurityEvent`, `SecurityEventStatus`
- `TelemetryContextValue.baseline`, `.securityEvents`, `.refreshSecurityFoundation`,
  `.runBaselineAction`, and `.updateSecurityEventStatus`

### Decisions that must remain explicit

- Observation is not Detection; Detection is not Threat.
- Learning creates facts and never novelty events.
- No prior comparable fact means seed the baseline, not emit a change.
- Event IDs are stable per baseline/type/entity; refreshes update one row.
- Reopen uses the same ID; ignored stays ignored.
- Confidence means factual correlation confidence only.
- Baseline history is versioned and preserved.
- Internal timestamps are UTC; UI formatting is local time.
- Command lines are never persisted.
- Security Score stays pending until evidence-calibrated rules exist.

### Exact Sprint 2B next steps

1. Add migration v5; never edit migration 0004 after release.
2. Define a small versioned `RuleDefinition` contract with ID, name, description, category,
   conditions, evidence requirements, severity, confidence semantics, remediation guidance,
   enabled state, and version.
3. Implement at most a small explainable rule set over existing factual events; require
   multiple corroborating facts where severity is assigned.
4. Keep raw observation events distinct from rule detections in storage and UI.
5. Add paginated event/entity-history queries instead of increasing the current 250-row cap.
6. Add calibration fixtures, false-positive regression tests, rule-version tests, and clear
   evidence provenance before exposing any severity.
7. Add a detection detail section only after a rule fires; preserve the factual drawer.
8. Design Security Score only after rule coverage/calibration is measurable. Do not infer a
   score from baseline novelty alone.
9. Repeat native smoke, performance comparison, SQLite integrity/retention checks, all four
   themes, security audits, release packaging, and real screenshots.

### Risks for Sprint 2B

- Short baselines create legitimate novelty volume; never calibrate rules from the 1-minute
  development scenario.
- Network endpoints are dynamic and can be kernel-associated or temporarily uncorrelated.
- Company metadata is not signer identity; signature availability can be restricted.
- Protected processes/services can omit metadata; absence is not proof of maliciousness.
- A missing service enumeration result is not automatically `Stopped`.
- Transitive Tauri/GTK/unicode RustSec maintenance warnings remain dependency-upgrade work;
  they are not a reason to bypass audit output.
- A numeric score without coverage denominators, rule calibration, and explanations would be
  misleading and remains prohibited.

Sprint 2B was not started in this delivery.
