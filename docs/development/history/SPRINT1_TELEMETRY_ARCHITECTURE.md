# Sprint 1 telemetry architecture

The Sprint 1 collector is a local, read-only Windows telemetry engine. The React layer invokes one
typed `get_live_telemetry` command; collection and SQLite work run on a blocking worker rather than
the WebView thread. No command string or shell input crosses the IPC boundary.

## Native sources and cadence

- Processes: persistent `sysinfo::System` snapshot refreshed on each live cycle. CPU is unavailable
  during the first delta warm-up instead of being reported as a factual zero. A Toolhelp snapshot
  supplies thread counts and `IsWow64Process2` supplies architecture where access is allowed.
- Connections: IP Helper `GetExtendedTcpTable` and `GetExtendedUdpTable`, independently for TCP and
  UDP over IPv4 and IPv6, at most every four seconds. Correlation uses the owning PID from Windows.
- Services: read-only Service Control Manager enumeration and configuration queries at most every
  fifteen seconds. There are no service-control operations in the IPC contract.
- System Overview: its existing collector remains available and SQLite persistence is throttled to
  one full snapshot every five minutes.

Executable FileDescription and CompanyName version resources and WinVerifyTrust results are cached
by path and file modification time. Enrichment is budgeted to 24 new executables per cycle to avoid
large initial stalls. CompanyName is descriptive version metadata, not the verified signer identity.
Signature verification is cache-only and does not perform network retrieval. Executable hashing is
not part of continuous collection.

## Tracking and partial failures

Process identity is PID plus creation time. Connection identity is protocol, address family, local
and remote tuple, and owning PID. Service identity is its case-normalized SCM name. The first good
snapshot establishes a baseline without generating synthetic start/open events. Closed or stopped
observations remain present as `active=false` for a cycle and in SQLite for the retention window.

Each TCP/UDP address family has an independent last-good baseline. If one native table fails, the
engine retains and exposes that family's previous values, marks the collector partial, and does not
manufacture mass connection-closed events. The same principle applies to total process or service
collector failure.

## SQLite and privacy

Migration `0002_live_telemetry.sql` adds current observation tables and factual telemetry events.
Observation writes occur at most once per minute, except that non-empty event batches are persisted
immediately. Cleanup runs at most every six hours. Inactive observations and full system snapshots
are retained for seven days; factual events are retained for thirty days.

Process command lines are intentionally returned only to the live local UI and are never stored in
SQLite or event messages because command arguments can contain secrets. No API key, reputation,
geolocation, inferred risk, kill/control action, or arbitrary shell capability is present.
