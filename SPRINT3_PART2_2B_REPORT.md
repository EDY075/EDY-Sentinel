# Sprint 3 Part 2.2B — CPE Coverage & Product Identity

## Scope and checkpoint

Work started on `main` at `25a132a9bf632fef4f33758e9f2ec169826763f3`
(`perf: isolate vulnerability cache and optimize NVD matching`). The preceding checkpoint
`f9991a9` is `feat: add conservative vulnerability matching engine v1`. Matching Engine remains
version 1; this part adds independently versioned Product Identity Resolver v1 and authoritative
schema migration 0009. No commit was created for this review checkpoint.

Security Score, Detection Engine, the six rules, severity, remediation, automatic updates, and
external enrichment beyond NVD/CISA KEV remain unchanged.

## Resolver design

The resolver keeps registry facts intact and uses only conservative, reviewed paths:

- strict canonical CPE lookup for an exact product identity;
- explicit aliases for Chrome, version-scoped Edge, Git, Node.js, Discord, and qBittorrent;
- product-specific extractors for Python, Oracle Java Runtime, WinRAR, Opera, PHP, and Oracle VM
  VirtualBox;
- exact publisher constraints and explicit vendor/product separation;
- stable unresolved reasons for missing/invalid facts, unsupported products, incompatible vendors,
  ambiguity, and insufficient evidence;
- no fuzzy matching, arbitrary substrings, CVE-description matching, or JDK/JRE conflation.

Python's registry package build is converted only when its exact Python family label and publisher
are present. Java 8 Update 401 is represented as `oracle / jre`, version `1.8.0`, CPE update
`update401`; the update field remains part of criterion applicability.

The following installed components deliberately remain unresolved unless an independent security
product can be proved: Python Launcher, Visual Studio Installer/Build Tools, Microsoft Visual C++
redistributables, helpers, and launchers.

## Persistence and UI

Migration 0009 extends immutable identity evaluations with resolver version/status/method,
canonical vendor/product, confidence, unresolved reason, and JSON provenance. Evaluation candidates
record their origin and canonical CPE. The Inventory detail drawer shows Resolved/Ambiguous/
Unresolved identity, canonical identity/version/CPE, method, confidence, reason, provenance, and
resolver version in English and pt-BR.

## Real inventory re-evaluation

The native Windows inventory was re-evaluated against the existing local NVD/KEV cache. Software
identity coverage and vulnerability counts are reported separately because they have different
units and must not be added or compared as one population.

### Software identity coverage

| Metric | Software records |
|---|---:|
| Total installed software | 81 |
| Identities resolved | 19 |
| Ambiguous | 0 |
| Unresolved | 62 |
| Not safely mappable (subset of unresolved) | 15 |

The other unresolved reasons are 41 with no CPE candidate, three vendor mismatches, and three
missing versions. All 81 current software records have an immutable resolver evaluation.

### Vulnerability matches

| Metric | CVEs |
|---|---:|
| Confirmed CVEs | 31 |
| Possible CVEs | 1 |
| KEV Confirmed | 1 |
| Oracle VirtualBox 7.2.12 | 15 Confirmed |
| Python 3.12.10 | 10 Confirmed, 1 Possible |
| Oracle JRE 8u401 | 6 Confirmed, including 1 KEV |

There are exactly 31 current Confirmed match rows and 31 unique Confirmed CVE IDs. The single
Possible CVE is not counted as Confirmed. CISA KEV is enrichment/prioritization and did not create
the software-to-CVE confirmation.

The 19 resolved records represent 14 distinct displayed products because the inventory contains
duplicate registry records. Node.js, Python, Java, Chrome, Edge, Git, Discord, qBittorrent, WinRAR,
Opera, PHP, and VirtualBox are covered by the reviewed aliases/extractors; Claude Code and Telegram
Desktop resolve through the pre-existing strict canonical lookup.

Confirmed findings group into Python 3.12.10 (10), Oracle Java 8u401 JRE (6), and Oracle VM
VirtualBox 7.2.12 (15). The complete CVE IDs remain in immutable database evidence and the UI; no
security score or detection conclusion was derived from them.

## CONFIRMED CVE AUDIT

