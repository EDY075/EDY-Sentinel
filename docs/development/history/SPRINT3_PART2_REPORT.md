# Sprint 3 Part 2 Report

Date: 2026-08-16  
Project: `D:\Projects\EDY-Sentinel`  
Part 1 checkpoint: `8ffedbe2908ced31acc0aababa9b30fb6ddf0d8f`  
Part 2 final commit: intentionally not created

## Outcome

Sprint 3 Part 2 implements the local conservative chain:

`installed software → identity → CPE candidates → version range → CVE → confidence → KEV → evidence → UI`

Matching engine version is `1`. Only a single, non-ambiguous `High` identity with a safely interpreted version and a true NVD applicability tree can become `Confirmed`.

## Git checkpoint

Sprint 3 Part 1 was audited, all essential gates passed, and the exact checkpoint commit was created:

`8ffedbe2908ced31acc0aababa9b30fb6ddf0d8f feat: add software inventory and vulnerability data foundation`

The tree was clean before Part 2 began. Part 2 remains intentionally uncommitted for review.

## Architecture and schema v8

Migration `0008_vulnerability_matching.sql` is append-only and upgrades the schema to v8. It adds:

- full NVD `configurations_json` preservation;
- indexed `nvd_cpe_matches` metadata;
- immutable `software_vulnerability_evaluations`;
- immutable identity snapshots;
- evaluation-scoped CPE candidates;
- vulnerability matches with state, confidence, engine/source versions, and evaluation time;
- append-only matching evidence;
- incremental evaluation queue.

All access remains in Rust through narrow typed Tauri commands. React is presentation-only. Evaluation runs in a blocking worker and reads only the local SQLite repository.

## Matching policy

The detailed policy is documented in [`../../technical/vulnerability-matching.md`](../../technical/vulnerability-matching.md).

- No CVE description matching.
- No `contains`-style confirmation.
- Different vendor rejects the candidate.
- Missing publisher can yield only a low product-only candidate.
- Multiple candidates are all ambiguous; none is chosen silently.
- Unknown version yields `Unresolved`.
- Unproven NVD environment conditions yield `Possible`.
- `Medium` and `Low` never become `Confirmed` in v1.
- KEV enriches only after CVE association and never changes the match state.

## Version engine and false-positive fixtures

Rust tests cover:

- numeric multi-component comparison;
- `1.2 == 1.2.0`;
- start including and excluding;
- end including and excluding;
- exact lower and upper boundaries;
- versions below and above the range;
- exact CPE version;
- fixed version producing `Not affected`;
- unknown and unsafe version producing `Unresolved`;
- similar product name without identity evidence;
- incompatible vendor;
- multiple candidates;
- NVD `AND` environment context remaining unknown;
- confirmed fixture with persisted provenance;
- repeated evaluation preserving historical candidates.

Calibration fixes made during validation:

1. Corrected strict start/end excluded boundary semantics.
2. Scoped CPE candidate IDs to each immutable evaluation.
3. Restricted `Confirmed` to `High` confidence only.
4. Preserved the full NVD configuration tree instead of relying on the v7 flattened list.
5. Required old v7 cache rows to be refreshed rather than silently treating missing applicability logic as true.
6. Deduplicated semantically identical NVD CPE-match records during page upsert.
7. Treated the CPE version value `-` as not applicable, never as the wildcard `*`.
8. Failed closed when `*` has no explicit version range; that evidence can no longer produce `Confirmed`.
9. Scoped the Microsoft Edge aliases by product generation: legacy `edge` through major 18 and `edge_chromium` from major 79.

## UI and i18n

Inventory now shows real persisted values for:

- confirmed vulnerability count;
- possible match count;
- highest confirmed CVSS;
- confirmed KEV count;
- evaluation status.

Software detail separates confirmed vulnerabilities from possible matches. The reusable CVE detail presents NVD metadata, installed version, CPE, affected range, confidence, comparison result, KEV context, source versions, append-only evidence, and user-clicked references.

English and pt-BR catalogs have matching keys. CVE IDs, CPEs, versions, vendors, products, URLs, CVSS values/vectors, paths, and registry identities remain untranslated.

## Real Windows smoke

The manual ignored Rust smoke was explicitly executed against the real application database and native Windows registry inventory.

