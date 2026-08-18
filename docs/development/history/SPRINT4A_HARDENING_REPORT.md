# EDY Sentinel — Sprint 4A Hardening Report

Audit date: 2026-08-17

Scope: adversarial technical audit before Release v1.0

Baseline: `main` at `29d13bc352206107ee3c57d16b79d9c174186add`

Final audit worktree: intentionally uncommitted for review

## Executive result

The product pipeline is functionally coherent and the audit corrected five
conservative defects: degraded-cache startup, provider sync-lock recovery,
external HTTP references, dialog initial focus, and command-palette state.
Telemetry, baseline, detection, vulnerability matching, and Security Score v2
remain internally consistent. The current 31 Confirmed CVEs remain auditable.

Release v1.0 is still blocked because every generated distribution artifact is
unsigned. The MSI is also a per-machine installer and therefore has a distinct
installation elevation boundary even though normal application runtime remains
standard-user.

## Findings by severity

### BLOCKER

None.

### HIGH

#### H-01 — Reconstructible vulnerability cache could abort startup — FIXED

- `VulnerabilityCache::open` deliberately returned a degraded handle for an
  absent, corrupt, inaccessible, or newer-schema cache.
- Application setup then propagated `recover_interrupted_syncs(...)?`; the
  recovery transaction failed on that same degraded handle and aborted Tauri
  setup.
- Startup recovery is now best-effort. Successful-cache recovery is unchanged,
  while the degraded status/error remains available to provider and coverage UI.
- Regression test:
  `vulnerability::tests::unavailable_cache_does_not_block_startup_recovery`.

#### H-02 — EXE, MSI, and NSIS artifacts have no Authenticode signature — OPEN

- `Get-AuthenticodeSignature` returns `NotSigned` for all three final artifacts.
- Public distribution would lack publisher authentication, trusted timestamping,
  and an integrity chain suitable for Windows reputation/SmartScreen decisions.
- A legitimate fix requires a release certificate, protected signing key,
  timestamp service, and CI/release verification policy. None was provided, so
  the audit did not fabricate or self-sign release trust.
- This finding must be closed before Release v1.0.

### MEDIUM

#### M-01 — Vulnerability evaluation history has no bounded retention — OPEN

- `sentinel.db` contains 1,220 evaluation records and 137,990 historical match
  rows and grew from about 348 MB at audit start to about 353.2 MB after audit
  activity.
- Telemetry, Security Events, and score snapshots have retention policies;
  vulnerability evaluations/matches are append-only without pruning or
  downsampling.
- Repeated full/provider evaluations can grow the primary database without a
  defined bound. This audit did not delete evidence because that would change
  historical audit semantics without an approved retention design.

#### M-02 — Provider sync lock survived exceptional worker termination — FIXED

- `finish()` previously ran only inside the blocking worker closure.
- A worker panic/join failure could leave the manager permanently `running`
  until process restart.
- The command now releases the lock after the awaited join on every result path.
  Provider transaction and checkpoint semantics are unchanged.

#### M-03 — Dialog container overrode intended initial focus — FIXED

- Real Tauri QA showed that `Ctrl+K` opened a correctly named dialog but direct
  typing did not reach the declared autofocus search field.
- The generic dialog effect focused the container after React mounted the input.
- Dialogs now prefer an explicit `data-initial-focus` target, then the first
  enabled focusable control, then the container. The palette and baseline
  confirmation input declare the target.
- Release revalidation: typing `Inventory` immediately after `Ctrl+K` reduced
  the command list to Software Inventory, and Enter opened the real 81-row grid.

### LOW

#### L-01 — Plaintext HTTP advisory links were clickable — FIXED

- Provider downloads were already HTTPS-only, but NVD references accepted both
  HTTP and HTTPS. The repository has 62,140 HTTP references in total; four were
  attached to currently confirmed CVEs.
- New ingestion and CVE Detail now allow only credential-free HTTPS references.
  Presentation filtering protects pre-existing cache content without rewriting
  matching evidence.
- Rust and frontend regression tests cover HTTP, credential-bearing HTTPS,
  `javascript:`, malformed URLs, and valid HTTPS.

#### L-02 — Command palette retained the previous query — FIXED

