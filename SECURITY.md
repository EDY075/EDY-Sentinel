# Security

## Sprint 1.1 posture

- Runs as the current user; administrator rights are not requested.
- Uses a narrow Tauri capability set (`core:default`) and a restrictive content security policy.
- Exposes no arbitrary SQL, WMI, Registry, shell, path, or command-execution IPC.
- Validates the theme identifier in Rust before persistence.
- Stores operational data only under the application-data directory.
- Makes no outbound service call and includes no analytics.
- Contains no required secret, API key, or credential.
- Uses native IP Helper and Service Control Manager reads; it does not invoke
  PowerShell or construct shell commands from UI input.
- Runs live collection on Tauri blocking workers and treats process access denial as
  a partial record rather than requesting elevation.
- Performs executable signature verification and signer extraction with local
  WinVerifyTrust/CryptoAPI calls, bounded enrichment, file-change-aware caching, and
  no external reputation lookup.
- Does not continuously hash executables and exposes no process, network, firewall,
  or service-control command.
- Treats CompanyName as version-resource metadata rather than proof of authorship,
  and never derives a risk label from absent company, signer, or signature data.
- Protects connection attribution with process creation identity, a bounded
  recently-exited cache, and a PID-reuse guard.

## Sprint 2A baseline and event posture

- Baseline learning and comparison remain fully local; no host, process, network, service,
  event, or identifier data leaves the device.
- Baseline host identity is an opaque SHA-256 derivation of MachineGuid and system-volume
  serial resolved from the actual Windows installation volume; no fixed drive letter is
  assumed. Raw values and hostname are not stored in baseline metadata.
- Executable identity uses normalized path and cached file/signature metadata. File content
  is not continuously hashed. Any future content SHA-256 must run on demand or as bounded,
  cached background work rather than on every refresh.
- CompanyName remains descriptive version-resource metadata and is never substituted for
  cryptographic signer identity.
- Service PID remains live runtime telemetry and is not part of persistent service identity;
  current PID may appear only as factual event evidence when available.
- A failed Service Control Manager enumeration retains the last-good state and is reported
  separately from a factual `Stopped` service or restricted service configuration.
- Network destinations are dynamic. A new endpoint, unusual port, or missing process
  correlation is not independently a threat verdict or sufficient evidence for severity.
- Learning emits no new-behavior events. Ready comparisons create factual observations,
  never malware claims or severity labels.
- The one-minute learning option is for controlled development/testing only, and its events
  are not a production calibration dataset.
- Schema-v5 hardening persists `Error` only when an already stored baseline fails in a way
  that must survive restart. Only a stable code, sanitized message, and update timestamp are
  stored; transient pre-baseline failures create no baseline, and stack traces or sensitive
  details are never persisted.
- Reset/relearn requires an exact confirmation phrase, preserves previous baseline versions,
  and does not remove telemetry/event history outside the active baseline.
- Event workflow changes are allowlisted and parameterized. SQL identifiers used internally
  are compile-time constants; user input never becomes SQL syntax.
- No shell, arbitrary path, cloud telemetry, external API, firewall, process termination,
  or service-control capability was added.

## Sprint 2B detection and score posture

- Factual Security Events and rule-produced Detections are separate tables, models, APIs, and
  UI views. Facts retain no severity; React never decides whether a fact is a Detection.
- The Detection Engine consumes append-only event-history deltas through a durable checkpoint.
  Migration v6 seeds the checkpoint at cutover, so installing a new rule does not silently
  reinterpret pre-v6 history.
- Rule evaluation runs in an analysis transaction independent from the Baseline Engine. An
  analysis failure retries from its checkpoint and cannot persist a false baseline Error.
- Every Detection stores an immutable rule ID/version and source-event evidence. Foreign keys
  use restrictive history-preserving behavior; factual retention skips referenced evidence.
- Rule IDs, versions, statuses, severities, confidence, limits, cursors, filters, and enabled
  changes cross typed/validated IPC boundaries. SQL values are bound; no UI string becomes SQL
  syntax and the UI cannot edit arbitrary rule conditions.
