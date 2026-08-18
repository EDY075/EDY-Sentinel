# Sprint 5A — v1.0 release candidate and final validation

Date: 2026-08-17

Repository: `D:\Projects\EDY-Sentinel`
Branch: `main`
Candidate: `1.0.0-rc.1`
MSI numeric version: `1.0.0.1`
Sprint 4B checkpoint: `bd690ebe2523d7680599b6383e9cc69cb23f0c51`

No tag, push, GitHub Release, public signed artifact, private certificate, or Sprint 5B work was
created.

## Version and product metadata

The npm package, Cargo package, Tauri application, EXE/NSIS version resources, Settings About panel,
and bundle filenames use `1.0.0-rc.1`. WiX requires a numeric version and uses `1.0.0.1`. The stable
MSI upgrade code is `229d16f0-7d28-522e-8cc0-6a06750db9a7`.

Product name and executable remain `EDY Sentinel` / `edy-sentinel.exe`, x64. Installer publisher
remains the established identifier-derived `edy` value. This was required for NSIS 0.1.0 upgrade
compatibility and is package metadata, not Authenticode authentication or an invented company.
Source authorship and copyright remain `EDY Sentinel contributors`.

The build wrapper removes only the ignored prior bundle-output directory before building, preventing
stale installers from entering the current artifact set.

## Install, upgrade, and uninstall

### Clean installer

NSIS installed silently into a dedicated directory under the Sprint 5A temporary root. Exit code was
0; registry, executable, version `1.0.0-rc.1`, publisher `edy`, and uninstaller were present. Silent
uninstall returned 0, removed the install directory and uninstall registration, and preserved the
real application data.

An empty first runtime could not be isolated safely on this Windows account. Tauri resolves the
Roaming Known Folder through Windows APIs, so process-level `APPDATA`/`LOCALAPPDATA` overrides did not
redirect the database. The attempted isolated launch reached the real database; because the old
unsigned executable was running from `Temp`, `EDY-PROC-001` correctly created one factual Low
Detection and the score temporarily became 83. The process was closed immediately. The normal RC
then observed the process end, resolved the condition through its existing lifecycle, and returned
the score to 86 without deleting or editing history. No database evidence was erased.

Therefore first-run database creation, initial locale/baseline warm-up and restart from a genuinely
empty profile remain a disposable Windows user/VM acceptance step. No further attempt was made to
fake isolation or modify the real databases.

### Upgrade

The original unsigned NSIS 0.1.0 artifact was preserved outside Git before RC build. The first RC
publisher attempt used `EDY Sentinel contributors`; real testing showed that NSIS rejected the prior
publisher `edy` and installed to another directory. Restoring the established publisher fixed the
blocker.

The repeated test upgraded 0.1.0 to 1.0.0-rc.1 in the same custom install directory. Registry
version/publisher and the installed EXE changed to the RC. SHA-256 for `sentinel.db` and
`vulnerability-cache.db` was identical before upgrade, after upgrade, and after silent uninstall.

### Uninstall policy

MSI does not remove application data. NSIS preserves it by default; the interactive NSIS uninstaller
offers an explicit unchecked `Delete app data` option. Silent uninstall leaves the option unset.
Tests confirmed removal of installer-owned files and registration while the main/cache databases
remained present.

## Functional and security regression

The real database after all lifecycle tests reports:

- SQLite schema 11;
- 81 latest software evaluations;
- 31 Confirmed CVEs;
- 1 Possible CVE with score impact 0;
- 1 KEV Confirmed;
- Security Score 86 = 100 − Detections 0 − Vulnerabilities 14;
- formula version 2 and availability `available`;
- product impacts JRE 6, VirtualBox 4, Python 4;
- zero active Detections;
- six enabled rules, all rule version 1;
- primary/cache `integrity_check=ok` and zero foreign-key violations;
- local NVD 378,386 and CISA KEV 1,666 records.

The native Confirmed audit passed all 31 relationships with original software/version/publisher,
canonical vendor/product/CPE, selected range, configuration, confidence, resolver/matching versions,
source versions and evidence. No matching, CPE, range, score weight/cap, or Detection rule changed.

Cache failure fixtures proved that an unavailable cache and startup recovery failure degrade
explicitly without blocking the primary application. The live Windows collector smoke passed. Full
rule tests cover registry integrity, positive/negative conditions, precedence, deduplication,
reopen, evidence, severity and confidence; no test-only rule exists in the RC registry.

An external-network proxy restriction was attempted but blocked by the execution environment before
process creation. No Windows firewall/security setting was changed. Offline behavior remains covered
by real local-cache UI observation plus provider/cache failure tests; a fully network-isolated VM
sync attempt remains a manual acceptance item.

## Performance

Window creation was observed after 176 ms; this is a native window-ready time, not a claim of full
data hydration. After a five-second warm-up, three 15-second process samples measured:

| Sample | CPU, one core | CPU, 12-core host | Working set | Private | Threads | Handles |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 20.00% | 1.67% | 44.8 MiB | 21.3 MiB | 28 | 727 |
| 2 | 8.75% | 0.73% | 46.6 MiB | 23.3 MiB | 28 | 727 |
| 3 | 7.92% | 0.66% | 47.3 MiB | 24.3 MiB | 28 | 727 |

Sample 1 still includes startup/hydration work. Steady samples align with Sprint 4B's 8.65–9.06%
of one core and remain below 0.8% of the host. The native score benchmark measured 5.595 ms query,
0.468 ms calculation, and 6.138 ms end-to-end persistence. No relevant regression was found.

