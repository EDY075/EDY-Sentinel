# ADR 0006: Versioned behavioral baseline and factual security events

- Status: Accepted
- Date: 2026-08-16

## Context

Sprint 1.1 provides reliable local observations and factual telemetry transitions.
Sprint 2A needs to answer what is normally present, what is new, and what changed
without treating novelty as maliciousness or calculating a Security Score.

A learning period must survive application restarts, avoid a flood of “new” events,
remain bound to the correct Windows installation, and preserve prior baseline versions.
The live telemetry path must remain responsive.

## Decision

Use a local Rust `BaselineEngine` after the existing tracking layer:

```text
Windows collectors → TelemetryEngine tracking → BaselineEngine → Security events
                                            ↘ SQLite schema v4
```

The engine observes live process/connection/service state at most every 10 seconds and
network configuration at most every 15 seconds. During `Learning`, it stores only real
facts and creates no new-behavior events. A baseline becomes `Ready` after its configured
UTC learning period and at least one observation, or after an explicit controlled manual
completion. `Learning` state, timestamps, counts, and facts are transactional and recover
after restart. A Ready baseline becomes `Stale` when it has no successful observation for
24 hours and returns to Ready after a new observation.

Each baseline has a unique ID, monotonically increasing host-local version, opaque host
identity, timestamps, state, observation count, learning period, schema version, and an
`active` marker. Starting or resetting creates a new version and preserves prior versions.
Host identity is a SHA-256-derived opaque value from Windows MachineGuid plus the system
volume serial; neither raw identifier nor hostname is stored in baseline metadata.

Executable identity is a SHA-256 key over normalized path, file size, file modification
time, signer, signature state, and architecture. It hashes identity metadata, not file
contents. No continuous executable content hashing is added.

Schema v4 stores executable facts, process patterns, parent-child relationships, remote
destinations, services, and network configuration. Network facts include the Windows best
route identity, address, gateway, DNS set, interface type, route metric, and serialized
interface inventory.

Security events are factual comparison records with stable ID, event/entity identity,
title, UTC timestamps, evidence JSON, baseline context, source collector, optional rule
and factual-correlation confidence, local workflow status, observation count, active
condition, and schema version. No severity field is introduced for baseline events.

The same baseline/event/entity key upserts one event and updates `lastSeen` and
`observationCount`. When a condition disappears it becomes inactive. If a resolved event
later reappears, the same event ID reopens as `New`; ignored events remain ignored. This
prevents refresh floods while preserving continuity.

Inactive resolved/ignored events are retained for 90 days. All inactive factual security
events are bounded to 180 days. Active conditions are preserved. Existing raw telemetry
retention remains unchanged.

## Consequences

- Learning does not produce hundreds of novelty events.
- A baseline restart cannot silently become Ready or lose its facts.
- New process, executable, destination, route, gateway, DNS, or service facts remain
  observations, not threats.
- Event evidence is locally verifiable and supports a later rule engine without defining
  its severity or Security Score early.
- SQLite work is delta-oriented and cadence-limited rather than performed at every 2.5 s
  UI hydration.
- Manual completion is available for controlled development tests and explicitly labeled
  as such in the UI.

## Rejected alternatives

- Create events while Learning: produces guaranteed false novelty floods.
- Identify a host by hostname: unstable and not unique.
- Identify executables by filename: conflates unrelated files.
- Hash every executable continuously: unnecessary I/O and privacy cost.
- Overwrite the current baseline: destroys audit history and makes relearn ambiguous.
- Assign severity or a numeric score to novelty: unsupported by factual evidence alone.
- Use cloud reputation or threat intelligence: outside Sprint 2A and local-first scope.