- The six v1 rules require corroborating evidence and are capped at Medium. Signature states
  `unknown` and `restricted` are never treated as unsigned; loopback, unspecified, multicast,
  unavailable collectors, incomplete service configuration, ambiguous process correlation,
  and non-Ready baselines fail closed.
- Confidence means evidence/correlation quality, not malware probability. Severity explains the
  corroborated facts and never comes from novelty, signer absence, port, or destination alone.
- Security Score formula v1 is not a percentage guarantee. It requires a Ready baseline,
  enabled rules, and measured system/process/connection/service coverage. Failed or incomplete
  coverage produces no number; degraded coverage is explicitly Limited.
- Score inputs are active Detections only. Resolved and Ignored records preserve history but do
  not penalize current posture; Acknowledged remains relevant while its condition is active.
- Controlled smoke uses a benign test helper that only sleeps. It performs no malware behavior,
  network action, service mutation, persistence, privilege change, or automated remediation.
- Local screenshots and the SQLite database contain sensitive endpoint metadata and stay outside
  Git. The release contains no fixture data, database, screenshot, dump, API key, or credential.

Known dependency-audit warnings are transitive: GTK3/ATK/GDK and `glib` are outside the Windows
runtime graph; `proc-macro-error` and the `unic` family are unmaintained transitives. No applicable
Rust or npm vulnerability was reported in the final audit, but these upgrade debts remain visible.

## Future integration credentials

Secrets must be stored in Windows Credential Manager. SQLite may contain only a non-secret reference. Secrets must never enter React state, logs, `.env`, command-line arguments, snapshots, or Git.

## Sprint 3 Part 1 inventory and provider posture

- Installed software is read directly from the current user's accessible HKLM/HKCU uninstall
  Registry views. No PowerShell, shell command, elevation, uninstall string execution, or system
  mutation is used.
- Missing Registry fields remain absent. Similar display names are not merged, and an unresolved
  normalized identity is never guessed into a product or CVE.
- Software installed/removed/version-changed records are facts without severity. They do not
  affect Detection rules or Security Score in Part 1.
- NVD and CISA KEV are isolated HTTPS-only download providers. Endpoint inventory, paths, users,
  host identity, and telemetry are never sent to either service.
- External responses have fixed size/record/string bounds and validated CVE/date/pagination data.
  Retries, backoff, public rate limits, timeouts, cooperative cancellation, and sanitized UI errors
  are enforced in Rust.
- The interface contains no API key. A future NVD key must use Windows Credential Manager and
  remain outside React, SQLite, logs, command lines, reports, and Git.
- Repository writes are transactional and independent from baseline/detection transactions. An
  external outage preserves existing cache rows and cannot poison the behavioral baseline.
- KEV membership is exploitation context only; it does not independently imply that local
  software is affected or that severity is Critical.
- NVD and CISA KEV provider transport is HTTPS-only. Stored external advisory references are also
  limited at ingestion and presentation to credential-free HTTPS URLs; older plaintext HTTP cache
  records are not rendered as links.
- The reconstructible vulnerability cache cannot abort application startup. Interrupted provider
  recovery is best-effort, while the cache error code and degraded coverage remain explicit.

## Release artifact trust

The Sprint 4A technical artifacts are reproducible EXE, MSI, and NSIS outputs, but they are not
Authenticode-signed. Signing and timestamp verification are mandatory release gates before public
distribution. The MSI is per-machine and may request installation elevation; normal Sentinel
runtime collection continues as the current standard user and does not request administrator rights.

## Data classification

Snapshots contain local host, user, hardware, storage, process identity, executable
paths, service configuration, and network endpoints. Treat the database as sensitive
endpoint metadata. Process command lines are not persisted. Reports and exports need
explicit consent and redaction in a future sprint.

## Reporting a vulnerability

Do not include real credentials, personal data, or sensitive host information in a public report. Provide reproduction steps against synthetic data and state the affected version.

## Pre-commit audit

Run the repository secret/path scan described in `DEVELOPMENT.md`, review dependency advisories, and ensure databases, logs, build outputs, and environment files remain ignored.