Part 2.2C audited every one of the 30 original `Confirmed` rows from the Windows inventory through
canonical identity, CPE, normalized version, selected NVD criterion/configuration, CVSS, KEV, and
Matching Engine v1. All 30 remain defensible. The audit also found one false negative:
`CVE-2023-41993` had been suppressed by a manual association rejection even though the local NVD
configuration contains an exact vulnerable Oracle JRE 8u401 criterion and Oracle's April 2024 CPU
lists Java SE 8u401 as affected. The rejection was removed. The result is 31 `Confirmed`, not a
coverage regression: 30 retained, zero removed/regraded, and one corrected addition. Because this
CVE is in CISA KEV, confirmed KEV coverage changes from zero to one.

Every selected criterion in the 31-row compact audit had `vulnerable=true`, a complete verified
configuration payload, a root `OR` path with `negate=false`, a single non-ambiguous High-confidence
identity, and `identity_version_and_nvd_applicability_confirmed`. Other branches in a CVE tree were
not treated as local facts. In particular, `CVE-2023-41993` contains several product/platform
branches; only its exact `oracle:jre:1.8.0:update401` branch established the local match.

| Installed product | Canonical identity / CPE | Confirmed | Audited CVEs and range pattern |
|---|---|---:|---|
| Java 8 Update 401 (64-bit), registry `8.0.4010.10`, Oracle Corporation | `oracle/jre`; `cpe:2.3:a:oracle:jre:1.8.0:update401:*:*:*:*:*:*` | 6 | `CVE-2023-41993`, `CVE-2024-21003`, `-21005`, `-21011`, `-21085`, `-21094`; exact `= 1.8.0` plus exact `update401`; `CVE-2023-41993` is KEV |
| Oracle VirtualBox 7.2.12, Oracle and/or its affiliates | `oracle/vm_virtualbox`; `cpe:2.3:a:oracle:vm_virtualbox:7.2.12:*:*:*:*:*:*:*` | 15 | `CVE-2026-47041`, `-47043`, `-47044`, `-47047`, `-47053`, `-47054`, `-47055`, `-47062`, `-60150`, `-60155`, `-60158`, `-60159`, `-60160`, `-60161`, `-60162`; exact `= 7.2.12` |
| Python 3.12.10 (64-bit), registry package build `3.12.10150.0`, Python Software Foundation | `python/python`; `cpe:2.3:a:python:python:3.12.10:*:*:*:*:*:*:*` | 10 | `CVE-2025-12084` `<3.13.11`; `-12781` `<3.13.10`; `-13462` `<3.13.13`; `-13836` `>=3.12.0 and <3.12.13`; `-13837` `<3.13.10`; `CVE-2026-15308` `<3.15.0`; `-3644`/`-4519` `<3.13.13`; `-6019`/`-7210` `<3.13.14` |

The Java extractor preserves JRE versus JDK and the update dimension; a different distribution or
JDK label does not resolve to this JRE. Python derives security version `3.12.10` from the exact
runtime DisplayName rather than comparing the four-component registry package build, and rejects
Launcher/helper/component identities. VirtualBox requires its exact display family, publisher, and
version. The compact real-database audit is reproducible with the ignored
`native_windows_confirmed_cve_audit` test and asserts all 31 evidence chains.

## False-positive review

- Three Java candidates that required an unobserved specific/not-applicable CPE context were
  rejected after all CPE 2.3 dimensions became fail-closed.
- The earlier `CVE-2023-41993` manual rejection was itself a false negative. Oracle's CPU and the
  exact NVD JRE 8u401 branch support the installed JRE, so a regression now requires `Confirmed`
  and null `reviewedAssociationRejection` evidence.
- Wrong vendors, similar product labels, different products, missing/invalid versions, casing, and
  Unicode-confusable names are covered by negative Rust fixtures.
- AND/OR/negate, vulnerable=false, exact/ranged versions, unknown environment, incompatible vendor,
  ambiguity, and KEV-as-enrichment regression fixtures continue to fail closed.

Primary validation references:

