# ADR 0004: Managed live telemetry and bounded local retention

- Status: Accepted
- Date: 2026-08-16

## Context

Sprint 1 must observe hundreds of Windows processes, network endpoints, and services
without freezing the WebView, running PowerShell in a fast loop, inventing facts, or
growing SQLite without bounds. Each screen owning a polling loop would duplicate
work and make tracking and diff semantics inconsistent.

## Decision

Use one frontend `TelemetryProvider` and one Tauri `get_live_telemetry` hydration
command. The command executes on Tauri's blocking worker pool and delegates to one
managed Rust `TelemetryEngine`. The engine owns persistent process sampler state,
collector cadences, last-good snapshots, PID correlation, first/last seen values,
observation counts, and factual diffs.

Initial cadences are:

- UI/process hydration: 2.5 seconds, non-overlapping;
- TCP/UDP owner-PID tables: 4 seconds;
- Windows services: 15 seconds;
- system overview: 15 seconds;
- live SQLite heartbeat: 60 seconds, with immediate factual event writes;
- full system/network snapshot persistence: 5 minutes;
- retention cleanup: startup and at most every 6 hours.

Windows data sources are native or in-process: sysinfo plus Toolhelp/WinTrust for
processes, IP Helper for TCP/UDP, and the Service Control Manager for services. WMI
remains limited to the existing slow hardware data where needed. No PowerShell or
localized command output is used by live collectors.

The first successful snapshot establishes a baseline and emits no start/open events.
A partial protocol-family failure retains its last-good state and cannot produce
mass closure events. Recently inactive processes/connections remain visible for one
collector cycle so details can show the factual ended state.

SQLite migration 0002 stores normalized observation state and factual events. It
does not persist process command lines. Inactive observations and full snapshots are
retained for seven days; events are retained for 30 days. Active observations are
never deleted by retention.

## Consequences

- All operational screens read one coherent snapshot and cannot create independent
  collector loops.
- Process CPU has a warm-up state because it depends on a real delta between samples.
- Service and connection data can be slightly older than process metrics by design.
- Pause retains the last UI snapshot and stops automatic hydrations; resume collects
  again without discarding the retained view.
- Static executable metadata/signature checks are cached and hashes remain absent
  from continuous collection.
- SQLite writes are bounded and transactional, at the cost of observation heartbeats
  being durable at minute-level granularity rather than every UI refresh.

## Rejected alternatives

- One polling loop per screen: duplicates collection and breaks coherent diffs.
- PowerShell or WMI in the fast path: too expensive and sensitive to localization.
- Full JSON list snapshots every refresh: unnecessary write amplification and growth.
- Continuous executable hashing: unacceptable I/O cost for an observation UI.
- Risk labels inferred from missing publisher/signature data: not evidence-based.
