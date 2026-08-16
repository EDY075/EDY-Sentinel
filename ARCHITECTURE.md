# Architecture

## Decision summary

EDY Sentinel uses a modular local-first monolith. React owns presentation. Rust owns collection, validation, application logic, and persistence. Tauri commands form the typed trust boundary.

```text
React UI
  -> typed command wrappers
Tauri command boundary
  -> collection orchestration / validation (`spawn_blocking`)
Rust domain models
  -> Windows adapters (Registry, WMI, IP Helper, SCM, sysinfo)
  -> managed TelemetryEngine (cadence, tracking, factual diff)
  -> BaselineEngine (versioned learning, factual comparison, deduplication)
  -> DetectionEngine (versioned rules, bounded delta correlation, provenance)
  -> ScoreEngine (coverage gate, explainable penalties, versioned snapshots)
  -> native Software Inventory (on-demand Registry collection, factual change tracking)
  -> NVD / CISA KEV providers (bounded HTTPS sync, validation, local cache)
  -> SQLite repository (migrations, observations, baselines, events, detections, score, inventory, vulnerability data)
```

## Frontend responsibilities

- `src/App.tsx`: application shell, navigation, and UI action composition
- `src/main.tsx`: React root and the single `TelemetryProvider` mount
- `src/features/overview`: real snapshot presentation and formatting
- `src/features/telemetry`: the single live-store and collection health boundary
- `src/features/processes`: virtualized process table and process detail drawer
- `src/features/connections`: virtualized TCP/UDP table and connection detail drawer
- `src/features/services`: read-only Windows services table
- `src/features/baseline`: behavioral baseline status, details, and guarded lifecycle actions
- `src/features/events`: factual security-event table and evidence drawer
- `src/features/detections`: rule conclusions, structured evidence, workflow, and provenance
- `src/features/rules`: read-only rule metadata and the local enabled switch
- `src/features/score`: current Security Score and its complete breakdown
- `src/features/security`: Detections/Events workspace and bounded cursor pager
- `src/features/inventory`: installed-software table, filters, sorting, and detail drawer
- `src/features/settings`: language preference and vulnerability-provider status/actions
- `src/features/theme`: four token-based theme choices
- `src/components/ui`: reusable badge, status, skeleton, dialog, drawer, tooltip, and chart primitives
- `src/lib/tauri.ts`: the only frontend IPC entry point
- `src/types`: DTO contracts mirrored from serialized Rust types

The frontend cannot execute SQL, WMI, Registry reads, shell commands, or arbitrary file access.

## Rust responsibilities

- `collectors/system.rs`: host, OS, CPU, GPU, memory, disks, uptime
- `collectors/network.rs`: adapters, addresses, gateways, DNS, native best-route
  selection, route metrics, and interface classification
- `collectors/processes.rs`: persistent sysinfo process sampling, total-capacity CPU
  normalization, Toolhelp thread counts, cached executable metadata, local
  WinVerifyTrust validation, and certificate signer extraction
- `collectors/connections.rs`: native IP Helper TCP/UDP owner-PID tables for IPv4/IPv6
- `collectors/services.rs`: read-only Service Control Manager enumeration and configuration
- `telemetry.rs`: collector cadences, last-good snapshots, first/last seen tracking,
  observation counts, PID-identity correlation, bounded recent-process state,
  deduplicated tracking, and versioned factual change events
- `baseline.rs`: host-bound baseline versions, learning/recovery state, executable and
  behavior identity, Ready comparisons, factual event deduplication, and reopen policy
- `rules.rs`: immutable, versioned rule contract and the six-rule built-in registry
- `detection.rs`: independent checkpoint consumer, correlation, precedence, deduplication,
  reopen/status policy, evidence snapshots, and append-only detection history
- `detection_query.rs`: parameterized keyset pagination and evidence queries
- `score.rs`: formula-v1 coverage gate, grouped penalties, immutable snapshots, and retention
- `inventory.rs`: native HKLM/HKCU 64/32-bit inventory, conservative identity,
  snapshot persistence, and factual installed/removed/version-changed events
- `vulnerability.rs`: dedicated NVD and CISA KEV HTTPS providers, validation, bounded sync,
  cancellation, incremental cache writes, and provider lifecycle
- `commands.rs`: narrow Tauri commands and input validation
- `models.rs`: serialization contracts
- `persistence/mod.rs`: SQLite lifecycle, migrations, snapshots, and settings
- `migrations`: append-only schema evolution

Collectors do not know about React. The persistence layer does not accept SQL from IPC. Future engines will consume collector DTOs through application services rather than coupling to Windows APIs.

