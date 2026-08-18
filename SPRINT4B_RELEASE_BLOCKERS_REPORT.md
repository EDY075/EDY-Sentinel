# Sprint 4B — Release blockers and final hardening

Date: 2026-08-17  
Repository: `D:\Projects\EDY-Sentinel`  
Branch: `main`  
Sprint 4A checkpoint: `5d2762c70c8fea04b0e3154429fa88fa28d1f81c` —
`fix: harden sentinel before v1 release`

Sprint 4B remains uncommitted for review. No tag, push, release, Sprint 5 work, Security Score
redesign, Matching Engine change, Detection Engine rule change, or fabricated release evidence was
created.

## Release blockers resolved technically

### Vulnerability-history retention

Schema 11 introduces retention policy v1 and persisted maintenance state. The policy preserves, as
complete evaluation families, the latest evaluation for every software record, the 100 newest
families, the latest checkpoint per UTC day through 90 days, and the latest checkpoint per UTC
month through 730 days. Identity evidence, CPE candidates, matches, matching provenance and
source/engine versions are retained or removed together.

Cleanup is oldest-first, capped at 250 obsolete families per transaction, persisted on a 24-hour
cadence, reached through the existing six-hour maintenance boundary. It does not run on every
refresh and does not run live `VACUUM`. The migration deleted zero rows from the real current data
because it already fits the policy. The focused lifecycle test proved removal of five obsolete
families, preservation of current Confirmed evidence, an immediate no-op due to cadence, and an
exact 250-family second backlog batch.

### Duplicate security analysis

`get_live_telemetry` is now the single owner of the polling-cycle Detection Engine and Security
Score analysis. The concurrent 15-second system request only collects/persists system and network
facts. Collector frequencies remain 2.5 seconds for live hydration and 15 seconds for system
collection; a system fact committed after concurrent analysis is consumed by the next bounded live
cycle. Structurally, joint-cycle analysis invocations fell from two to one, removing one redundant
Detection/Score transaction every 15 seconds.

### Authenticode release infrastructure

`CODE_SIGNING.md`, `scripts/build-release.mjs`, and `scripts/verify-authenticode.ps1` define the
production boundary. A public release job must inject a legitimate certificate from the Windows
Certificate Store plus its provider timestamp URL, set
`EDY_SENTINEL_REQUIRE_SIGNED_RELEASE=1`, and verify `Status=Valid`, signer and timestamp on the app
EXE, MSI and NSIS executable. Private keys, PFX material and credentials remain outside the
repository and command line. The pipeline neither creates nor accepts a self-signed substitute.

A local build without credentials remains supported and prints/writes exactly:

`UNSIGNED DEVELOPMENT BUILD`

The signed-release mode was also exercised without a thumbprint and failed before compilation, as
required.

## Real data and score invariants

- primary database schema: 11;
- primary `integrity_check`: `ok`; foreign-key violations: 0;
- cache `integrity_check`: `ok`; foreign-key violations: 0;
- current vulnerability evaluations before final release close: 1,220;
- current vulnerability matches before final release close: 137,990;
- latest result: 31 Confirmed CVEs, 1 Possible CVE and 1 KEV Confirmed;
- score: 86 = 100 − Detection penalty 0 − Vulnerability penalty 14;
- formula version: 2; Possible CVE impact: 0;
- product impacts: Oracle JRE 6, Oracle VirtualBox 4, Python 4;
- six Detection rules remain enabled at version v1;
- active Detections: 0; active High/Critical Detections: 0;
- retention last/total deleted on the real current data: 0/0.

The native Confirmed-audit gate independently returned 31. Matching Engine v1, the CPE resolver,
version ranges, Security Score formula/caps and all six Detection rules were left unchanged. The 31
Confirmed matches therefore remain traceable to their stored identity, CPE, range, source and
engine-version evidence.

## Performance

Three 15-second release-process samples after the polling fix measured 9.06%, 9.06% and 8.65% of
one logical core (0.76%, 0.76% and 0.72% of the 12-core host), with 52.2–54.7 MiB working set,
28.9–31.2 MiB private memory, 24–26 threads and 735–739 handles. The Sprint 4A single sample was
8.02% of one core / 0.67% of the host; the small sample variation is not evidence of regression,
while the removed transaction is structurally verified.

The final native score benchmark measured 42.401 ms query time, 0.322 ms calculation time and
23.492 ms end-to-end persistence. The real release reached a fully hydrated Overview during the QA
observation; a new precise startup distribution was not claimed because WebView2 capture could not
provide a reliable hydration timestamp.

## Real desktop visual QA

The actual release executable was opened, not a browser-only frontend. Direct WebView2 capture
failed with `SetIsBorderRequired ... 0x80004002`; safe Windows foreground-window clipboard capture
was used instead and each image was visually inspected.

Validated in the real English/Cyber Green release at configured 1440x900:

- Overview collector health, score 86, formula/cap explanation and schema 11;
- Inventory: 81 records, search/filter controls, columns and scroll presentation;
- Security Score drawer: `100 - 0 - 14 = 86`, product cards and separate identity coverage;
- Oracle JRE product-risk detail: canonical identity, six Confirmed CVEs, one KEV, impact 6,
  matching engine/resolver versions, evaluation timestamp and explicit statement that KEV only
  prioritizes an already confirmed relationship;