- Reopening the palette after `Inventory` retained that query, so typing
  `Settings` produced `InventorySettings`.
- Both keyboard and top-bar open paths now clear the query.
- Release revalidation confirmed all commands are present after reopening.

#### L-03 — Security analysis is duplicated on the 15-second system cycle — OPEN

- Live telemetry runs every 2.5 seconds and system telemetry every 15 seconds.
  Both backend commands run Detection Engine and score refresh; when the system
  collection is due the frontend invokes them concurrently.
- The database mutex serializes writes and no inconsistent score was observed,
  but the work is duplicated. Removing either call is not a safe mechanical fix:
  system/network baseline events could otherwise be delayed or missed while live
  polling is paused. A later orchestration design should consolidate analysis
  after both collection domains commit.

### INFORMATIONAL

- The production JS chunk is 502.36 kB minified / 145.22 kB gzip. Vite reports
  the default 500 kB advisory; no runtime regression was demonstrated.
- `cargo audit` reports 17 allowed warnings. GTK3/ATK/GDK and `glib` are absent
  from the Windows target graph. `unic-*` remains an unmaintained Tauri/urlpattern
  transitive on Windows; no applicable known vulnerability was reported.
- The Windows Computer Use screenshot path failed with
  `SetIsBorderRequired (0x80004002)`. Accessibility-tree QA remained available,
  but pixel screenshots and exact-resolution visual comparison were not claimed.

## Architecture review

Reviewed flow:

`Windows collectors → tracking → baseline → factual Security Events → Detection Engine/rules → Vulnerability Intelligence → Security Score v2`

- Rust owns Windows collection, SQLite, provider ingestion, matching, Detection
  evaluation, and score calculation. React remains presentation/orchestration.
- Tauri capability scope is `core:default`; no arbitrary shell, Registry, WMI,
  SQL, file-path, process-control, firewall, or service-control IPC exists.
- IPC models and inputs are typed. User values are parameterized; internal SQL
  identifiers are fixed or allowlisted.
- Primary and reconstructible databases remain independent. Cache/provider
  failure cannot poison baseline or Detection state after H-01.
- Provider generations, exact resume checkpoints, transactional page commits,
  durable outbox, analysis checkpoints, rule versions, and formula versions give
  explicit crash/restart semantics.
- No deadlock or impossible persisted state was reproduced. L-03 is the remaining
  orchestration inefficiency.

## Telemetry and tracking

The real release reported healthy collectors with factual partial coverage:

- System: 12 observations, about 86 ms.
- Processes: 245–248 observations, about 51 ms; 112–113 restricted metadata
  records remained coverage information, not `unsigned` or threat evidence.
- Network/connections: 385 observations, about 2 ms.
- Services: 279 observations, about 47–50 ms.
- Primary route: real Ethernet route, metric 25, gateway and DNS shown separately.

Native and unit validation confirms total-logical-capacity CPU normalization,
first-sample calculating state, PID creation-identity/reuse guards, network-byte
order, best-route/interface classification, last-good behavior after partial
connection/service failures, and no mass-close inference from unavailable data.

## Baseline

- Three baseline versions are preserved; the active baseline is v3/Ready.
- Learning, Ready, Stale, Error, restart recovery, reset/relearn, host identity,
  alternate Windows volume, and durable failure behavior passed Rust tests.
- Event audit found zero Security Events created inside the Learning window for
  baseline versions 1, 2, and 3.
- Historical event counts remain 274, 228, and 2,876 respectively after each
  baseline became Ready; history was not deleted during the audit.

## Detection Engine and false positives

- Registry remains exactly six enabled immutable v1 rules.
- Maximum severity remains Medium; no rule produces High/Critical.
- Tests cover positive, should-not-trigger, missing/partial metadata,
  `unknown`/`restricted`, loopback, ambiguous correlation, isolated service
  change, transient network change, precedence, deduplication, reopen,
  evidence, rule version, severity, and confidence.
- Live legitimate observations included 38 active Chrome rows, 7 Discord rows,
  2 Discord helper rows, and 6 Edge WebView2 rows, all with restricted access
  where applicable and no unsigned inference.
- Current database: zero active Detections and zero active High/Critical.
  The two historical controlled process-rule identities are inactive.
