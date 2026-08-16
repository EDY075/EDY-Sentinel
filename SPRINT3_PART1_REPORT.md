# Sprint 3 Part 1 Report — Software Inventory and Vulnerability Data Foundations

Date: 2026-08-16
Initial HEAD: `44c9910338beda48ae1c26d35c1ec3048d342cff`
Branch: `main`
Scope: inventory and vulnerability-source foundations only; no software-to-CVE matching.

## Outcome

Sprint 3 Part 1 adds a real on-demand Windows installed-software inventory and two separate
local vulnerability repositories for NVD and CISA KEV. The application preserves the existing
Observation → Event → Detection boundary: software changes are factual events with no severity,
and neither NVD nor KEV records affect detections or Security Score.

The worktree intentionally remains uncommitted for review. No screenshot was generated and
Sprint 3 Part 2 was not started.

## Software Inventory

Rust reads these native uninstall Registry views without PowerShell or shell execution:

- HKLM, 64-bit native view
- HKLM, 32-bit WOW64 view
- HKCU, 64-bit native view
- HKCU, 32-bit WOW64 view

The collector preserves display name, version, publisher, location, install date, architecture,
machine/user scope, exact source, Registry identity, and a strict MSI-style product code when the
uninstall key itself is a GUID. Missing values remain absent.

Normalization is separate from display data. Vendor, product, version, architecture, and scope
are prepared for future matching, but identity remains `Unresolved` unless strict evidence is
present. A display name is never treated as product identity.

Deduplication merges only the same strict product code in the same install scope. Otherwise the
exact Registry identity stays separate, including two entries with equal or similar names.

Native smoke on this host found:

| Metric | Result |
|---|---:|
| Raw Registry entries | 81 |
| Deduplicated installed software | 80 |
| Accessible Registry sources | 4 |
| Collection time | 6 ms |

Inventory is collected only when the repository is empty on first Inventory use or when the user
requests Refresh Inventory. It is not part of the 2.5-second live telemetry cadence.

## Inventory UI

The former placeholder is now a functional, virtualized Inventory view with search, exact
machine/user and resolved/unresolved filters, sorting, keyboard navigation, and a detail drawer.
The drawer shows raw installation facts, sources, Registry identities, normalized identity, and
the explicit boundary “Vulnerability matching not evaluated yet.”

English and Português (Brasil) catalogs have exact key/interpolation parity. All new styling uses
existing semantic tokens, so Sentinel Blue, Cyber Green, Terminal, and Spectrum inherit their
approved color/contrast contracts without theme-specific branches.

## Change Tracking

Migration 0007 stores inventory snapshots, current installed rows, and append-only observations.
After the initial seed, comparisons can create only:

- `software_installed`
- `software_removed`
- `software_version_changed`

The dedicated factual-event table has no severity column. The same facts enter the central
Security Events/provenance foundation as software observations, with no rule ID, confidence,
baseline claim, or threat classification. Initial discovery does not manufacture install events.

## NVD Provider

`src-tauri/src/vulnerability.rs` is the sole HTTP/provider boundary. The NVD implementation uses
the official CVE API 2.0 endpoint:
`https://services.nvd.nist.gov/rest/json/cves/2.0`.

Behavior:

- HTTPS-only Rust client; no frontend HTTP and no API key required or hardcoded
- 45-second request timeout and 64 MiB decompressed-response cap
- 2,000-record cursor pages
- six seconds between public-client pages, honoring the official 5 requests / 30 seconds limit
- three bounded retries with cancellable exponential backoff
- resumable cursor for an interrupted initial sync
- incremental `lastModStartDate`/`lastModEndDate` windows of 119 days
- successful sync not repeated within two hours
- cooperative cancellation between pages, waits, and retries
- transactional upsert cache and sanitized lifecycle errors

The minimal local record preserves CVE ID, published/modified timestamps, English description,
CVSS score/version/severity, weaknesses, bounded references, vulnerability status/source, and
bounded CPE applicability/version-range fields. Raw provider payloads are not retained.

Controlled fixture synchronization persisted 2 NVD records and verified incremental overwrite.
The manual official-provider smoke parsed 5 live records from a current total of 378,386 in
576 ms. A full public initial download was intentionally not performed during the quality gate;
at the mandatory public cadence it has a minimum request-delay cost of roughly 19 minutes for the
current page count, excluding transfer and persistence time.

Official documentation used:

- https://nvd.nist.gov/developers/vulnerabilities
- https://nvd.nist.gov/developers/start-here

## CISA KEV Provider

CISA KEV uses a separate HTTPS fetch, parser, transaction, table, and provider lifecycle. The
authoritative feed is:
`https://www.cisa.gov/sites/default/files/feeds/known_exploited_vulnerabilities.json`.

The cache preserves CVE ID, vendor/project, product, vulnerability name, date added, due date,
required action, known ransomware campaign use, notes, and CWEs. CVE and ISO date fields are
validated; duplicate CVEs are deterministically collapsed before the authoritative replacement
transaction.

The fixture test reduced 3 source entries to 2 unique persisted CVEs. The manual official-provider
smoke synchronized and persisted 1,665 current KEV records into an isolated in-memory schema in
1.345 seconds.

