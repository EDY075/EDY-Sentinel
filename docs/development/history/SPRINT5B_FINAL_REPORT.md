# Sprint 5B — EDY Sentinel 1.0.0 final local preparation

Date: 2026-08-17

Repository: `D:\Projects\EDY-Sentinel`

Branch: `main`

Source checkpoint before Sprint 5B: `73d4dcb0b7cae90ae551c95540c2645753a31c71`

No push, tag, GitHub Release, public installer, external artifact transfer, certificate, private key,
or fabricated signature was created.

## Version and product metadata

The npm package, Cargo package/lock, Tauri application, EXE/NSIS version resources, Settings About
panel, MSI, NSIS, filenames, documentation, and NVD/CISA provider User-Agent use final version
`1.0.0`. The final MSI uses numeric version `1.0.0` and retains stable UpgradeCode
`229d16f0-7d28-522e-8cc0-6a06750db9a7`.

Product identity remains `EDY Sentinel`, executable `edy-sentinel.exe`, Windows x64. Installer
publisher remains the established identifier-derived `edy` value for NSIS upgrade compatibility.
It is package metadata, not a claimed legal company or cryptographic publisher identity. Source
authorship remains `EDY Sentinel contributors`.

Historical `0.x` and `1.0.0-rc.1` references were retained only in changelog, roadmap, prior sprint
reports, dependency versions, and upgrade evidence. No active product metadata remains on an old
version.

## Functional and security regression

The final source tree preserves:

- real Windows system, process, connection/network, service, and Inventory collection;
- versioned behavioral baseline and factual Security Events;
- Detection Engine v1 with exactly six enabled rules, all rule version 1;
- zero active Detections in the audited host state;
- 81 latest software evaluations, 31 Confirmed CVEs, 1 Possible at impact zero, and 1 Confirmed KEV;
- Security Score 86 = 100 − Detections 0 − Vulnerabilities 14;
- formula version 2 and availability `available`;
- product impacts Oracle JRE 6, Oracle VirtualBox 4, and Python 4;
- English, Português (Brasil), and four token-based themes.

The complete Rust suite passed 128 tests with 14 explicit manual/opt-in tests ignored by the normal
run. Separate native opt-in gates passed for Windows collectors, all 31 Confirmed relationships, and
the real Security Score calibration. Cache absence, unavailable cache, interrupted recovery,
identity/range/applicability, Possible-zero-impact, caps, KEV post-match semantics, v1/v2 history,
rule positive/negative behavior, precedence, deduplication, and reopen remain covered.

No mock, fixture score, test Detection rule, or fabricated endpoint fact is present in the release
registry or production UI.

## Privacy and security audit

Code and documentation were crossed directly:

- Tauri stores `sentinel.db` and `vulnerability-cache.db` under its Windows `app_data_dir`;
- process command lines may exist in the live DTO but are absent from persisted observation schema,
  with an explicit regression test;
- no user file-content collector or arbitrary file/path IPC exists;
- Tauri desktop capability remains only `core:default`;
- React cannot execute shell, WMI, Registry, SQL, service, firewall, or process-control operations;
- only public NVD and CISA KEV providers perform external network requests;
- providers are HTTPS-only, bounded, validated, retry-limited, and use sanitized failures;
- endpoint Inventory and telemetry are not sent to NVD or CISA;
- no product analytics, proprietary cloud telemetry, API key, required credential, or repository-held
  release secret exists.

`pnpm audit --prod` found no known vulnerability. `cargo audit` found no blocking applicable
Windows-runtime vulnerability; 17 previously documented unmaintained/unsound transitive warnings
remain visible. High-confidence secret, environment-file, personal runtime artifact, and tracked
database/executable/installer scans passed.

## Database integrity

Final read-only checks:

| Database | Bytes | Schema | integrity_check | FK issues |
| --- | ---: | ---: | --- | ---: |
| `sentinel.db` | 361,725,952 | 11 | `ok` | 0 |
| `vulnerability-cache.db` | 801,083,392 | 1 | `ok` | 0 |

The cache contains 378,386 NVD records and 1,666 CISA KEV records.

## Installation, upgrade, and uninstall

The final NSIS `1.0.0` was installed silently into an isolated Sprint 5B temporary directory.
Executable version, publisher `edy`, uninstall registration, and uninstaller were present. Silent
uninstall returned zero and removed the install directory and registration.

A separate upgrade test installed the preserved unsigned `0.1.0` NSIS into another isolated
directory and then ran the final `1.0.0` installer. The same directory was upgraded to executable
version `1.0.0`, with publisher and registration updated. Silent uninstall returned zero. SHA-256
for the real `sentinel.db` and `vulnerability-cache.db` was identical before and after both complete
cycles.

MSI and NSIS preserve application data by default. Interactive NSIS exposes an explicit unchecked
`Delete app data` choice; it was not selected. A genuinely empty-profile first run remains a
disposable Windows user/VM acceptance gate because process-level `APPDATA` overrides do not redirect
Tauri's Windows Known Folder resolution safely.

## Performance

The final release executable exposed its native window after 114 ms. This is window readiness, not
a claim of complete data hydration. After a five-second warm-up, three 15-second polling samples
reported:

