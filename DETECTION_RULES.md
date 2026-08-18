# EDY Sentinel Detection Rules

Final v1 contract: application `1.0.0`, registry version `v1`, exactly six built-in
rules. Packaging and documentation changes do not introduce test rules or change triggers,
exclusions, deduplication, reopen behavior, severity, confidence, or evidence requirements.

## Rule policy

The built-in Sprint 2B registry contains exactly six enabled, versioned rules. Rules consume
factual Security Events and baseline context; they do not reinterpret observations as malware
or threats. Every persisted Detection must retain its `rule_id` and `rule_version`.

Common gates and semantics:

- the active baseline must be `Ready`;
- all required evidence must be available within the documented correlation window;
- any matching exclusion prevents a Detection;
- `unknown` and `restricted` signature states never mean `unsigned`;
- confidence describes evidence availability and correlation quality, not malware probability;
- novelty, an unsigned state, or a user-writable location alone is insufficient;
- no v1 rule assigns High or Critical severity;
- no rule executes a command or performs an automatic response.

For overlapping executable findings, higher precedence wins:

`EDY-NET-001 → EDY-PROC-002 → EDY-PROC-001`

All three share `executable:{entityKey}` as their score group so one factual cluster cannot be
penalized repeatedly.

## EDY-PROC-001 v1

**Objective:** identify an actually executed, new unsigned binary under a temporary
user-writable path.

- Category: `process_execution`
- Enabled: yes
- Window: 60 seconds
- Default severity: Low
- Default confidence: High
- Precedence: 100
- Score group: `executable:{entityKey}`
- Required evidence: one `executable_first_seen` and one `process_first_seen`, joined by the
  normalized executable path.
- Positive conditions: signature state is exactly `unsigned`; path class is
  `user_writable_temp`; the process event confirms execution.
- Exclusions: baseline not Ready; signature signed/unknown/restricted; path missing, ambiguous,
  outside the class, or already known under the same normalized path; ambiguous process match.
- False-positive context: installers, updaters, and development tools may execute legitimate
  unsigned temporary helpers.
- Recommended action: verify origin, launching application, signature, and recent installation
  activity. Do not delete the file automatically.

## EDY-PROC-002 v1

**Objective:** add parent-child context when a known signed parent launches a new unsigned
temporary child through a relationship absent from the baseline.

- Category: `process_execution`
- Enabled: yes
- Window: 60 seconds
- Default severity: Medium
- Default confidence: High
- Precedence: 200
- Supersedes: `EDY-PROC-001`
- Score group: `executable:{entityKey}`
- Required evidence: `executable_first_seen`, `process_first_seen`, and
  `parent_child_first_seen`, joined by normalized child path; the parent must resolve to a
  signed executable in the baseline.
- Exclusions: all PROC-001 exclusions; parent unavailable/not signed; an eligible outbound
  correlation exists, in which case NET-001 is more specific.
- False-positive context: signed installers, browsers, and enterprise deployment software may
  launch an unsigned temporary helper.
- Recommended action: confirm whether the parent was performing an expected install/update and
  compare the child path and signature state with recent user activity.

## EDY-NET-001 v1

**Objective:** identify first-seen outbound activity correlated with a newly executed unsigned
temporary binary.

- Category: `network_activity`
- Enabled: yes
- Window: 60 seconds
- Default severity: Medium
- Default confidence: Medium
- Precedence: 300
- Supersedes: `EDY-PROC-002`, `EDY-PROC-001`
- Score group: `executable:{entityKey}`
- Required evidence: `executable_first_seen`, `process_first_seen`, and
  `destination_first_seen`.
- Positive conditions: unsigned temporary executable; destination association is `associated`;
  remote address is valid remote unicast; process-name bridge has exactly one candidate.
- Exclusions: signature signed/unknown/restricted; unresolved, recently exited, kernel, or
  not-applicable association; loopback/unspecified/multicast address; ambiguous process match.
- False-positive context: legitimate installers/updaters may download content, and a new
  private-network destination may be an expected local service.
- Recommended action: review executable origin, parent, remote address, port, and recent install
  activity. Confirm the destination before taking any network action.

Confidence remains Medium despite the v2 destination evidence carrying a correlated executable
key: there is no external reputation, content identity, or independent destination trust signal.
The association is sufficient for this Medium rule, but it is not evidence of malicious intent.

## EDY-SVC-001 v1