## Data honesty contract

- Values originate at refresh time from Windows.
- Optional data remains optional; it is never converted to a fabricated zero.
- Collector-specific failure becomes a partial result with an issue message.
- The first successful live snapshot is a baseline and does not emit synthetic
  start/open events.
- A failed network protocol family retains its last-good baseline and never creates
  a mass of false close events.
- A failed Service Control Manager enumeration retains the last-good service state;
  collection failure, restricted configuration, and a factual `Stopped` state remain
  distinct conditions.
- Process CPU is unavailable during sampler warm-up rather than fabricated as zero.
- Displayed process CPU is normalized by total logical-processor capacity; the raw
  aggregate is retained only as an explicitly named core-equivalent diagnostic.
- Collector execution health and observation coverage are independent. Access-denied
  metadata may increase `restrictedCount` without degrading a successful collector.
- A connection is associated only when PID plus process identity are consistent;
  ambiguous PID reuse remains unresolved rather than being guessed.
- Connection closure requires two consecutive successful misses, while failed
  protocol families preserve their last-good baseline.
- Company version metadata, trust status, and certificate signer are separate facts.
- Remote destinations are dynamic observations. A new endpoint or an unresolved process
  association is not, by itself, evidence of a threat or a basis for elevated severity.
- UDP remote endpoints and TCP state remain unavailable because those concepts do
  not apply to an unconnected UDP binding.
- A total orchestration failure becomes an error state.
- Security Events remain factual and have no severity. Detections are separate rule conclusions.
- Confidence describes evidence/correlation quality, never malware probability.
- A numeric Security Score exists only with a Ready baseline, enabled rules, and measured
  system/process/connection/service coverage; otherwise the UI shows an explicit unavailable state.

## Persistence

SQLite opens from Tauri's `app_data_dir`. Startup enables foreign keys, WAL, and a busy timeout, then applies each migration transactionally. `schema_migrations` is the single version ledger. Current schema version: 7.

Tables prepared in Sprint 0: `system_snapshots`, `network_snapshots`, `devices`, `alerts`, `security_events`, `settings`, and `integrations`. Integration secrets are not stored in the database; only a future `secret_ref` may be stored.

Sprint 1 adds `process_observations`, `connection_observations`,
`service_observations`, and `telemetry_events`. Command lines are deliberately not
persisted. Live entity heartbeats are batched at approximately 60 seconds, while
factual events are written promptly. Full Sprint 0 system/network snapshots are
limited to one write per five minutes.

Service PID is runtime telemetry stored in `service_observations`; it is deliberately
not part of persistent service identity in `baseline_services`. A factual event may
include the current PID as evidence when Windows provides it, without using that PID as
the baseline key.

Sprint 1.1 migration 0003 separates executable company/signature/signer facts,
persists connection association state and recent process timestamps, and versions
event collector/payload semantics without removing existing observations.

Sprint 2A migration 0004 adds append-only baseline metadata/fact tables and extends the
existing `security_events` foundation with entity identity, evidence, baseline context,
workflow state, deduplication counts, activity, and schema version. Baseline lifecycle and
fact writes are transactional. Previous baseline versions remain stored and inactive.

Pre-Sprint 2B hardening migration 0005 makes factual-event timestamps and workflow values
schema-enforced, adds a non-cascading baseline reference, persists controlled baseline Error
metadata, and introduces append-only provenance for meaningful event transitions. It also
adds bounded cursor-pagination/entity-history query support. Continuous refreshes do not
append provenance rows, preserving the existing cadence and storage profile.

Sprint 2B migration 0006 adds immutable rule versions, local rule state, detections,
detection evidence/history, score snapshots, and a durable analysis checkpoint. Foreign keys
use `RESTRICT` where deleting a source would destroy explanation; factual-event retention skips
events referenced by detection evidence. Rule evaluation runs in its own analysis transaction,
so a detection failure cannot put the behavioral baseline into Error. The migration seeds the
checkpoint at the current factual-history maximum: pre-v6 events remain facts and are not
retroactively classified by a new rule version.

Sprint 3 Part 1 migration 0007 adds inventory snapshots/current rows/append-only observations,
factual software-change events, minimal NVD and CISA KEV caches, and provider state. Inventory
facts remain severity-free. The NVD cache retains CVE metadata, CVSS, weaknesses, essential
references, and bounded CPE applicability needed by a future matcher; no match is produced yet.

## Behavioral baseline and security events

