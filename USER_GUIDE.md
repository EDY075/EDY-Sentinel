# EDY Sentinel v1 user guide

This guide covers technical release candidate `1.0.0-rc.1`. The candidate is unsigned and intended
for controlled validation, not public distribution.

## Install and first launch

Use either the MSI or NSIS setup supplied by the release operator. Windows may warn about an
unsigned publisher for this RC; that warning is expected and is a blocker for public release, not a
reason to bypass organizational policy.

EDY Sentinel runs as the current user. On first launch it creates `sentinel.db` under
`%APPDATA%\com.edy.sentinel`. The reconstructible `vulnerability-cache.db` is created when the
provider repository is initialized or synchronized. No database is stored beside the executable.

Initial interface language is selected in this order: saved preference, Windows user locale, then
English. Theme defaults to Sentinel Blue until changed. Both preferences persist locally.

## Understand the first-run states

- **Collectors:** each module reports healthy, degraded, unavailable, or error independently.
  Restricted metadata is shown as restricted rather than guessed.
- **Behavioral baseline:** starts in Learning. Detections that require a Ready baseline fail closed
  until enough factual observations have been learned.
- **Vulnerability Intelligence:** requires installed-software inventory plus locally synchronized
  NVD/CISA data. Missing cache data limits this module without blocking endpoint telemetry.
- **Security Score:** may be unavailable or limited during baseline learning, incomplete collector
  coverage, or pending vulnerability evaluation. The UI explains the missing coverage; it does not
  display a fabricated warm-up value.

## Main views

### Overview

Review collector status, Windows/system facts, behavioral baseline, and Security Score. Use Refresh
for a manual update. Pause freezes automatic collection without disabling navigation.

### Processes, Network, and Services

These read-only views show current Windows observations. Search, filter, sort, and open detail
drawers for evidence. Company metadata and cryptographic signer identity are separate. An unknown or
restricted signature state never means unsigned.

### Inventory and CVE details

Inventory reads the accessible Windows uninstall Registry views. Product Identity shows original
and normalized values, resolution method, canonical CPE, confidence, and an unresolved reason when
applicable. A CVE is Confirmed only when high-confidence identity, version range, and NVD
applicability all succeed. Possible has zero score impact. CISA KEV is prioritization context after
matching; it never confirms the product-to-CVE relationship.

### Security analysis

Security Events are factual observations without severity. Detections are separate conclusions from
six versioned rules and retain evidence, severity, confidence, deduplication, reopen, and workflow
history. EDY Sentinel does not automatically terminate a process, modify a service, or block a
connection.

### Security Score

Formula v2 begins at 100, subtracts the grouped Detection contribution and the bounded confirmed
vulnerability contribution, then clamps the result to 0–100. The breakdown lists each contribution,
formula version, coverage, evidence versions, and calculation time. The score is a posture indicator,
not a guarantee that the endpoint is secure.

### Settings

Switch English/Português (Brasil) at runtime, review provider status, and confirm installed product
version and channel. Theme selection is available from the application shell.

## Offline use

System telemetry, Inventory, baseline, Security Events, Detections, and locally stored scores remain
available without Internet access. Existing vulnerability evidence and a valid local cache remain
usable. Only NVD/CISA synchronization requires external HTTPS access; a failed sync preserves the
last successful local generation.

## Local data and privacy

The main database contains sensitive endpoint metadata including system, software, process,
service, network, baseline, event, Detection, and score history. Keep it protected as user data.
Process command lines are not persisted, file contents are not collected, and endpoint inventory is
not uploaded to NVD or CISA.

Uninstall preserves `sentinel.db`, preferences, `vulnerability-cache.db`, and future local logs by
default. The interactive NSIS uninstaller includes an unchecked `Delete app data` option; select it
only after explicit user intent and a backup. Silent NSIS and MSI uninstall preserve the data.
Back up `%APPDATA%\com.edy.sentinel` before migration or device replacement.

## Troubleshooting

- If a collector is unavailable, review its status and Windows access restrictions; do not infer a
  stopped service or safe process from missing evidence.
- If vulnerability data is unavailable, retry provider status after connectivity returns. The
  primary telemetry application should still open with a missing or inaccessible cache.
- If Security Score is limited, open its coverage explanation and resolve the listed baseline,
  collector, identity, or evaluation prerequisites.
- Preserve databases before reporting a reproducible migration issue. Do not attach real endpoint
  data, credentials, or screenshots to a public report.
