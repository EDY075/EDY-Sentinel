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
  serial. Raw values and hostname are not stored in baseline metadata.
- Executable identity uses normalized path and cached file/signature metadata. File content
  is not continuously hashed.
- Learning emits no new-behavior events. Ready comparisons create factual observations,
  never malware claims or severity labels.
- Reset/relearn requires an exact confirmation phrase, preserves previous baseline versions,
  and does not remove telemetry/event history outside the active baseline.
- Event workflow changes are allowlisted and parameterized. SQL identifiers used internally
  are compile-time constants; user input never becomes SQL syntax.
- No shell, arbitrary path, cloud telemetry, external API, firewall, process termination,
  or service-control capability was added.

## Future integration credentials

Secrets must be stored in Windows Credential Manager. SQLite may contain only a non-secret reference. Secrets must never enter React state, logs, `.env`, command-line arguments, snapshots, or Git.

## Data classification

Snapshots contain local host, user, hardware, storage, process identity, executable
paths, service configuration, and network endpoints. Treat the database as sensitive
endpoint metadata. Process command lines are not persisted. Reports and exports need
explicit consent and redaction in a future sprint.

## Reporting a vulnerability

Do not include real credentials, personal data, or sensitive host information in a public report. Provide reproduction steps against synthetic data and state the affected version.

## Pre-commit audit

Run the repository secret/path scan described in `DEVELOPMENT.md`, review dependency advisories, and ensure databases, logs, build outputs, and environment files remain ignored.