- No installer, updater, or VPN was launched because that would mutate the host.
  Those scenarios were reviewed through conservative rule gates and fixtures;
  the audit does not claim a new live production corpus.

## Vulnerability Intelligence

- Native compact audit passed all 31 current Confirmed CVEs.
- Current state remains 31 Confirmed/high-confidence, 1 Possible/low-confidence,
  and 1 KEV Confirmed; Possible impact remains zero.
- JRE, VirtualBox, and Python identities preserve publisher, original and
  normalized version, canonical vendor/product/CPE, resolution method,
  confidence, range, configuration applicability, engine version, and evidence.
- AND/OR/negate, numeric boundary comparison, exact-version failure modes,
  aliases/extractors, candidate ambiguity, NVD environment requirements, and
  unavailable-cache failure-closed behavior passed Rust tests.
- KEV remains post-match enrichment and cannot independently confirm a match.

## Security Score v2 and coverage

Validated formula:

`100 - min(100, detectionPenalty + vulnerabilityPenalty)`

- Formula version 2; product cap 7; Vulnerabilities cap 18.
- Product impacts: JRE 6, VirtualBox 4, Python 4.
- Detection penalty 0, vulnerability penalty 14, current score 86.
- Possible, Not affected, Unresolved, and not-mappable records have zero penalty;
  not-mappable remains outside the denominator.
- v1 and v2 snapshots remain readable and distinct (30 v1, 9 v2 at final DB
  audit).
- Real UI says “Observed posture under current coverage,” displays `/100`, and
  separates Good from Limited coverage; it does not call the score a security
  percentage.

## Database health and recovery

### `sentinel.db`

- Size: 353,210,368 bytes.
- `integrity_check=ok`; `quick_check=ok`; zero foreign-key violations.
- WAL enabled; migrations 1–10 applied.
- Current-match states: 31 Confirmed, 1 Possible, 1,720 Unresolved, 8,220 Not
  affected; evaluation queue empty.
- Relevant evaluation/match/evidence/history/score/outbox indexes are present.

### `vulnerability-cache.db`

- Size: 801,083,392 bytes.
- `integrity_check=ok`; `quick_check=ok`; zero foreign-key violations.
- WAL enabled; checksummed cache migration 1 applied.
- NVD 378,386 records; CISA KEV 1,666 records; zero pending outbox delivery.

Cache absence/failure is covered by the new startup regression. Corrupt-cache UI
was not induced against the user's real 801 MB repository.

## Performance

- Real release warm window creation: 138 ms in the controlled launcher sample.
- Fully populated accessible overview: 6.925 s in that sample, including native
  collection and security state hydration.
- 15-second active-polling sample: 1.203 CPU seconds = 8.02% of one logical core,
  approximately 0.67% of the 12-logical-core host; 46.7 MiB working set, 24.4
  MiB private memory, 27 threads, 710 handles.
- Baseline processing observed in the real UI: about 14–44 ms.
- Latest stored all-software matching run: 81 products, 2,278 ms total, 28.12 ms
  average, 701 ms maximum (Chrome).
- Score v2: risk query 6.320 ms, calculation 0.473 ms, calculation plus snapshot
  persistence 58.782 ms.
- Representative read-query medians: inventory 0.079 ms, latest score 0.004 ms,
  detections page 0.005 ms, latest evaluations 0.133 ms, historical confirmed
  match query 13.650 ms, NVD CVE lookup 0.007 ms, KEV lookup 0.004 ms, product
  identity lookup about 9.5–9.8 ms.

No runaway thread/handle growth or collector loop was observed. L-03 and M-01
remain the meaningful performance/storage debts.

## Security and privileges

- CSP blocks remote scripts, objects, frames, and foreign base URIs. Inline
  styles remain allowed for the existing theme/layout implementation.
- Provider clients use rustls, HTTPS-only mode, bounded bodies, 45-second
  timeout, three retries, bounded exponential backoff, strict payload/date/CVE
  validation, sanitized errors, and cooperative cancellation.
- No API key or secret exists in React or tracked configuration. Manual tracked
  signature-pattern scan found no secret. `gitleaks` and `trufflehog` were not
  installed, so their absence is an audit limitation.