| Metric | Result |
|---|---:|
| Installed software | 80 |
| Inventory collection | 7 ms |
| Software evaluated | 80 |
| Matching wall time | 908 ms |
| Average per software | 11.35 ms |
| Candidate CVEs | 0 |
| Confirmed software | 0 |
| Possible software | 0 |
| Unresolved software | 80 |
| Not affected software | 0 |
| Confirmed CVEs | 0 |
| Confirmed KEV CVEs | 0 |
| Process RAM snapshot | 20,299,776 bytes (19.36 MiB) |
| Process CPU snapshot | 0.0% |
| SQLite size | 40,857,600 bytes (38.96 MiB) |
| Schema | v8 |
| `integrity_check` | `ok` |
| Foreign-key issues | 0 |

The result is expected: the real NVD repository is currently empty/not synchronized. The engine therefore produced no candidates and conservatively classified all 80 products as unresolved. It did not manufacture vulnerabilities from names.

## Manual validation

No real confirmed match existed, which is acceptable and explicitly permitted by the sprint requirement. Manual inspection used real inventory examples including Google Chrome, Microsoft Edge, and WinRAR. Product/version/publisher facts were present, but without synchronized local NVD CPE/configuration records no CVE, range, or CPE association was asserted.

The controlled persisted fixture `Google Chrome 1.5 / Google LLC → cpe:2.3:a:google:chrome → CVE-2026-10001 / < 2.0` was separately validated in the Rust suite. It confirms only with the complete configuration tree and becomes `Not affected` at `2.0`.

## Performance design

The engine uses `nvd_cpe_matches(normalized_product, normalized_vendor, cve_id)` and evaluation/match indexes. It does not execute an NVD request per software and does not scan CVE description text. A bounded queue avoids continuous full-database reprocessing.

The real empty-repository run is retained as the before-sync control. The full-cache benchmark and recalibration are recorded below.

## REAL NVD VALIDATION

### Sync

The existing provider performed a complete unauthenticated NVD 2.0 synchronization. No API key was required or exposed to React. The provider used its existing 2,000-record pages, six-second public rate limit, bounded retry/backoff, cooperative cancellation, and persisted cursor.

| Metric | Result |
|---|---:|
| NVD CVEs persisted | 378,386 |
| NVD pages | 190 |
| Provider elapsed time | 2,072,465 ms (34m 32.465s) |
| End-to-end test time including integrity/FK checks | 2,125,460 ms (35m 25.460s) |
| Last successful NVD sync | `2026-08-16T16:10:59.401196600Z` |
| SQLite before sync | 40,857,600 bytes (38.96 MiB) |
| SQLite immediately after NVD sync | 3,509,612,544 bytes (3.27 GiB) |
| Indexed NVD CPE matches | 3,125,419 |
| CISA KEV records | 1,665 |
| KEV sync | 1,499 ms |
| Last successful KEV sync | `2026-08-16T16:12:19.195313200Z` |
| `integrity_check` | `ok` |
| Foreign-key issues | 0 |

The first NVD attempt committed its first page (2,000 CVEs) and then stopped safely on a real duplicate CPE-match identity from the upstream data. The cache and cursor were preserved. After an idempotent semantic-deduplication fix, the same provider resumed from cursor 2,000 and completed without deleting or restarting the repository. Progress was observed from the persisted provider state (`recordsProcessed`, `pagesProcessed`, `lastSuccessfulPage`, status, and elapsed time). The Settings UI polls that state every two seconds while a sync is active and exposes the existing cooperative cancel action.

Only normalized/selected CVE fields, bounded references/weaknesses, the NVD configuration tree required for applicability, and the indexed CPE metadata are stored. Raw HTTP response pages are not retained.

### Matching

The final run evaluated the native Windows inventory after both NVD and KEV were ready.

| Metric | Result |
|---|---:|
| Installed software evaluated | 80 |
| Confirmed software | 0 |
| Possible software | 0 |
| Unresolved software | 73 |
| Not affected software | 7 |
| Confirmed CVEs | 0 |
| Confirmed KEV CVEs | 0 |
| CPE identity candidates | 9 |
| Candidate CVE rows examined | 6,294 |
| Distinct KEV CVEs among non-confirmed candidates | 76 |

Zero confirmed matches is the accepted factual result. KEV metadata joined by exact CVE ID, but did not change any match state.

### Manual Validation

No final `Confirmed` or `Possible` match exists, so there is no defensible applicable-CVE drawer sample to fabricate. The real identities most useful for calibration were:

| Software | Installed version | Reviewed identity | Final result | Reason |
|---|---|---|---|---|
| Google Chrome | `151.0.7922.138` | `google / chrome` | Unresolved | 5,863 CPE/CVE criteria were outside the installed range; 21 wildcard/insufficient criteria failed closed. |
| Microsoft Edge | `151.0.4129.86` | `microsoft / edge_chromium` | Unresolved | 293 criteria were outside the installed range; 2 insufficient criteria failed closed. Legacy `microsoft / edge` was explicitly rejected for this major version. |
| Git | `2.55.0.3` | controlled `git-scm / git` identity | Not affected | All 41 examined criteria were outside the installed version/applicability. |

