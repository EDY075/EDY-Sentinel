# EDY Sentinel 1.0.0

Date: 2026-08-18

Status: final `UNSIGNED BUILD`. The Windows binaries do not carry an Authenticode publisher
signature and must not be represented as signed or publisher-authenticated.

## Highlights

- Real, read-only Windows system, process, TCP/UDP connection, service, route, adapter, DNS, and
  installed-software collection with explicit unavailable/restricted states.
- A versioned behavioral baseline and factual Security Events kept separate from explainable
  rule-produced Detections.
- Detection Engine v1 with exactly six enabled rules, immutable rule provenance, bounded
  correlation, exclusions, evidence, deduplication, reopen behavior, and workflow history.
- Security Score formula v2 with separate Detection and confirmed-vulnerability contributions,
  explicit coverage, immutable formula-versioned history, and no penalty for Possible matches.
- Conservative Product Identity/CPE resolution and NVD applicability evaluation with CISA KEV as
  post-match prioritization context only.
- English and Português (Brasil), Sentinel Blue, Cyber Green, Terminal, and Spectrum themes,
  keyboard/focus support, reduced-motion styles, and responsive desktop layouts.

## Requirements

- Windows 10 or Windows 11, x64.
- Microsoft Edge WebView2 Runtime.
- Standard-user runtime. The per-machine MSI may request elevation only for installation.
- Internet access is optional for normal local operation and required only to synchronize public
  NVD/CISA repositories.

## Installation

Local review outputs include a portable/release executable, MSI, and NSIS setup. MSI is suitable
for managed installation; NSIS is the alternative setup and provides an explicit, unchecked
`Delete app data` choice during interactive uninstall. Silent uninstall and MSI uninstall preserve
application data.

These outputs are marked `UNSIGNED BUILD`. Do not bypass organizational Windows security policy or
represent them as authenticated publisher artifacts. MSI and NSIS are the supported installation
assets; the standalone executable is retained for local QA and troubleshooting rather than normal
installation.

## Code signing

The Windows binaries included in this v1.0.0 release are currently unsigned. The Authenticode
signing pipeline is prepared, but a trusted public code-signing certificate has not yet been
applied. Signed releases remain fail-closed until publisher identity and trusted timestamp
verification succeed.

## Local data and offline behavior

The authoritative `sentinel.db` and reconstructible `vulnerability-cache.db` are stored under
`%APPDATA%\com.edy.sentinel`. The main database contains endpoint telemetry, Inventory, baseline,
event, Detection, score, preference, and vulnerability-evidence history. Protect it as sensitive
endpoint data.

Telemetry, Inventory, baseline, Security Events, Detections, scores, and already materialized
vulnerability evidence remain local and usable without Internet access. A valid local NVD/CISA
cache remains available offline; only provider synchronization requires HTTPS. Missing,
inaccessible, or failed cache recovery degrades Vulnerability Intelligence explicitly without
blocking the primary application.

Process command lines are not persisted, file contents are not collected, and endpoint inventory
or telemetry is not uploaded to NVD or CISA. EDY Sentinel 1.0.0 includes no product analytics or
proprietary cloud telemetry.

## Validated security state

The final local preparation preserves the audited reference state when host facts are equivalent:

- Security Score 86 = 100 − Detections 0 − Vulnerabilities 14;
- formula version 2;
- 31 Confirmed CVEs;
- 1 Possible CVE with score impact zero;
- 1 Confirmed CVE in CISA KEV;
- product impacts Oracle JRE 6, Oracle VirtualBox 4, and Python 4;
- six enabled Detection rules at version 1.

These values describe the audited local host state, not fixed demo data, a universal expected
result, or a guarantee that another endpoint is secure.

## Known limitations

- The binaries are an `UNSIGNED BUILD`; Authenticode publisher identity and trusted timestamp
  verification are not present in this release.
- An empty-profile first run and a completely network-isolated provider attempt require a
  disposable Windows account or VM; Tauri's Known Folder resolution cannot be redirected safely
  with process-level `APPDATA` overrides on the development account.
- Final responsive validation used controlled WebView2 viewports; it is not a substitute for every
  physical monitor, scaling configuration, or enterprise Windows policy.
- Product identity is deliberately narrow and fails closed when vendor, product, version, or NVD
  applicability evidence is ambiguous.
- The six local rules are conservative observability rules, not full EDR or malware classification.
- Automated response, process/network blocking, firewall/service control, external reputation,
  and advanced network scanning are outside v1 scope.
