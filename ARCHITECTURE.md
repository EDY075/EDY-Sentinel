# Architecture

## Decision summary

EDY Sentinel uses a modular local-first monolith. React owns presentation. Rust owns collection, validation, application logic, and persistence. Tauri commands form the typed trust boundary.

```text
React UI
  -> typed command wrappers
Tauri command boundary
  -> collection orchestration / validation
Rust domain models
  -> Windows adapters (Registry, WMI, IP Helper, sysinfo)
  -> SQLite repository (migrations, snapshots, settings)
```

## Frontend responsibilities

- `src/app`: future provider and shell composition boundary
- `src/features/overview`: real snapshot presentation and formatting
- `src/features/theme`: four token-based theme choices
- `src/components/ui`: reusable badge, status, skeleton, dialog, drawer, tooltip, and chart primitives
- `src/lib/tauri.ts`: the only frontend IPC entry point
- `src/types`: DTO contracts mirrored from serialized Rust types

The frontend cannot execute SQL, WMI, Registry reads, shell commands, or arbitrary file access.

## Rust responsibilities

- `collectors/system.rs`: host, OS, CPU, GPU, memory, disks, uptime
- `collectors/network.rs`: adapters, addresses, gateways, DNS, primary-route selection
- `commands.rs`: narrow Tauri commands and input validation
- `models.rs`: serialization contracts
- `persistence/mod.rs`: SQLite lifecycle, migrations, snapshots, and settings
- `migrations`: append-only schema evolution

Collectors do not know about React. The persistence layer does not accept SQL from IPC. Future engines will consume collector DTOs through application services rather than coupling to Windows APIs.

## Data honesty contract

- Values originate at refresh time from Windows.
- Optional data remains optional; it is never converted to a fabricated zero.
- Collector-specific failure becomes a partial result with an issue message.
- A total orchestration failure becomes an error state.
- Security Score remains `not_implemented` until an explainable engine exists.

## Persistence

SQLite opens from Tauri's `app_data_dir`. Startup enables foreign keys, WAL, and a busy timeout, then applies each migration transactionally. `schema_migrations` is the single version ledger. Current schema version: 1.

Tables prepared in Sprint 0: `system_snapshots`, `network_snapshots`, `devices`, `alerts`, `security_events`, `settings`, and `integrations`. Integration secrets are not stored in the database; only a future `secret_ref` may be stored.

## Planned module seams

Process, connections, services, software inventory, device discovery, detection, baseline, vulnerability, threat intelligence, alerts, reports, and integrations are architectural seams only. Empty fake implementations are deliberately absent.

## ADRs

- `docs/adr/0001-modular-local-first.md`
- `docs/adr/0002-real-data-partial-results.md`
- `docs/adr/0003-sqlite-and-secret-boundary.md`
