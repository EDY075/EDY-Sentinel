# Sprint 2B Report — Explainable Detection Engine and Security Score v1

## Delivery state

- Starting commit: `8cb799a232b6f35ac9a00505f288a5179bd89f59`
- Branch: `main`
- Delivery commit subject: `feat: add explainable detection engine and security score v1`
- Starting SQLite schema: v5; delivered schema: v6
- Migration history remains append-only: `0001` through `0006`; no earlier migration changed
- No push was performed and Sprint 3 was not started

Sprint 2B adds a real local Detection Engine and the first evidence-derived Security Score. The
three domains remain explicit:

`Windows observations → factual Security Events → rule-produced Detections → Score v1`

React presents results and sends validated workflow actions. All classification, severity,
confidence, correlation, provenance, deduplication, and score policy remain in Rust.

## Architecture

`DetectionEngine` is an independent durable consumer of `security_event_history`. Migration v6
seeds `analysis_checkpoints` at the current maximum history ID, preventing retroactive
classification of pre-v6 facts. Each refresh processes at most 250 rows per batch and eight
batches; it advances the checkpoint in the same analysis transaction as Detection writes.
Analysis transactions are independent from baseline transactions, so a rule/query failure cannot
put the behavioral baseline into Error.

The engine evaluates enabled immutable `RuleDefinition` versions, joins evidence inside bounded
correlation windows, applies exclusions and precedence, and stores:

- one stable Detection identity with exact `rule_id` and `rule_version`;
- append-only evidence references to source event/history rows;
- append-only workflow/lifecycle history;
- structured explanations, severity/confidence rationales, and safe remediation guidance.

The `ScoreEngine` consumes active Detections only after a Ready baseline and complete measured
collector coverage. It persists immutable, versioned snapshots only when inputs change or the
15-minute heartbeat is due.

## SQLite schema v6

Migration `src-tauri/migrations/0006_detection_engine.sql` adds seven domain tables:

- `detection_rule_versions`: immutable serialized rule definitions and SHA-256 definition hash
- `detection_rule_state`: local enabled/version state
- `detections`: current aggregate identity, rule, entity, explanation, workflow, and lifecycle
- `detection_evidence`: immutable source event/history provenance and evidence snapshot
- `detection_history`: append-only created/reactivated/inactive/status transitions
- `security_score_snapshots`: immutable formula/coverage/breakdown history
- `analysis_checkpoints`: cutover and last-consumed factual history cursor

Indexes serve actual keyset/status/severity/rule/entity/time and provenance queries. No isolated
low-selectivity boolean index was added. Baseline and factual-source references use restrictive
history-preserving foreign keys. Evidence/history/rule/score triggers reject mutation or
provenance mismatches. Baseline event retention now excludes source events referenced by Detection
evidence. Score snapshots retain 365 days; Detection/evidence history is preserved until a future
explicit archival/downsampling design exists.

Final real database audit after native smoke:

- `integrity_check=ok`
- `foreign_key_check=[]`
- WAL enabled
- migrations 1–6 present
- 3 behavioral baselines; active v3 is Ready with no Error
- 6 immutable rule versions and 6 enabled local states
- 1 durable analysis checkpoint at the current processed history
- 2 controlled Detection identities, 18 immutable evidence rows, 22 lifecycle/history rows
- 15 Security Score snapshots preserving real 100, 97, and 92 states

## Rule registry

Exactly six enabled v1 rules are shipped. All are capped at Medium; High/Critical are unused.

| Rule | Purpose | Severity | Confidence | Window |
| --- | --- | --- | --- | ---: |
| EDY-PROC-001 v1 | Executed unsigned temporary executable | Low | High | 60 s |
| EDY-PROC-002 v1 | New parent/child plus unsigned temporary child | Medium | High | 60 s |
| EDY-NET-001 v1 | First-seen outbound activity from that executable | Medium | Medium | 60 s |
| EDY-SVC-001 v1 | Privileged automatic service from user-writable path | Medium | High | 30 s |
| EDY-SVC-002 v1 | Correlated service binary plus startup/account change | Medium | High | 45 s |
| EDY-NET-002 v1 | Persistent coordinated gateway and DNS change | Low | High | 30 s |

Common fail-closed gates include Ready baseline, complete required facts, exact unsigned state,
unambiguous joins, valid temporal window, and rule-specific exclusions. `unknown`/`restricted`
never means unsigned. Dynamic destination, port, novelty, CompanyName, restricted metadata, or
missing process association cannot independently create elevated severity.