- Settings exposed both English and Português (Brasil), NVD Ready with 378,386 records and CISA KEV
  Ready with 1,666 records;
- command-palette filtering and keyboard selection remained stable across telemetry rerenders after
  the Dialog focus correction.

Real screenshots kept outside Git:

1. `%LOCALAPPDATA%\Temp\EDY-Sentinel-Sprint4B-Screenshots\overview-en-cyber-green-1440x900.png`;
2. `%LOCALAPPDATA%\Temp\EDY-Sentinel-Sprint4B-Screenshots\inventory-en-cyber-green-1440x900.png`;
3. `%LOCALAPPDATA%\Temp\EDY-Sentinel-Sprint4B-Screenshots\security-score-en-cyber-green-1440x900.png`.

Limitations are explicit: this pass did not obtain honest real-release evidence for pt-BR, Sentinel
Blue, Terminal, Spectrum, 1920x1080, 1600x900, 1366x768 or 1280x720, nor an individual CVE Detail
drawer. WebView2 exposed no usable geometry for those interactions, and the automation API could
not select the runtime locale reliably. Automated 65-test frontend/i18n coverage, type checking and
the production build passed, but they are not substituted for visual evidence. These remaining
matrix items belong to the manual public-release acceptance checklist.

## Clean install and uninstall acceptance

A clean install/uninstall was deliberately not executed on the development host because it would
mutate installed-product state and real user application data. Use this exact acceptance procedure
in a disposable Windows VM snapshot or dedicated test account:

1. Record installed-product entries and confirm EDY Sentinel is absent.
2. Confirm `%APPDATA%\com.edy.sentinel` is absent; if testing upgrade/preservation separately,
   copy that directory to a protected backup instead of deleting it.
3. Install the candidate MSI and then repeat from a restored snapshot with the NSIS candidate.
4. Launch as a standard user; verify schema 11, first-run settings, both locales, all four themes,
   baseline lifecycle, Inventory, provider/cache state, score coverage and close/reopen persistence.
5. Execute the complete resolution matrix and capture Overview, Inventory, Product Identity, CVE
   Detail, KEV Detail and Security Score evidence in both locales.
6. Uninstall through Windows Installed apps; verify binaries, shortcuts and installer registration
   are removed.
7. Verify `%APPDATA%\com.edy.sentinel\sentinel.db`, preferences and
   `vulnerability-cache.db` remain. Reinstall and confirm the preserved data opens and migrates.
8. Only for an explicit erase-data test, remove the preserved directory after obtaining separate
   user consent and a backup; current installers intentionally provide no silent data deletion.

Sprint 5A follow-up: generated NSIS already contains an explicit unchecked `Delete app data` option;
the earlier statement that the option was only future work was incorrect. Preserve-by-default still
holds: silent NSIS and MSI uninstall remove installer-owned binaries/shortcuts/registration while
leaving endpoint history, preferences, reconstructible cache and future logs. Data removal requires
the user's explicit interactive NSIS selection.

## Final quality gates

- `cargo fmt --check`: pass;
- `cargo check`: pass;
- `cargo clippy --all-targets -- -D warnings`: pass;
- Rust suite: 128 passed, 0 failed, 14 ignored;
- native collectors smoke: pass;
- native Confirmed audit: pass (31);
- native Security Score benchmark: pass;
- lint: pass;
- TypeScript typecheck: pass;
- frontend/i18n tests: 65 passed in 19 files;
- React production build: pass (502.43 kB JS / 145.24 kB gzip; advisory only);
- Tauri release: pass for EXE, MSI and NSIS;
- production npm audit: no known vulnerability;
- Rust audit: no blocking applicable Windows-runtime vulnerability; 17 documented unmaintained
  transitive warnings remain;
- primary/cache integrity: `ok`; foreign-key violations: 0/0;
- `git diff --check`: pass;
- tracked artifact/secret audit: no database, log, dump, release executable, installer or credential
  added to Git.

## Unsigned artifact record

| Artifact | Bytes | SHA-256 | Authenticode | Timestamp |
| --- | ---: | --- | --- | --- |
| `edy-sentinel.exe` | 16,237,056 | `76DE417202DA5E1E1C13B4FF8412814EBFBB8208165FF7031046948B9B279245` | NotSigned | No |
| `EDY Sentinel_0.1.0_x64_en-US.msi` | 6,037,504 | `EE27D3758C98D88DB4C5DA84D7A8542D82F9B2FBCA395AB61AEDB40712650EB8` | NotSigned | No |
| `EDY Sentinel_0.1.0_x64-setup.exe` | 4,342,664 | `9075CF1F37B204094CCA668FE02FFD56F05CE1A91A058B8E63EBA1D1DFA40B49` | NotSigned | No |

These hashes describe local QA outputs only and are not public release hashes.

## Verdict

**TECHNICALLY READY FOR V1**

**BLOCKED FOR PUBLIC SIGNED RELEASE**

The public gate is blocked by the absent legitimate Authenticode certificate/timestamped signatures.
The manual clean-install/uninstall run and remaining real desktop locale/theme/resolution matrix must
also be attached to the public-release acceptance record. Nothing in this report claims those
unexecuted checks or a nonexistent signature.