## Desktop QA and accessibility

The actual release EXE was used. Direct Windows.Graphics.Capture remains unavailable for this
WebView2 (`0x80004002`), so the foreground app was captured through `Alt+PrintScreen`; every image
was saved outside Git and inspected.

Real QA covered:

- English and Português (Brasil);
- Sentinel Blue, Cyber Green, Terminal, and Spectrum;
- configured 1440x900 window (captured 1442x932 including frame);
- Overview, Processes, Network, Inventory, Detections, Security Score, and Settings;
- version/channel in Settings, score formula/details, provider status, collector states, search/filter
  surfaces, drawer focus/Escape, command palette, tooltips/ARIA tree and keyboard routing.

The screenshots showed no overlap, horizontal page overflow, broken contrast, raw i18n keys or
truncation in the reviewed surfaces. Terminal correctly reflowed long processor text. The command
palette retained input across telemetry rerenders.

The WebView2 UIA provider did not expose usable click geometry for the Inventory rows, so an
individual CVE Detail screenshot was not obtained. Physical 1920x1080, 1600x900, 1366x768 and
1280x720 resizing was also unavailable through the approved desktop control API. Automated
responsive CSS/build tests passed, but they are not represented as desktop screenshots. CVE Detail
and those four physical resolutions remain manual release-environment acceptance items.

### Screenshot inventory outside Git

Root: `%LOCALAPPDATA%\Temp\EDY-Sentinel-Sprint5A-RC\screenshots`

- `overview-en-cyber-green-1440x900.png`
- `overview-pt-BR-terminal-1440x900.png`
- `overview-en-sentinel-blue-1440x900.png`
- `overview-en-spectrum-1440x900.png`
- `processes-en-cyber-green-1440x900.png`
- `network-en-cyber-green-1440x900.png`
- `inventory-en-cyber-green-1440x900.png`
- `detections-en-cyber-green-1440x900.png`
- `security-score-en-cyber-green-1440x900.png`
- `settings-en-cyber-green-1440x900.png`
- `settings-pt-BR-terminal-1440x900.png`

No CVE Detail image is claimed.

## Technical quality gates

- lint: pass;
- TypeScript: pass;
- frontend/i18n: 65 passed in 19 files;
- React production build: pass, 504.16 kB JS / 145.65 kB gzip, advisory only;
- cargo fmt: pass;
- cargo check all targets: pass;
- cargo clippy all targets with warnings denied: pass;
- Rust: 128 passed, 0 failed, 14 opt-in/manual ignored;
- real collectors, 31-CVE audit and score benchmark opt-in gates: pass;
- production npm audit: no known vulnerability;
- Rust audit: no blocking applicable Windows-runtime vulnerability; 17 previously documented
  transitive warnings remain;
- Tauri release EXE/MSI/NSIS: pass;
- signed-release fail-closed test without thumbprint: expected exit 2 before compilation;
- database integrity/FK: pass;
- final secret/artifact scan and `git diff --check`: pass.

## Unsigned RC artifacts

| Artifact | Bytes | SHA-256 | Authenticode | Timestamp |
| --- | ---: | --- | --- | --- |
| `edy-sentinel.exe` | 16,237,056 | `BAE0E9EAC8F39E35BD17DCF51B11EAFAF5A4221867046E48B4D3493E2D506CDE` | NotSigned | No |
| `EDY Sentinel_1.0.0-rc.1_x64_en-US.msi` | 6,037,504 | `012E4F2AE0D08DE78DE0745285D158AA15F568C964D40EB6D9F400DF0DED5173` | NotSigned | No |
| `EDY Sentinel_1.0.0-rc.1_x64-setup.exe` | 4,336,794 | `452CB64976A345252DA8737D44B5F161D246FB19495A6E6762015C80CD12E248` | NotSigned | No |

`BUILD-TRUST.txt` states `UNSIGNED RELEASE CANDIDATE` and version `1.0.0-rc.1`. These are local
technical QA artifacts, not public release hashes.

## Status

**TECHNICAL V1 STATUS:** Release-candidate code and gates pass.

**INSTALL/UPGRADE/UNINSTALL STATUS:** Installer/upgrade/uninstall pass; isolated empty-profile first
runtime remains a disposable-user/VM gate.

**SECURITY STATUS:** Core regressions, six rules, score, vulnerability evidence, audits and database
integrity pass. The temporary-path Detection created by QA resolved factually and remains in history.

**PERFORMANCE STATUS:** Pass; steady runtime aligns with Sprint 4B and score persistence improved in
the final sample.

**DOCUMENTATION STATUS:** README, user guide, checklist, architecture, security, localization,
Detection, score, matching and signing documents updated.

**AUTHENTICODE STATUS:** `PUBLIC SIGNED RELEASE: BLOCKED`. No legitimate certificate/private key was
available; EXE/MSI/NSIS are unsigned and untimestamped.

**GIT STATUS:** Sprint 4B committed separately. Sprint 5A is ready for the requested RC commit; no
tag, push, or release.

## Verdict

**READY FOR V1.0 FINAL RELEASE PREPARATION**

This does not mean ready for public distribution. Besides Authenticode, the public acceptance record
still needs the disposable-profile first-run/offline test, individual CVE Detail capture, and the
four remaining physical desktop resolutions.
