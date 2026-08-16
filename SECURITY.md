# Security

## Sprint 0 posture

- Runs as the current user; administrator rights are not requested.
- Uses a narrow Tauri capability set (`core:default`) and a restrictive content security policy.
- Exposes no arbitrary SQL, WMI, Registry, shell, path, or command-execution IPC.
- Validates the theme identifier in Rust before persistence.
- Stores operational data only under the application-data directory.
- Makes no outbound service call and includes no analytics.
- Contains no required secret, API key, or credential.

## Future integration credentials

Secrets must be stored in Windows Credential Manager. SQLite may contain only a non-secret reference. Secrets must never enter React state, logs, `.env`, command-line arguments, snapshots, or Git.

## Data classification

Snapshots contain local host, user, hardware, storage, and network configuration. Treat the database as sensitive endpoint metadata. Reports and exports need explicit consent and redaction in a future sprint.

## Reporting a vulnerability

Do not include real credentials, personal data, or sensitive host information in a public report. Provide reproduction steps against synthetic data and state the affected version.

## Pre-commit audit

Run the repository secret/path scan described in `DEVELOPMENT.md`, review dependency advisories, and ensure databases, logs, build outputs, and environment files remain ignored.