Precedence is `NET-001 > PROC-002 > PROC-001`, with one executable correlation group for scoring.
`EDY-PROC-003` was deliberately deferred: path/size/mtime/signer metadata is not stable enough to
infer tampering without bounded content identity. Full rule rationale, exclusions, false-positive
context, and remediation are in [`../../technical/detection-rules.md`](../../technical/detection-rules.md).

## Detection and score policy

- Persistent activity updates one Detection; occurrence count increments only on reactivation.
- Resolved activity reopens under the same stable identity when it returns.
- Ignored history is preserved. Disabled rules stop contributing and active matches are closed.
- `New`, `Investigating`, and `Acknowledged` contribute only while the condition is active.
- `Resolved`, `Ignored`, and inactive conditions do not contribute to the current score.
- Confidence means evidence completeness/correlation quality, never malware probability.

Score formula v1 starts at 100. Base penalties are Informational 0, Low 3, Medium 8, High 18,
Critical 35; confidence factors are Low 0.50, Medium 0.75, High 1.00. Only the largest penalty per
correlation key is counted, and the total penalty is capped at 100. A failed/incomplete required
collector or non-Ready baseline produces no numeric score; degraded coverage is marked Limited.
The complete contract is in [`../../technical/security-score.md`](../../technical/security-score.md).

## Real Rust interfaces

Primary modules:

- `src-tauri/src/detection.rs`: `DetectionEngine::{initialize, process_pending,
  set_detection_status, set_rule_enabled, rule_definitions}`
- `src-tauri/src/detection_query.rs`: `DetectionCursor`, `DetectionQueryInput`,
  `DetectionPage`, `DetectionEvidenceCursor`, `DetectionEvidenceInput`,
  `DetectionEvidencePage`, parameterized keyset queries
- `src-tauri/src/rules.rs`: `RuleDefinition`, `RuleCondition`, `EvidenceRequirement`,
  `RulePolicy`, `ConditionValue`, `RuleOperator`, `registry`, `lookup`
- `src-tauri/src/score.rs`: `ScoreEngine::{calculate_and_persist, current}`
- `src-tauri/src/models.rs`: severity/confidence/status, Detection/explanation/evidence,
  rule-enabled input, coverage/breakdown/score DTOs
- `src-tauri/src/persistence/mod.rs`: migration ledger plus independent analysis read/transaction

New Tauri commands:

- `get_detections_page`
- `get_detection_evidence`
- `set_detection_status`
- `get_detection_rules`
- `set_detection_rule_enabled`
- `get_security_score`

Existing factual commands remain separate: `get_security_events_page`,
`get_security_event_history`, and `set_security_event_status`.

## Real TypeScript interfaces and UI

- `src/types/detection.ts`: Detection severity/confidence/status, exact cursor query/page,
  evidence cursor/query/page, and structured explanation types
- `src/types/rules.ts`: mirror of the real versioned Rust `RuleDefinition`
- `src/types/score.ts`: available/limited/unavailable score, coverage, and breakdown
- `src/lib/tauri.ts`: typed wrappers for all commands above; no SQL or classification in React
- `src/features/security`: Detections/Events workspace, bounded page stack, debounce, notification
  seed, and cursor footer
- `src/features/detections`: paginated list and wide structured provenance drawer
- `src/features/rules`: six-rule metadata table and local typed enabled switch; no rule editor
- `src/features/score`: Overview score and complete explanation drawer
- `src/features/telemetry/TelemetryProvider.tsx`: remains the single polling owner; no second loop

The sidebar retains one Events entry. The security workspace separates Detections from factual
Events. Rules is a subordinate view, not a new sidebar item. Server keyset order is canonical;
the UI never sorts/searches one page and implies a global result.

## Detection Quality

- Rules: 6 enabled, all explicit and versioned
- Rule-focused Rust tests: positive, negative, missing/partial evidence across all six rules;
  global tests cover disabled/version/precedence/dedup/reopen/status/provenance/pagination
- Safe native true condition: an unsigned temporary helper that only sleeps for 45 seconds
- Controlled result: PROC-001 and the higher-specificity PROC-002 identity, with PROC-001
  suppressed/inactivated by precedence
- Legitimate negatives: signed/unknown executables, loopback and ordinary destinations,
  isolated service changes, incomplete service metadata, incomplete/transient network changes
- Real-host noise check: 1,371 factual events existed at the final audit point; only the two
  controlled helper identities existed as Detections, with no unrelated normal-use flood
- Disabled rules: none; weak PROC-003 was not registered rather than shipping disabled ambiguity

The short development baseline and existing factual ledger validate infrastructure only. They are
not a production corpus, and no detection-rate or malware-accuracy claim is made.

## Native Windows smoke