KEV membership is stored only as source context. It does not imply that local software matches a
CVE and does not create Critical severity.

## Provider Status and Offline Behavior

Settings now shows NVD and CISA KEV independently with Idle/Ready/Updating/Error state, last
successful sync, local record count, Synchronize, and Cancel. Sync runs on Tauri blocking workers,
never the UI thread. Existing local data remains queryable after a network failure; only the
external update fails. An interrupted process state is recovered as a sanitized error on restart
without deleting the cache.

The real application database was left with both providers `idle` and zero records because no
long-lived user repository sync was initiated. Live provider gates used isolated in-memory
databases and did not change the user's cache.

## Database

Migration `0007_software_inventory_vulnerability_repository.sql` is append-only relative to the
previous ledger and adds seven tables:

1. `software_inventory_snapshots`
2. `installed_software`
3. `software_inventory_observations`
4. `software_inventory_events`
5. `nvd_vulnerabilities`
6. `cisa_kev_vulnerabilities`
7. `vulnerability_provider_state`

Real release-startup validation:

| Check | Result |
|---|---|
| Migrations | 1, 2, 3, 4, 5, 6, 7 |
| Schema version | 7 |
| Journal mode | WAL |
| Database size | 40,431,616 bytes |
| `integrity_check` | `ok` |
| `foreign_key_check` | 0 rows |

Migration tests also confirm schema-v1-to-v7 upgrade, provider constraints, append-only facts,
no severity column on software events, and foreign-key/integrity validity.

## Performance and Runtime

- Inventory is on demand and completed in 6 ms for 81 raw entries on this host.
- NVD live first-page validation: 576 ms for 5 records; between-page waits are six seconds.
- CISA KEV live isolated sync: 1.345 seconds for 1,665 records.
- External response memory is bounded to 64 MiB for NVD and 16 MiB for KEV before parsing.
- Release startup after 8 seconds: 44,630,016-byte working set, 18,874,368-byte private memory,
  1.812 CPU seconds, and 26 threads. This is a point-in-time smoke metric, not a benchmark.
- Production frontend bundle: 469.27 kB JavaScript / 137.19 kB gzip and 50.63 kB CSS /
  9.53 kB gzip.

## Verification Gates

- `pnpm lint` — pass
- `pnpm typecheck` — pass
- `pnpm test` — 17 files / 61 tests pass
- i18n parity/interpolation verification — pass as part of frontend tests
- `pnpm build` — pass
- `cargo fmt --all -- --check` — pass
- `cargo check --all-targets` — pass
- `cargo clippy --all-targets -- -D warnings` — pass
- `cargo test` — 82 pass / 3 explicit manual smokes ignored in the default run
- Native Inventory smoke — pass, 80 software records
- Manual NVD official-provider smoke — pass
- Manual CISA KEV official-provider smoke — pass
- `pnpm audit --audit-level=low` — no known vulnerabilities
- `cargo audit 0.22.2` — no vulnerabilities; 17 allowed transitive warnings remain
- React production build — pass
- Tauri release build — EXE, MSI, and NSIS pass
- Release startup smoke — pass
- SQLite integrity/foreign keys — pass
- `git diff --check` — pass

The linker emits the same localized informational message about creating the Windows import
library as previous releases; it is not a compile or Clippy warning in application code.

## Release Artifacts

| Artifact | Size | SHA-256 |
|---|---:|---|
| `src-tauri/target/release/edy-sentinel.exe` | 15,353,856 bytes | `81AA43ADB154EFD42DC7434ED66D8441016AC4B0A1F70EAAF5CB94C9B78B7FB7` |
| `src-tauri/target/release/bundle/msi/EDY Sentinel_0.1.0_x64_en-US.msi` | 5,750,784 bytes | `40EE43CB33F2DF6D04A3131A3A8EE930A2864DD2A6765560A2E584998157EE1A` |
| `src-tauri/target/release/bundle/nsis/EDY Sentinel_0.1.0_x64-setup.exe` | 4,129,931 bytes | `2E0765C2EFF100ADE5A58E578B94B0A7B68D6AC9AD5F114570A24EF68823A6BA` |

## Limitations and Deferred Work

- No software-to-CVE association exists. Display-name substring matching is explicitly absent.
- Most non-MSI Registry entries remain `Unresolved`; publisher metadata is not proof of vendor
  identity, and Registry architecture is only used when the view makes it determinable.
- Per-user inventory covers the current user's HKCU, not every profile on the machine.
- Store/MSIX/AppX packages outside these uninstall Registry sources are not collected in Part 1.
- An in-flight HTTPS request cannot be interrupted inside the networking library; cancellation
  takes effect at the next cooperative boundary and the request itself is capped at 45 seconds.
- The NVD public initial sync is intentionally slow under official limits. No API-key credential
  flow exists yet.
- No CVE matching, CPE decision engine, vulnerability Detection rule, score penalty, VirusTotal,
  AbuseIPDB, HIBP, urlscan, AI, auto-update, or remediation was added.

Sprint 3 Part 2 must begin from evidence-based identity/CPE/version-range matching with explicit
confidence and provenance. It must not convert this Part 1 repository into name-based claims.