| Sample | CPU, one core | CPU, 12-core host | Working set | Private | Threads | Handles |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 17.79% | 1.48% | 46.0 MiB | 23.8 MiB | 28 | 708 |
| 2 | 4.90% | 0.41% | 44.4 MiB | 22.0 MiB | 28 | 708 |
| 3 | 6.35% | 0.53% | 46.4 MiB | 24.2 MiB | 27 | 705 |

The first sample includes additional startup/hydration work. Steady CPU improved from the Sprint
4B/5A 7.92–9.06% one-core range and stays well below 1% of the 12-core host. No thread/handle growth
was observed. Polling remains 2.5 seconds live and 15 seconds system.

The real score benchmark measured 5.971 ms query, 0.355 ms calculation, and 6.017 ms end-to-end
persistence. The latest 81-software matching set records 2,278 ms total computation, 28.12 ms
average, and 701 ms maximum. No final-release optimization was necessary.

## UX, localization, themes, accessibility, and screenshots

The actual final release EXE was opened. Its accessibility tree verified real Overview content,
healthy collector surfaces, baseline Ready, SQLite schema 11, score 86, zero Detection contribution,
Vulnerability contribution 14, and 31 items requiring attention. The application closed normally.

Direct Windows.Graphics.Capture again failed for this WebView2 with `SetIsBorderRequired ...
0x80004002`. The UIA tree exposed semantic content but no click geometry, and injected shortcuts did
not cross the RootWebArea reliably. No new final-build pixel screenshot, Settings interaction, or CVE
Detail interaction is claimed.

The eleven real Sprint 5A screenshots covering English/pt-BR, Sentinel Blue, Cyber Green, Terminal,
Spectrum, Overview, Processes, Network, Inventory, Detections, Security Score, and Settings were
preserved outside Git at:

`%LOCALAPPDATA%\Temp\EDY-Sentinel-Sprint5B-Final\screenshots-from-rc-ui`

They are explicitly RC-lineage UI evidence. Sprint 5B changed version/channel text and documentation,
not layout, themes, responsive CSS, or domain views. Automated i18n parity/runtime tests and the
final production build passed, but they are not substituted for missing final pixel evidence.

## Technical gates

- pnpm lockfile frozen install: pass;
- lint: pass;
- TypeScript typecheck: pass;
- frontend/i18n: 65 passed in 19 files;
- React production build: pass, 504.10 kB JS / 145.63 kB gzip; advisory only;
- cargo fmt: pass;
- cargo check all targets: pass;
- cargo clippy all targets with warnings denied: pass;
- Rust: 128 passed, 0 failed, 14 manual/opt-in ignored;
- native collectors, 31-CVE audit, and score benchmark: pass;
- npm audit: pass;
- cargo audit: pass with 17 documented allowed warnings;
- signed-release missing-credential fail-closed test: expected exit 2 before compilation;
- Tauri release EXE/MSI/NSIS: pass;
- clean install, `0.1.0` upgrade, silent uninstall, and data preservation: pass;
- primary/cache integrity and foreign keys: pass;
- `git diff --check`, secret, `.env`, and runtime-artifact scans: pass.

## Final local artifacts

| Artifact | Bytes | SHA-256 | Authenticode | Timestamp |
| --- | ---: | --- | --- | --- |
| `edy-sentinel.exe` | 16,236,032 | `772E2D17FC190FF48AC9E297C6D23A8CDE00CB4F71A9B021FCD88EBFAE0C2E9F` | NotSigned | No |
| `EDY Sentinel_1.0.0_x64_en-US.msi` | 6,037,504 | `940386708A06F54E252A536EF8635BD4D3600265B61B021FB074BD38B6081B18` | NotSigned | No |
| `EDY Sentinel_1.0.0_x64-setup.exe` | 4,339,778 | `BE42E3F4C0C13017B0CF815D4530C39D7C779ADAB6EE7F0A1D5880E073118136` | NotSigned | No |

`BUILD-TRUST.txt` states `UNSIGNED BUILD`, version `1.0.0`, and that public distribution is blocked
until Authenticode signing and timestamp verification pass. These are mutable local QA hashes, not
public release hashes.

## Documentation

README, CHANGELOG, ROADMAP, SECURITY, ARCHITECTURE, LOCALIZATION, Detection Rules, Security Score,
Vulnerability Matching, Code Signing, User Guide, and Release Checklist were reviewed. Architecture
now correctly reports primary schema 11. [`../releases/v1.0.0.md`](../releases/v1.0.0.md) and this final report were added.

## Known limitations and final status

- `PUBLIC SIGNED RELEASE: BLOCKED` because no legitimate Authenticode certificate/private key was
  available. The repository does not create or accept a false self-signed production identity.
- Disposable-profile first-run and fully network-isolated provider acceptance remain VM gates.
- Individual CVE Detail and physical 1920x1080, 1600x900, 1366x768, and 1280x720 final captures
  remain release-environment visual gates.
- No tag, push, public release, or external publication was performed.

## Verdict

**V1.0.0 READY FOR USER REVIEW BEFORE PUBLICATION**
