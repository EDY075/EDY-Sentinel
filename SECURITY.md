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