**Objective:** identify a new running automatic service that combines a privileged account with
a binary under a user-writable path.

- Category: `service_persistence`
- Enabled: yes
- Window: 30 seconds
- Default severity: Medium
- Default confidence: High
- Precedence: 200
- Score group: `service:{entityKey}`
- Required evidence: one `service_first_seen` event.
- Positive conditions: Running; Automatic or Automatic Delayed; LocalSystem or
  `NT AUTHORITY\SYSTEM`; unambiguously parsed user-writable binary path.
- Exclusions: collector failed; configuration unavailable; Manual/Disabled startup; missing or
  ambiguous binary path.
- False-positive context: vendor updaters and enterprise management products may legitimately
  install a privileged automatic service.
- Recommended action: verify software origin, binary path, account, and recent installation.
  Check signature separately because service evidence does not currently carry signer metadata.

## EDY-SVC-002 v1

**Objective:** identify a known service whose binary path changed together with its startup type
or account.

- Category: `service_persistence`
- Enabled: yes
- Window: 45 seconds
- Default severity: Medium
- Default confidence: High
- Precedence: 200
- Score group: `service:{entityKey}`
- Required evidence: one `service_binary_changed` plus either one
  `service_startup_changed` or one `service_account_changed`, joined by service name.
- Exclusions: isolated field change; different services; missing after value; service collector
  failed.
- False-positive context: approved upgrades, endpoint security, and device-management tools may
  change multiple service fields during maintenance.
- Recommended action: compare all before/after values with an approved change and verify the new
  binary and account privileges. Do not modify the service automatically.

## EDY-NET-002 v1

**Objective:** identify a coordinated gateway and DNS change that persists beyond a transient
network cycle.

- Category: `network_configuration`
- Enabled: yes
- Window: 30 seconds and at least two successful observations
- Default severity: Low
- Default confidence: High
- Precedence: 100
- Score group: `network_configuration:{baselineId}`
- Required evidence: one `gateway_changed` and one `dns_changed` from the same baseline;
  `primary_route_changed` is retained as supporting evidence when present.
- Exclusions: system collector degraded/failed; missing before/after values; isolated change;
  reversion before the second observation.
- False-positive context: VPN activation, Wi-Fi roaming, DHCP renewal, router replacement, and
  managed corporate network changes commonly alter gateway and DNS together.
- Recommended action: confirm expected VPN/Wi-Fi/DHCP/router activity and compare gateway,
  resolver, interface, and route values. This rule never promotes such a change to High.

## Deferred rules

### EDY-PROC-003 — executable identity change

Deferred from the enabled v1 registry. The current executable identity includes path, size,
mtime, signer, signature state, and architecture, but the live enrichment budget can yield
`unknown` metadata and there is no content SHA-256 in the live path. Implementing a generic
identity-change rule now could mistake normal updates or incomplete enrichment for tampering.

A future version may be justified only with bounded/cached enrichment and explicit before/after
evidence for an exact normalized path. Size/mtime changes alone must never trigger it, signer
rotation must remain low priority, and `signed → unsigned` must require complete evidence.

Also deferred:

- unsigned service binary rules, because service facts do not include signature metadata;
- command-line or LOLBin rules, because command lines remain live-only and are not factual event
  evidence;
- suspicious port/IP rules without reputation or production-calibrated context;
- isolated service removal and High/Critical classifications without corroborating evidence.

## Calibration and fixture coverage

The Sprint 2B suite gives every enabled rule a positive fixture plus negative/missing/partial
evidence coverage appropriate to that rule. Cross-engine tests cover disabled state, exact rule
version, deduplication, precedence, reopen, status, severity/confidence, immutable evidence, and
cursor pagination. Legitimate fixtures cover signed/unknown executables, ordinary new network
destinations, loopback, isolated service changes, incomplete service metadata, and incomplete or
transient network reconfiguration. No fixture is compiled into the application registry.

The native smoke helper is a benign executable that sleeps and is intentionally unsigned under a
temporary path; it validates the expected PROC-001/PROC-002 path without network or service
mutation. During ordinary host use, 1,371 factual events existed while only the two controlled
process-rule identities were present; no unrelated Detection flood was observed.

This is infrastructure and conservative development calibration, not a production corpus. The
short development baseline and its historical events must not be used to claim detection rates.
Future releases need longer Ready baselines and broader installer/updater/VPN/service-maintenance
fixtures before raising severity or enabling additional rules.
