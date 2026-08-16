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
  -> SQLite repository (migrations, observations, baselines, events, settings)
```

## Frontend responsibilities

- `src/app`: future provider and shell composition boundary
- `src/features/overview`: real snapshot presentation and formatting
- `src/features/telemetry`: the single live-store and collection health boundary
- `src/features/processes`: virtualized process table and process detail drawer
- `src/features/connections`: virtualized TCP/UDP table and connection detail drawer
- `src/features/services`: read-only Windows services table
- `src/features/baseline`: behavioral baseline status, details, and guarded lifecycle actions
- `src/features/events`: factual security-event table and evidence drawer
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
- UDP remote endpoints and TCP state remain unavailable because those concepts do
  not apply to an unconnected UDP binding.
- A total orchestration failure becomes an error state.
- Security Score remains `not_implemented` until an explainable engine exists.

## Persistence

SQLite opens from Tauri's `app_data_dir`. Startup enables foreign keys, WAL, and a busy timeout, then applies each migration transactionally. `schema_migrations` is the single version ledger. Current schema version: 4.

Tables prepared in Sprint 0: `system_snapshots`, `network_snapshots`, `devices`, `alerts`, `security_events`, `settings`, and `integrations`. Integration secrets are not stored in the database; only a future `secret_ref` may be stored.

Sprint 1 adds `process_observations`, `connection_observations`,
`service_observations`, and `telemetry_events`. Command lines are deliberately not
persisted. Live entity heartbeats are batched at approximately 60 seconds, while
factual events are written promptly. Full Sprint 0 system/network snapshots are
limited to one write per five minutes.

Sprint 1.1 migration 0003 separates executable company/signature/signer facts,
persists connection association state and recent process timestamps, and versions
event collector/payload semantics without removing existing observations.

Sprint 2A migration 0004 adds append-only baseline metadata/fact tables and extends the
existing `security_events` foundation with entity identity, evidence, baseline context,
workflow state, deduplication counts, activity, and schema version. Baseline lifecycle and
fact writes are transactional. Previous baseline versions remain stored and inactive.

## Behavioral baseline and security events

The BaselineEngine runs after factual tracking. Live facts are sampled for baseline work no
more often than every 10 seconds; network configuration is sampled at the existing 15-second
system cadence. During Learning, facts are accumulated without emitting novelty events.
Ready comparisons process keys and deltas against the immutable active baseline.

Baseline states are `Not initialized`, `Learning`, `Ready`, `Stale`, and `Error`. Learning
state survives crashes because timestamps, counts, and facts are persisted in the same
transaction. Host identity is an opaque SHA-256 derivation of MachineGuid and system-volume
serial; raw values and hostname are not stored in baseline metadata.

Factual security events have no severity. Their stable key is baseline + event type +
entity key. Repeated observations update one event. A resolved inactive condition reopens
under the same ID when it reappears; ignored events stay ignored. Security Score remains
pending until a future evidence-calibrated detection engine exists.

Initial retention is seven days for inactive observations and full snapshots, and
30 days for factual telemetry events. Cleanup runs at startup and then at most once
every six hours; active rows are never removed. `VACUUM` is not run in the live path.

Inactive resolved/ignored security events use 90-day retention; any other inactive
security event is bounded to 180 days. Active conditions are preserved.

## Live cadence and pause semantics

The React `TelemetryProvider` is the only UI hydration loop and prevents overlapping
requests. The managed Rust engine samples processes on each 2.5-second hydration,
connections no more often than every four seconds, and services no more often than
every 15 seconds. Overview/system collection runs every 15 seconds. Expensive static
executable metadata is cached by path and last-write time.

Pause freezes automatic collection at the latest snapshot while keeping the UI,
filters, sorting, and drawers navigable. Resume performs the next normal collection.
Manual refresh remains available in both states.

## Planned module seams

Software inventory, device discovery, detection, baseline, vulnerability, threat
intelligence, alerts, reports, and integrations remain architectural seams only.
Empty fake implementations are deliberately absent.

## ADRs

- `docs/adr/0001-modular-local-first.md`
- `docs/adr/0002-real-data-partial-results.md`
- `docs/adr/0003-sqlite-and-secret-boundary.md`
- `docs/adr/0004-live-telemetry-and-retention.md`
- `docs/adr/0005-telemetry-accuracy-semantics.md`
- `docs/adr/0006-behavioral-baseline-and-factual-events.md`