- Oracle Java 8u401 release notes: https://www.oracle.com/java/technologies/javase/8u401-relnotes.html
- Oracle April 2024 Critical Patch Update: https://www.oracle.com/security-alerts/cpuapr2024.html
- Oracle July 2026 Critical Patch Update: https://www.oracle.com/security-alerts/cpujul2026.html
- Oracle public CVE-to-bulletin mapping: https://www.oracle.com/security-alerts/public-vuln-to-bulletin-mapping.html
- Python 3.12.10 release: https://www.python.org/downloads/release/python-31210/
- Python 3.12.11 release: https://www.python.org/downloads/release/python-31211/
- Python security announcements, December 2025: https://mail.python.org/archives/list/security-announce%40python.org/2025/12/
- Python security announcements, January 2026: https://mail.python.org/archives/list/security-announce%40python.org/2026/1/
- Python security announcements, March 2026: https://mail.python.org/archives/list/security-announce%40python.org/2026/3/
- Python 3.12.13/3.11.15/3.10.20 security releases: https://discuss.python.org/t/python-3-12-13-3-11-15-and-3-10-20-are-now-available/106363
- Python 3.14.6/3.13.14 security releases: https://discuss.python.org/t/python-3-14-6-and-3-13-14-are-now-available/107714
- CISA KEV catalog: https://www.cisa.gov/known-exploited-vulnerabilities-catalog

## Performance calibration

Part 2.2A's 81-item wall baseline was 6.259–6.450 seconds with 6,294 candidate decisions. Part
2.2B expands work to 9,972 candidate decisions and three new high-cardinality product families;
the measured full wall is 18.588–20.209 seconds. This is a significant full-run regression and is
reported rather than hidden.

The indexed two-phase query, one decoded configuration per unique CVE/product bundle, external
cache isolation, and no N×M/description scan remain intact. Isolated measurements were:

| Product | Identity | Matching | Confirmed CVEs |
|---|---:|---:|---:|
| Chrome | 16 ms | 5,978 ms | 0 |
| Node.js | 15 ms | 350 ms | 0 |
| Python | 15 ms | 320 ms | 10 |
| Java | 15 ms | 1,995 ms | 5 |

Chrome bundle load was 1,896 ms versus 1,856 ms in 2.2A (+2.2%); its isolated wall was 5,988 ms
versus 5,154 ms (+16.2%). The original storage/indexing optimization remains structurally present,
but expanded coverage has a real total cost. Further performance work should target bundle/query
cost without weakening identity or applicability rules.

## PERFORMANCE AFTER COVERAGE EXPANSION

Part 2.2C reduces the preserved 81-item real-database wall from 18.588–20.209 seconds to
9.473–10.233 seconds across repeated post-change runs (approximately 45–53% faster) while keeping
19 resolved identities, zero ambiguous identities, fail-closed behavior, all 9,972 unique CVE
decisions, and complete per-decision provenance. The technically explained coverage change is the
audited `CVE-2023-41993` correction: 31 Confirmed and one confirmed KEV.

The best instrumented full run measured 897 ms for an independent identity-resolution pass,
6,844 ms of stored pre-persistence evaluation time, and a 2,629 ms persistence/orchestration
residual inside the 9,473 ms matching wall. UI/IPC was not part of this direct Rust measurement.

| Product/stage | Post-change measurement | Cardinality/context |
|---|---:|---|
| Google Chrome bundle load | 1,824 ms | 35,925 criteria; 5,884 unique CVEs/configurations; 7,857,527 configuration bytes |
| Google Chrome applicability only | 451 ms | 5,884 grouped decisions and one configuration lookup per CVE |
| Google Chrome full isolated matching | 3,411 ms | dominant product cost; zero Confirmed |
| Oracle Java isolated matching | 1,241 ms | 6 Confirmed, including one KEV |
| Python isolated matching | 257 ms | 10 Confirmed |
| Node.js isolated matching | 238 ms | zero Confirmed |

The main causes were repeated tree evaluation for criteria already outside the installed range,
building full evidence for every criterion before CVE deduplication, deep cloning cache snapshots,
rewriting unchanged NVD/KEV materializations, and compiling match/evidence SQL repeatedly. The fix
groups criteria by unique CVE, performs cheap identity/range rejection before Boolean tree
evaluation, retains deterministic strongest-result/tie ordering, shares immutable snapshots,
skips materialization when the repository version is unchanged, caches bundles by product key, and
prepares persistence statements once per transaction. Parsed configurations and candidate coverage
were not removed.

## Deliberate limitations

- The NVD-derived cache is not a complete official CPE Dictionary.
- The alias/extractor registry is intentionally small and versioned.
- There is no fuzzy service, generic MSI/package conversion, or universal vendor-version grammar.
- Environment-dependent configurations remain unresolved/possible without observed facts.
- Specific unobserved CPE dimensions fail closed, which trades recall for precision.
- No real populated vulnerability is allowed to influence Security Score or Detection Engine in
  this sprint.

## Quality gates