The real NVD identity probe confirmed distinct Microsoft product families (`edge`, `edge_chromium`, `edge_ios`, and `edge_update`) and 5,886 Chrome CVEs in the local index. The KEV probe sampled current catalog entries and confirmed that each exact CVE ID was present in NVD.

### False Positives

The first populated run produced 233 confirmed CVEs across Chrome and Edge. Manual inspection rejected the result. Three conservative fixes reduced the intermediate count to 82 and then the final count to zero:

1. NVD criteria such as `cpe:2.3:a:google:chrome:-:...` used `-` (not applicable), but v1 incorrectly treated it like `*`. The comparison now returns `Not affected` for an installed concrete version.
2. A bare `*` without a start/end range was too weak to prove that a current installed build remained affected. It now returns an unresolved comparison rather than `Affected`.
3. The generic `Microsoft Edge` registry name was mapped to the legacy `microsoft:edge` CPE. The controlled alias is now version-scoped and resolves major 151 only to `microsoft:edge_chromium`.

Regression tests cover CPE `-`, unbounded wildcard failure, and legacy/Chromium Edge version boundaries. The existing different-vendor, fixed-version, unknown-version, multiple-candidate, medium-confidence, and environmental-context fixtures continue to pass.

### Performance

| Metric | Result |
|---|---:|
| Full evaluation wall time | 15,102 ms |
| Wall average (80 products) | 188.78 ms/software |
| Persisted per-software average | 171.81 ms |
| Persisted p50 | 0 ms |
| Persisted p95 | 1 ms |
| Process CPU over evaluation interval | 96.28% |
| Process RAM snapshot | 35,332,096 bytes (33.69 MiB) |
| SQLite after matching history | 3,542,097,920 bytes (3.30 GiB) |

Per-software timings are persisted in whole milliseconds, so p50/p95 round to zero/one millisecond; the total is dominated by the small number of indexed browser candidate sets. `EXPLAIN QUERY PLAN` confirmed `SEARCH m USING COVERING INDEX idx_nvd_cpe_identity (normalized_product=? AND normalized_vendor=?)`, followed by the NVD CVE primary-key index. There is no full description scan or N×M product/CVE loop. The manual smoke's 69.68-second process duration included a full 3.5 GB `integrity_check`; matching itself was 15.102 seconds.

### Runtime UI validation

The current release executable was built and opened as a real Tauri/WebView2 application against the real application database. Its accessibility tree confirmed the pt-BR runtime, live telemetry, and `SQLite v8` persistence state. The release did not remain in an infinite provider-loading state. Repeated desktop minimization/user-input detection prevented safe click automation into Inventory, so no click-level claim or screenshot was manufactured. Because there were no final applicable `Confirmed` or `Possible` CVEs, the real CVE drawer had no valid record to open; English/pt-BR drawer contracts remain covered by the passing frontend/i18n suite.

### Limitations

- The local repository is large (about 3.3 GiB) because full NVD applicability trees and 3.1 million indexed CPE matches are retained for offline decisions.
- Bare wildcard criteria intentionally remain unresolved even when NVD encodes `*`; this trades recall for the required precision.
- Only three installed products matched the deliberately small controlled alias set in this host validation. Other real products remain unresolved instead of being forced to a CPE.
- No final applicable CVE existed, so a real populated CVE detail drawer could not be exercised without fabricating a match.
- The release UI was opened and inspected, but Inventory click-through was not safely automatable while the desktop repeatedly minimized/changed focus.
- Vulnerability results remain isolated from Security Score, Detection Engine, rule severity, and remediation.

## Quality gates

- `pnpm lint`: pass.
- `pnpm typecheck`: pass.
- frontend/i18n tests: 17 files, 61 tests passed.
- React production build: pass, 1,901 modules; JS 483.29 kB / 140.51 kB gzip; CSS 54.29 kB / 10.02 kB gzip.
- `cargo fmt --check`: pass.
- `cargo check --all-targets`: pass.
- `cargo clippy --all-targets -- -D warnings`: pass.
- Rust tests: 92 passed, 0 failed, 9 manual/network smokes ignored by default.
- real matching smoke: explicitly executed and passed.
- `pnpm audit`: no known vulnerabilities.
- `cargo audit` 0.22.2: no vulnerabilities; 17 existing allowed transitive warnings (GTK3/unicode/proc-macro maintenance and `glib` advisory), unchanged upgrade debt.
- Tauri release: pass; release EXE plus MSI and NSIS bundles generated in ignored build output.
- SQLite schema v8, `integrity_check=ok`, `foreign_key_check=0`.
- secret scan: no credential/private-key pattern found.
- encoding scan: no U+FFFD or common mojibake pattern found.
- `git diff --check`: pass; only line-ending conversion notices from Git.