The BaselineEngine runs after factual tracking. Live facts are sampled for baseline work no
more often than every 10 seconds; network configuration is sampled at the existing 15-second
system cadence. During Learning, facts are accumulated without emitting novelty events.
Ready comparisons process keys and deltas against the immutable active baseline.

Baseline states are `Not initialized`, `Learning`, `Ready`, `Stale`, and `Error`. Learning
state survives crashes because timestamps, counts, and facts are persisted in the same
transaction. The schema-v5 hardening boundary reserves persisted `Error` for a failure of
an already stored baseline whose state must survive restart; it records a stable error code,
a sanitized message, and an update timestamp. Transient failures before a baseline exists
return normally and do not create a false baseline. Stack traces and sensitive details are
not persisted.

Host identity is an opaque SHA-256 derivation of MachineGuid and the dynamically resolved
Windows system-volume serial; no drive letter is assumed. Raw identifiers and hostname are
not stored in baseline metadata. Executable Company metadata remains separate from verified
signer identity. Executable content SHA-256 is outside the live loop and, if introduced,
must be computed on demand or by bounded background work with caching.

Factual security events have no severity. Their stable key is baseline + event type +
entity key. Repeated observations update one event. A resolved inactive condition reopens
under the same ID when it reappears; ignored events stay ignored.

The DetectionEngine consumes only new `security_event_history` rows through a durable cursor,
in batches of at most 250 and no more than eight batches per refresh. It evaluates six enabled
v1 rules, all capped at Medium. A Detection stores the exact rule version and immutable source
evidence. Persistent conditions update one identity; a resolved condition reopens under the same
ID, while ignored history remains preserved. Rule precedence prevents one executable cluster
from producing additive lower-specificity score penalties.

Security Score formula v1 starts at 100 and subtracts the highest penalty per correlation group.
It uses only active detections whose rules remain enabled. Resolved and Ignored detections do not
contribute; Acknowledged remains active until the condition ends or is resolved. Equal inputs are
deduplicated, with a 15-minute heartbeat; snapshots have 365-day retention. The value is an
observed posture under current Sentinel coverage, not a percentage guarantee of security.

The one-minute learning period is a controlled development/test configuration. Events
created from that deliberately short baseline are not a production calibration dataset.

Initial retention is seven days for inactive observations and full snapshots, and
30 days for factual telemetry events. Cleanup runs at startup and then at most once
every six hours; active rows are never removed. `VACUUM` is not run in the live path.

Inactive resolved/ignored security events use 90-day retention; any other inactive
security event is bounded to 180 days. Active conditions are preserved.

Detection and evidence history are currently preserved without automatic deletion. This is
intentional for explainability; a future downsampling/archival policy must preserve every source
needed by an active Detection. Score snapshots use 365-day retention.

## Live cadence and pause semantics

The React `TelemetryProvider` is the only UI hydration loop and prevents overlapping
requests. The managed Rust engine samples processes on each 2.5-second hydration,
connections no more often than every four seconds, and services no more often than
every 15 seconds. Overview/system collection runs every 15 seconds. Expensive static
executable metadata is cached by path and last-write time.

Pause freezes automatic collection at the latest snapshot while keeping the UI,
filters, sorting, and drawers navigable. Resume performs the next normal collection.
Manual refresh remains available in both states.

## Software inventory and vulnerability repositories

Inventory is explicitly outside the live telemetry cadence. Opening Inventory seeds an empty
repository once; further collection is manual. Original Registry values are retained, while
normalization stays separate. Strict product-code + install-scope equality is the only cross-key
deduplication rule; name similarity is never evidence of identity.

NVD and CISA KEV synchronization runs on Tauri blocking workers. Both providers use an HTTPS-only
client, bounded response bodies, payload validation, parameterized SQLite writes, sanitized errors,
and cooperative cancellation. NVD respects public rate limits and uses last-modified windows for
incremental updates. CISA KEV uses an independent authoritative replacement transaction. Neither
provider receives endpoint data, and the UI uses the resulting cache offline.

## Planned module seams

Device discovery, software-to-CVE matching, additional threat intelligence, alerts, reports,
integrations, and automated response remain architectural seams only.
Empty fake implementations are deliberately absent.

## ADRs

- `docs/adr/0001-modular-local-first.md`
- `docs/adr/0002-real-data-partial-results.md`
- `docs/adr/0003-sqlite-and-secret-boundary.md`
- `docs/adr/0004-live-telemetry-and-retention.md`
- `docs/adr/0005-telemetry-accuracy-semantics.md`
- `docs/adr/0006-behavioral-baseline-and-factual-events.md`
- `docs/adr/0007-software-inventory-and-local-vulnerability-repositories.md`