- `pnpm audit --prod`: no known vulnerabilities.
- `cargo audit`: no blocking vulnerability; warnings documented above.
- Runtime launched and collected as the current user. Restricted metadata is
  surfaced rather than prompting for elevation.
- MSI metadata: Product 0.1.0, language 1033, `ALLUSERS=1` (per-machine). MSI
  installation may elevate; application runtime does not.

## Offline mode

- Provider synchronization is explicit, not a startup dependency.
- Real Settings UI states that successful repositories remain available offline;
  local NVD/KEV status was Ready.
- Cache-unavailable startup and matching-unavailable failure modes are covered by
  regression tests; telemetry, inventory, baseline, detections, and primary DB
  have no provider-network dependency.
- The audit did not disable the host network because changing OS connectivity
  would affect unrelated user applications. Therefore it does not claim a new
  end-to-end physical-disconnection run.

## I18n, accessibility, themes, and responsiveness

- English real-release tree was inspected across Overview, Inventory, command
  palette, and Settings. No literal i18n key or placeholder was observed.
- pt-BR/English key parity, interpolation parity, plurals, runtime language
  switching, document language, and localized rendering passed the automated
  frontend suite. Settings exposed both Português (Brasil) and English.
- A pt-BR runtime visual pass was not repeated because screenshot/click geometry
  failed; no claim is made beyond automated/runtime-control coverage.
- Keyboard: `Ctrl+K`, immediate search typing, Enter execution, query reset,
  dialog semantics, labels, and Escape dismissal passed in the real release.
  Dialog focus trap/restoration and reduced-motion CSS are present.
- The real theme menu exposed Sentinel Blue, Cyber Green, Terminal, and Spectrum;
  Cyber Green persistence was observed in SQLite. Pixel comparison of all four
  themes was not possible without screenshots.
- Responsive CSS provides compact table columns at 1,180 px and mobile layouts at
  820/540 px; Tauri min size remains 960×640. Exact 1920×1080, 1600×900,
  1440×900, 1366×768, and 1280×720 pixel captures were not produced because the
  Windows capture helper failed. No fabricated visual result is reported.

## Installation and artifacts

Final build outputs:

- `edy-sentinel.exe`: 16,340,992 bytes; SHA-256
  `7071CFD4D8CFFC7CE96C36A4F9F594547D6A78F0DAB9EB01F55B92A9962B0F21`.
- MSI: 6,057,984 bytes; SHA-256
  `E4A0FBD9F2114C43956C23CB208D8B1D2726AD01D4171A0F7C320FAF287EF06C`.
- NSIS: 4,350,628 bytes; SHA-256
  `799ED028E324F097E0DA618A9F475A524D816456AFA3AE5F99D4B2EE035F0C9D`.

EXE launch, repeated clean process close/reopen, primary DB open, migrations,
locale/theme persistence, MSI metadata, and package generation were validated.
A clean machine install/uninstall was not executed because it would mutate the
user's installed-product and application-data state; no uninstall claim is made.

## Gates

- `cargo fmt --check`: pass.
- `cargo check`: pass.
- `cargo clippy --all-targets -- -D warnings`: pass.
- Rust tests: 127 pass, 0 fail, 14 explicit manual/ignored.
- Native Windows collectors smoke: pass.
- Native 31-CVE audit: pass.
- Native Score v2 calibration/benchmark: pass.
- `pnpm lint`: pass, no warnings.
- `pnpm typecheck`: pass.
- Frontend/i18n tests: 65 pass across 19 files.
- `pnpm audit --prod`: no known vulnerabilities.
- `cargo audit`: no blocking advisory; 17 allowed warnings documented.
- React production build: pass, chunk-size advisory only.
- Tauri release: pass.
- EXE/MSI/NSIS generation: pass.
- DB integrity/FK checks: pass for both databases.
- Secret-pattern scan: pass with tool-availability limitation documented.
- `git diff --check`: pass.

## Final verdict

BLOCKED BEFORE RELEASE

Required closure: establish and verify Authenticode signing/timestamping for EXE,
MSI, and NSIS before Release v1.0. M-01 should receive an explicit retention
decision; L-03 can be addressed in a later bounded orchestration change.