The Windows MSVC linker emitted its existing informational Portuguese message while creating the import library; it did not fail any build or Clippy gate.

## Scope preserved

The following were intentionally not changed:

- Security Score formula or inputs;
- Detection Engine behavior;
- the six detection rules or their severity;
- automatic remediation or software update;
- external enrichment services beyond NVD and CISA KEV.

## Remaining boundaries

- Existing rows cached under v7 have no full configuration tree until refreshed by the v8 provider path.
- Product-specific registry/package version conversion is not implemented.
- Complex environment-dependent NVD configurations remain possible/unresolved when their platform facts are absent.
- The current alias registry is intentionally small; expansion requires reviewed fixtures for vendor, product, version, ambiguity, and false-positive behavior.
- Vulnerability results remain isolated from Security Score and detection severity; integration requires a separate product/security decision after this calibration.

## Sprint 3 Part 2.2A — Storage, Performance & NVD Sync Hardening

### Checkpoint and scope

Part 2 was checkpointed on `main` as `f9991a937ad40bcbde0749b987123afc2988551f`
(`feat: add conservative vulnerability matching engine v1`) with a clean worktree before 2.2A.
No alias, CPE-coverage, Node/Python/Java version mapping, Detection Engine, Security Score,
six-rule, or remediation behavior was changed.

### Isolated cache and migration

`vulnerability-cache.db` now owns complete NVD/KEV data, compact CPE dictionaries and
relationships, provider generations, sync checkpoints and a durable change outbox. It has its own
checksummed schema-v1 migration. `sentinel.db` remains schema v8 and owns inventory, evaluations,
matches and evidence. Only CVE/KEV records referenced by historical matches remain in the main
database as offline evidence snapshots; their configuration and flattened applicability fields are
empty by design.

The real legacy import completed in 328.33 seconds and preserved exact repository counts:

| Cache record | Count |
|---|---:|
| NVD CVEs | 378,386 |
| CPE relationships | 3,125,419 |
| Distinct CPE criteria | 427,514 |
| CISA KEV entries | 1,665 |

The pre-migration backup outside the repository is schema v8, `integrity_check=ok`, FK=0,
3,556,425,728 bytes, SHA-256
`459D49941A3A250978EAA3AB72C597795127CB9E5AF91EF2371ABB87C737CEDF`. A second verified
post-regression/pre-cleanup backup is 3,557,007,360 bytes, integrity `ok`, FK=0, SHA-256
`0AAEC1B63B01E97D2A2400F114E331E1795F01412F57B54E7F22299761C73F8D`.

### Storage result

| Metric | Before | After |
|---|---:|---:|
| `sentinel.db` | 3,556,425,728 B | 111,763,456 B |
| `vulnerability-cache.db` | — | 801,083,392 B |
| Combined SQLite files | 3,556,425,728 B | 912,846,848 B |
| Absolute reduction | — | 2,643,578,880 B |
| Total reduction | — | 74.33% |
| Main-database reduction | — | 96.86% |

Final real main-database state: 81 active software records (qBittorrent 5.2.3 was newly observed
after the 80-item checkpoint), 6,956 referenced CVE snapshots, 84 referenced KEV snapshots, zero
legacy CPE rows, `integrity_check=ok`, FK=0. The external cache also passed full integrity and FK
checks. No database, backup or cache artifact is tracked by Git.

### Matching query and performance

The matching query now runs in two indexed phases: relevant compact CPE relationships, then unique
CVEs and their single configuration/content payload. A configuration is decompressed and parsed once
per CVE and reused for every relevant criterion. Invalid/incomplete/hash-mismatched payloads fail
closed before any main-database evaluation transaction begins.

| Metric | Part 2 before | 2.2A after |
|---|---:|---:|
| Same 80 products, wall | 15,102 ms | 6,450 ms |
| Same 80 products, wall reduction | — | 57.29% |
| Same 80 products, persisted average | 171.81 ms | 48.40 ms |
| Same 80 products, p50 | 0 ms | 0 ms |
| Same 80 products, p95 | 1 ms | 7 ms |
| Chrome isolated wall | ~13,674 ms | 5,154 ms |
| Chrome isolated wall reduction | — | 62.31% |
| Chrome bundle load | not separated | 1,856 ms |
| Chrome criterion rows | 35,925 | 35,925 |
| Chrome unique configurations/parses | repeated per criterion | 5,884 |
| Chrome configuration bytes deserialized | ~439.86 MiB returned repeatedly | 7,857,527 B |