The optimized Tauri EXE was opened against the real user database and migrated v5→v6. The active
baseline remained Ready and system/process/connection/service collectors were healthy. The benign
helper produced factual executable/process/parent-child evidence; PROC-002 created a Detection,
the drawer displayed exact source evidence, repeat runs reused the same ID and reached occurrence
5, status Acknowledge/Resolve persisted, and source Events remained navigable.

The real score changed from 100 to 92 for the active Medium/High-confidence Detection and returned
to 100 after resolution/inactivity. A Low/High-confidence intermediate state at 97 is preserved in
SQLite. No malware, network connection, service change, persistence, privilege change, or response
action was used.

## Performance

Sprint 2A reference was approximately 0.385% native-host total CPU capacity and 3–13 ms baseline
work. Sprint 2B native release measurements on 12 logical processors were:

- native host CPU: 0.612% over 20 s and 0.582% over 30 s of total processor capacity;
- native host working set: 42.68 MiB steady, 53.27 MiB observed peak;
- controlled Detection lifecycle latency (`observed_at` → append-only recorded transition),
  including bounded correlation and SQLite persistence: n=12, p50 30.363 ms, p95 69.259 ms,
  maximum 82.469 ms;
- Security Score calculation: 15 real snapshots, all below the persisted 1 ms resolution;
- observed collector ranges during smoke: processes roughly 50–76 ms, connections 2–3 ms,
  services about 50 ms, and system overview roughly 104–125 ms;
- UI: no horizontal overflow in Detections, Events, or Rules at 1920×1080, 1600×900,
  1440×900, 1366×768, or 1280×720; all four themes passed the same DOM/layout check.

Database size changed from the pre-Sprint v5 backup 17,022,976 bytes (647 events/670 history rows)
to 29,958,144 bytes after extended native testing (1,371 events and 4,858
factual-history rows). The 12.34 MiB growth includes hours of real telemetry/provenance and five
controlled reactivation cycles, not just empty schema-v6 overhead. Analysis remained delta-based;
the checkpoint reached the current factual-history tail.

The current instrumentation separates score calculation from end-to-end Detection lifecycle
latency, but does not persist micro-timings for rule matching, correlation SQL, and Detection write
as three independent counters. That finer observability is a documented follow-up; no p50/p95 is
fabricated for boundaries the product does not measure.

## Quality gates and audits

- `pnpm lint`: pass
- `pnpm typecheck`: pass
- Vitest: 10 files / 27 tests pass
- React production build: pass (1,830 modules; JS 297.01 kB / 88.18 kB gzip;
  CSS 45.94 kB / 8.84 kB gzip)
- `cargo fmt --all -- --check`: pass
- `cargo check`: pass
- `cargo clippy --all-targets -- -D warnings`: pass
- Rust: 64 pass, 0 fail, 1 manual native smoke ignored
- `pnpm audit`: no known vulnerabilities
- `cargo audit`: no vulnerabilities; 17 allowed transitive warnings documented in `SECURITY.md`
- Tauri optimized release: EXE, MSI, and NSIS produced
- real SQLite: integrity OK and zero FK violations
- Git diff/encoding/secret/artifact scans: required before the delivery commit

The linker emits a localized informational library-generation message under the current MSVC
toolchain. It is not a compiler warning from project code. Cargo audit warnings are distinguished
as unmaintained transitives or non-Windows runtime graph debt; none was hidden.

## Screenshots

Nine real 1920×1080 Tauri/WebView captures were generated outside Git because they contain local
endpoint metadata:

1. Overview with real Security Score
2. controlled Detections list
3. Detection detail
4. structured evidence section
5. real 92-point score breakdown
6. six-rule registry
7. factual Security Events
8. Detections in Cyber Green
9. Rules in Spectrum

The controlled helper context is explicitly documented here; no fixture row was inserted into the
release database or frontend bundle.

## Limitations and risks

- Six local rules are conservative development coverage, not production calibration.
- No v1 rule justifies High/Critical; no severity was forced for visual variety.
- PROC-003/content identity, signer-aware service rules, command-line/LOLBin analysis, external
  intelligence, CVE matching, geolocation, native Windows notifications, and response actions are
  deferred.
- Dynamic endpoints, incomplete protected-process/service metadata, and CompanyName remain weak
  facts; they never imply threat or signer identity.
- Detection history has no automatic retention yet. Future archival must keep evidence required
  by active and historical explanations.
- Score is bounded to current coverage and rules. A value of 100 is not a security guarantee.
- The real database and screenshots contain sensitive local metadata and must remain outside Git.
- Unmaintained transitive dependency warnings remain upgrade debt and require periodic review.

## Final disposition

Sprint 2B is complete when the final commit records this report and the repository returns clean.
No Sprint 3 work is included.