- `pnpm lint`: passed.
- `pnpm typecheck`: passed.
- frontend/i18n tests: 17 files, 61 tests passed.
- React production build: passed, 1,901 modules; JS 487.46 kB / 141.79 kB gzip; CSS
  54.69 kB / 10.07 kB gzip.
- `cargo fmt --check`: passed.
- `cargo check --all-targets`: passed.
- `cargo clippy --all-targets -- -D warnings`: passed.
- Rust suite: 127 discovered, 114 passed, 0 failed, 13 explicit manual/live smokes ignored by
  default.
- focused identity/matching regressions: passed as part of the full Rust suite.
- `pnpm audit --audit-level=low`: no known vulnerabilities.
- `cargo audit` 0.22.2: no vulnerabilities; the same 17 allowed transitive maintenance/unsound
  warnings documented in `SECURITY.md` remain.
- Tauri release: passed; release EXE, MSI, and NSIS bundles generated.
- `sentinel.db`: schema v9, `integrity_check=ok`, foreign-key issues 0.
- `vulnerability-cache.db`: cache schema v1, `integrity_check=ok`, foreign-key issues 0.
- secret/encoding scans: no credential/private-key, U+FFFD, or common mojibake pattern found.
- `git diff --check`: passed; Git emitted only Windows line-ending conversion notices.

The Windows MSVC linker emitted its existing informational Portuguese import-library message; it
did not fail Check, Clippy, tests, or release packaging.

## Runtime UI validation

The rebuilt release executable opened as one real Tauri/WebView2 `EDY Sentinel` window against the
real application databases. Native Windows capture was unavailable because the host returned
`SetIsBorderRequired: interface not supported`; the safe fallback attached local CDP to that exact
release WebView2 page (`http://tauri.localhost/`). No isolated Vite/browser frontend was used.

Inventory search was exercised with Java, VirtualBox, Python, Chrome, Edge, and an unresolved
Antigravity record. Machine/user and identity filters, both software sort directions, and the
virtualized table scroll were exercised. Product Identity displayed original registry identity,
canonical vendor/product, original and normalized versions, full CPE, resolution method,
confidence, provenance, resolver version, and localized unresolved reason where applicable.

One real Confirmed CVE detail was opened for each product with current Confirmed matches:
`CVE-2023-41993` for Java, `CVE-2026-47047` for VirtualBox, and `CVE-2026-15308` for Python. Each
showed NVD description, CVSS/version, NVD severity, installed version, affected range, selected NVD
CPE, matching result, confidence, source timestamps, evidence JSON, and references. The KEV detail
now states explicitly that KEV is prioritization enrichment and did not establish the match.

Runtime language switching passed in pt-BR and English with no literal i18n keys or new-screen
placeholders. Sentinel Blue, Cyber Green, Terminal, and Spectrum passed. Inventory/table and drawer
behavior passed at 1920x1080, 1600x900, 1440x900, 1366x768, and 1280x720; the 1280 table has no
horizontal overflow.

Small presentation defects fixed during visual QA:

- raw lowercase `high`/`low` confidence labels in match cards were localized;
- CVE Detail now shows the exact selected NVD CPE from immutable evidence rather than a product
  prefix;
- the KEV role is explicit in pt-BR and English;
- drawer horizontal overflow and the clipped Confirmed badge were corrected without redesign.

Real screenshots were written outside Git under
`C:\Users\<USER>\.codex\visualizations\2026\08\17\01a0113f-c75f-73d1-a1df-15df76b3982e\sprint3_2_2c_visual_qa`:

1. `01_inventory_ptbr_sentinel_blue_1920x1080.png`
2. `02_product_identity_java_ptbr_1920x1080.png`
3. `03_cve_detail_virtualbox_ptbr_1600x900.png`
4. `04_kev_cve_detail_java_ptbr_1920x1080.png`
5. `05_inventory_english_sentinel_blue_1600x900.png`
6. `06_cve_detail_python_english_1440x900.png`
7. `07_inventory_english_cyber_green_1600x900.png`
8. `08_inventory_english_terminal_1440x900.png`

Additional responsive evidence is `09_inventory_english_spectrum_1366x768.png` and
`10_inventory_english_spectrum_1280x720.png`.

## Recommendation

**READY FOR VULNERABILITY → SECURITY SCORE DESIGN.** Visual QA, auditability, language/theme
coverage, responsive layouts, release packaging, and database integrity are complete for review.
Security Score integration has not started; Security Score, Detection Engine, the six rules,
severity, and remediation remain unchanged and isolated.