The same-80 post-compaction process snapshot used 28,950,528 bytes of RAM and 88.41% CPU during
the interval. Whole-millisecond per-software persistence explains the zero p50; the increased p95
reflects cheap zlib/content snapshot work on a few candidates while total wall and browser cost fell
materially. The current 81-item real host run also passed in 6,259 ms with `0/0/74/7` because the
new qBittorrent record remained conservatively unresolved.

### Matching regression

The exact 80-item checkpoint inventory was evaluated from a byte-identical working copy before and
after main-database cleanup. Both runs produced:

| State | Before | After |
|---|---:|---:|
| Confirmed | 0 | 0 |
| Possible | 0 | 0 |
| Unresolved | 73 | 73 |
| Not affected | 7 | 7 |
| Candidate CVE decisions | 6,294 | 6,294 |

Matching Engine version remains 1. High-only confirmation, incompatible-vendor rejection,
multiple-candidate ambiguity, unsafe-version failure, CPE `-`, unbounded wildcard failure,
AND/OR/negate, vulnerable=false, version ranges and KEV-as-enrichment semantics are unchanged.

### Sync hardening

- Full sync keeps a relational page checkpoint and resumes without restarting.
- Incremental sync persists mode, generation, fixed window start/end, next `startIndex`, pages,
  totals and provider cursor.
- Each NVD page and checkpoint advance commits atomically.
- Incremental windows are at most 119 days with a five-minute overlap.
- The successful high-water mark is the request limit captured before network work, never the later
  completion clock.
- Upsert by CVE and modified time makes overlap/replay idempotent and prevents older payloads from
  replacing newer CVE/configuration/CPE content.
- Ready generation, provider state, checkpoint removal and outbox creation commit together. Outbox
  delivery to the main re-evaluation queue is idempotent and recovered after a crash.

Final provider state is `ready` for NVD (378,386 records) and CISA KEV (1,665 records), with zero
pending sync checkpoints and zero undelivered outbox records. Resume/window/high-water behavior is
covered by the passing focused Rust tests; no second 378,386-record full download was performed.

### Failure safety and remaining limits

Cache open, schema, quick-check, FK, codec, size and SHA-256 failures return explicit cache errors.
The main application still opens and Inventory, baseline, Detection Engine, Security Score and the
six rules remain usable. Settings has localized English/pt-BR cache-unavailable copy.

Remaining limits are deliberate: the cache still stores the full NVD Boolean tree as compressed
JSON rather than a normalized AST; the NVD-derived CPE set is not the complete official Dictionary;
product-specific version extractors and additional aliases remain deferred; environment-dependent
facts remain unresolved/possible; and vulnerability results remain disconnected from Detection
Engine, Security Score and remediation.

### 2.2A final quality gates

- `cargo fmt --check`: passed.
- `cargo check --all-targets --offline`: passed.
- `cargo clippy --all-targets --offline -- -D warnings`: passed.
- Rust suite: 119 tests discovered, 108 passed, 0 failed, 11 explicit live/manual tests ignored by default.
- Focused cache/migration tests: passed, including codec bounds/hash validation and the invariant that
  an already-ready external cache remains authoritative after legacy main-database compaction.
- Focused sync tests: passed, including retained interrupted checkpoints, fixed incremental windows,
  page resume and request high-water semantics.
- Frontend lint and TypeScript checks: passed.
- Frontend/i18n suite: 17 files, 61 tests passed.
- React production build: passed, 1,901 modules; JS 483.67 kB / 140.62 kB gzip; CSS 54.29 kB /
  10.02 kB gzip.
- `pnpm audit`: no known vulnerabilities.
- `cargo audit`: no vulnerabilities; the same 17 explicitly allowed transitive maintenance warnings
  remain upgrade debt.
- Tauri release build: passed; EXE, MSI and NSIS artifacts generated in ignored build output.
- Final release smoke: passed against the compact databases; reopening did not mutate or re-import
  the already-ready 801,083,392-byte cache.
- Final SQLite checks: both databases `integrity_check=ok`, FK=0, freelist=0 after compaction and WAL=0.
- Secret and encoding scans: no credential/private-key pattern, U+FFFD or common mojibake detected.
- `git diff --check`: passed; Git reported only Windows line-ending conversion notices.
