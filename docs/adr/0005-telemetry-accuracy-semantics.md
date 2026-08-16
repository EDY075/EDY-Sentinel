# ADR 0005: Telemetry accuracy, coverage, and identity semantics

- Status: Accepted
- Date: 2026-08-16

## Context

Operational telemetry can be technically collected yet still be misleading. A
process CPU aggregate may exceed 100% on a multi-core host; access-denied metadata
does not mean a collector failed; a PID can be reused; adapter enumeration order does
not identify the active route; CompanyName is not cryptographic signer identity; and
a single missed connection sample is insufficient evidence of closure.

Sprint 1.1 must make these distinctions explicit without adding scoring, external
reputation, blocking actions, or other Sprint 2 behavior.

## Decision

1. Process CPU shown in the main table is the sampler aggregate divided by the
   logical-processor count and clamped to 0–100%. The original aggregate is retained
   as nullable `coreEquivalentCpuPercent` and labeled as core-equivalent diagnostic
   data. Both values are null during warm-up.
2. Collector health describes execution: `healthy`, `degraded`, or `failed`.
   Coverage describes observations that are restricted or incomplete. A successful
   collector with restricted process metadata remains healthy and reports a separate
   `restrictedCount`.
3. Each collector reports last attempt, last success, duration, observation count,
   restriction count, and native failure diagnostics when applicable. Cached cadence
   responses retain the true collector timestamps rather than the UI hydration time.
4. Connection association uses the current process identity (PID plus creation
   identity) and a bounded ten-second recently-exited cache. A five-second reuse guard
   prevents attribution when a PID identity changes. Results are explicit:
   `associated`, `recently_exited`, `unresolved`, `system_kernel`, or
   `not_applicable`.
5. A connection close event requires absence from two consecutive successful
   snapshots. A failed protocol family keeps its last-good baseline and cannot emit
   mass closure events.
6. The primary interface is selected with the native IP Helper `GetBestRoute2`
   result for a public destination. Source address, gateway, route/interface metric,
   interface type, and classification source are exposed. Enumeration order is never
   used as routing evidence.
7. Executable CompanyName, signature verification status, and certificate signer are
   distinct fields. Verification uses local WinVerifyTrust/CryptoAPI calls with a
   bounded per-cycle enrichment budget and cache invalidation by path, last-write
   time, and file size. No network reputation lookup or continuous hashing is used.
8. Factual events carry `eventId`, type, entity type/key, timestamp, collector,
   factual payload, schema version, and human-readable message. Tracking is
   monotonic, counts each unique identity once per cycle, and does not assign risk.
9. SQLite schema v3 stores the added factual fields. Command lines remain live-only.
   Retention remains seven days for inactive observations/full snapshots, 30 days for
   events, and cleanup no more often than every six hours; active rows are preserved.

## Consequences

- Values in the UI have unambiguous units and warm-up behavior.
- Coverage limitations remain visible without falsely presenting an execution fault.
- Connection correlation can intentionally remain unresolved instead of guessing.
- Signed executables can show a certificate subject while unsigned, inaccessible, or
  indeterminate files retain honest states.
- Network route facts reflect Windows routing decisions at collection time.
- A close event is slightly delayed by one successful sample in exchange for fewer
  false transitions.
- Cache invalidation and enrichment budgets keep native trust work off the hot path.

## Rejected alternatives

- Showing the raw multi-core aggregate as percent: the unit is misleading to users.
- Marking every access-denied record as degraded: confuses coverage with execution.
- Correlating by PID only: vulnerable to stale association after PID reuse.
- Choosing the first/up adapter: enumeration order is not route preference.
- Treating CompanyName as publisher/signer: version metadata is not trust evidence.
- Emitting close on one absence: transient misses become false events.
- VirusTotal or continuous hashing in the live loop: outside scope, privacy-sensitive,
  and unnecessarily expensive.
