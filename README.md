# EDY Sentinel

EDY Sentinel is a local-first Windows endpoint intelligence desktop application. Its compact
desktop shell centers the observed device, local posture, software identity, processes,
connections, vulnerabilities, and explainable risk. It combines native endpoint telemetry, a
behavioral baseline, factual security events, six explainable detection rules, conservative
software-to-CVE matching, and an auditable Security Score.

Version `1.0.0` is the final release tree. Its EXE, MSI, and NSIS outputs are explicitly classified
as `UNSIGNED BUILD`: they are not authenticated publisher artifacts. Any distribution of these
files must preserve that visible disclosure; a future signed release remains subject to the
fail-closed Authenticode and trusted-timestamp verification gate.

## What the application does

- Collects real Windows system, process, TCP/UDP connection, service, route, adapter, and DNS facts.
- Maintains a versioned behavioral baseline with explicit Learning, Ready, Stale, and Error states.
- Stores factual Security Events separately from rule-produced Detections.
- Evaluates six immutable v1 detection rules with evidence, exclusions, deduplication, and history.
- Inventories installed software from native HKLM/HKCU 64-bit and 32-bit Registry views.
- Synchronizes public NVD and CISA KEV data into a rebuildable local cache on user request.
- Resolves product identity and version ranges conservatively; ambiguous evidence fails closed.
- Calculates Security Score formula v2 from active Detections and confirmed vulnerability evidence.
- Supports English and Português (Brasil), plus Sentinel Blue, Cyber Green, Terminal, and Spectrum.

No score, detection, process, connection, software identity, CVE relationship, or collector result
is fabricated. Missing or restricted evidence is shown as unavailable, limited, ambiguous, or
unresolved.

## Core views

- **Overview:** device identity and state first, followed by the recognizable 0–100 Security Score,
  collector health, system facts, vulnerability posture, and baseline state.
- **Processes, Network, and Services:** live read-only endpoint activity with factual detail.
- **Inventory:** a dense endpoint-inspection table for installed software, normalized Product
  Identity, vulnerability state, CVE evidence, and KEV context.
- **Security analysis:** factual events, explainable Detections, and the six-rule registry.
- **Settings:** interface language, installed version/release channel, and provider status.

Side drawers use a shared **Endpoint Inspector** language for identity, runtime, evidence, network,
vulnerability, and technical metadata. Sentinel Blue is the primary neutral-blue desktop theme;
Cyber Green uses green as a functional accent, Terminal is restrained rather than terminal-like,
and Spectrum provides controlled violet/blue depth without changing semantic state colors.

## Screenshots and visual evidence

Real desktop screenshots are generated outside Git because they may contain endpoint metadata. The
Sprint 5C visual pass validates the real Tauri WebView at 1920x1080, 1600x900, 1440x900, 1366x768,
and 1280x720; its local screenshots remain outside the repository. Release/publication status is
tracked separately in `SPRINT5B_FINAL_REPORT.md` and `RELEASE_CHECKLIST.md`.

## Installation status

Version 1.0.0 targets Windows 10/11 x64 with the Microsoft Edge WebView2 Runtime. MSI is intended for a
managed Windows installation; NSIS provides the alternative setup executable. Current uninstallers
remove installer-owned binaries, shortcuts, and registration while preserving the application data
directory by default.

Unsigned installation packages may be distributed only with explicit release approval and a
visible `UNSIGNED BUILD` notice. They must not be represented as signed or publisher-authenticated.
For a signed release, all of the following remain mandatory:

1. EXE, MSI, and NSIS are signed by the legitimate publisher identity;
2. every signature includes a trusted timestamp;
3. `scripts/verify-authenticode.ps1` passes for all three immutable artifacts;
4. clean-install, upgrade, uninstall, and visual acceptance evidence is complete.

Installer metadata keeps the established publisher value `edy`, derived from the stable application
identifier, so existing NSIS installations can be upgraded. This is package identity only and is
not a claim of Authenticode publisher authentication.

See `USER_GUIDE.md` for operation and `CODE_SIGNING.md` for the signing boundary.

## Development and verification

Requirements:

- Node.js 20+ and pnpm 10+;
- stable Rust MSVC toolchain;
- Visual Studio 2022 Build Tools with Desktop development with C++ and a Windows SDK;
- WebView2 Runtime.

```powershell
pnpm install
pnpm tauri dev
```

```powershell
pnpm verify
pnpm check:rust
pnpm release:build
```

An unsigned final local build prints and records `UNSIGNED BUILD`. Setting
`EDY_SENTINEL_REQUIRE_SIGNED_RELEASE=1` turns missing or invalid signing inputs into a hard failure.

## Architecture and security boundaries

React owns presentation. Rust owns Windows collection, validation, detection, scoring, provider
integration, and SQLite persistence. Tauri commands are a narrow typed boundary; the UI cannot run
arbitrary shell, SQL, Registry, WMI, file, or path operations.

The application runs as the current user. It does not block processes, change services, edit the
firewall, remediate vulnerabilities, or claim complete endpoint protection. Detection confidence
describes evidence quality, not malware probability, and Security Score is an explainable posture
indicator rather than a guarantee.

See `ARCHITECTURE.md`, `DETECTION_RULES.md`, `SECURITY_SCORE.md`, and
`VULNERABILITY_MATCHING.md` for the versioned technical contracts.

## Privacy

System telemetry, inventory, baselines, events, Detections, score history, settings, and software
matching evidence remain in local SQLite databases under the Windows application-data directory.
Process command lines are live-only and are not persisted. File contents are not collected or
stored, and executable content is not continuously hashed.

NVD and CISA synchronization downloads public vulnerability records over HTTPS. It does not upload
endpoint inventory or personal telemetry. Version 1.0.0 contains no analytics, cloud account, AI module,
API key, or required external credential.

## Known limitations

- Version 1.0.0 binaries are an `UNSIGNED BUILD`; legitimate Authenticode publisher authentication
  remains unavailable until an appropriate certificate and trusted timestamp are supplied.
- Product identity is intentionally narrow; unresolved software is not guessed into a CPE.
- Offline operation uses already persisted vulnerability evidence/cache; external synchronization
  naturally remains unavailable without network access.
- The six v1 rules are conservative and do not constitute malware classification.
- Active response, reputation services, advanced network scanning, and native Windows notifications
  are outside v1 scope.

## Project documents

- `USER_GUIDE.md` — installation, first run, views, offline use, and data preservation.
- `RELEASE_NOTES_v1.0.0.md` — v1 capabilities, requirements, limitations, and signing status.
- `RELEASE_CHECKLIST.md` — technical and public-release gates.
- `DEVELOPMENT.md` — toolchain and contributor workflow.
- `LOCALIZATION.md` — bilingual presentation contract.
- `SECURITY.md` — threat boundaries, privacy, and reporting guidance.
- `ROADMAP.md` and `CHANGELOG.md` — status and version history.
- `SPRINT5B_FINAL_REPORT.md` — final local preparation evidence and remaining public gates.
