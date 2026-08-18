# Sprint 3 Part 3B — Security Score v2 UI, QA & Finalization

## Outcome

Security Score v2 is implemented and validated in the real Tauri release. The score consumes the
persisted Rust DTO; the frontend contains no host-specific score or product constants. Matching
Engine v1, the product-identity resolver, Detection Engine, and the six rules were not modified.

Validated current state:

| Metric | Result |
| --- | ---: |
| Base | 100 |
| Detection contribution | 0 |
| Vulnerability contribution | -14 |
| Final Security Score | 86 |
| Confirmed CVEs | 31 |
| Possible CVEs | 1, impact 0 |
| Confirmed KEV | 1 |
| JRE / VirtualBox / Python impact | 6 / 4 / 4 |
| Formula version | 2 |

## Experience delivered

- The Overview card presents the numeric score, separate Detection/Vulnerability components,
  attention count, baseline state, and vulnerability coverage state.
- The breakdown shows the exact `100 - D - V` math, canonical products, confirmed/possible counts,
  KEV, product impact, software identity coverage, module coverage, timestamp, and formula version.
- Product details list installed versions, confirmed and possible CVE IDs, highest CVSS, evidence
  engine versions, and the action into the existing Inventory/CVE evidence flow.
- Possible CVEs are neutral and explicitly show score impact zero.
- KEV wording states that CISA enrichment prioritizes an already-confirmed match and does not
  confirm the software-to-CVE relationship.
- pt-BR and English switch in runtime with localized dates, numbers, pluralization, status, and
  accessible drawer/card names.

## Coverage and lifecycle

Software identity coverage is separate from vulnerability-match totals:

| Software coverage | Count |
| --- | ---: |
| Total software records | 81 |
| Eligible | 66 |
| Resolved identities | 19 |
| Ambiguous | 0 |
| Unresolved eligible | 47 |
| Not mappable | 15 |

The real release initially reported 81 pending evaluations. That state kept 86 visible but clearly
qualified the global score as limited. After `Evaluate queued`, pending became 0 and the score state
returned to Good at 86. Identity coverage correctly remained Limited because 47 eligible records
are unresolved. No missing identity or Possible match was converted into a penalty.

## Visual QA

Validated in the release Tauri executable, not an isolated browser frontend:

- Português (Brasil) and English;
- Sentinel Blue, Cyber Green, Terminal, and Spectrum;
- 1920×1080 Windows display work area and the native 1440×900 app window;
- score card, breakdown drawer, product detail, KEV context, long CVE lists, scroll, and narrow CSS
  breakpoints covered by automated rendering/build checks.

The Windows controller could not produce WebView2 screenshots directly, so evidence was captured
from the active Tauri window with Alt+Print and saved from the Windows clipboard. Every delivered
PNG was opened and visually inspected. Evidence lives outside Git at:

`%USERPROFILE%\Documents\EDY-Sentinel-QA\Sprint3B`

The eight numbered files are the delivery set; additional underscore-prefixed files are QA working
evidence for themes, lifecycle, settings, Inventory, and responsive work-area checks.

## Automated validation

- Rust formula, persistence, lifecycle, history, and migration tests;
- React rendering, presentation, accessibility, and i18n parity tests;
- lint, typecheck, frontend build, cargo fmt/check/clippy/tests;
- Tauri release, EXE/MSI/NSIS outputs;
- SQLite integrity and foreign-key checks;
- dependency audits and `git diff --check`.

## Auditability

All 31 Confirmed matches remain traceable to current software identity, CPE/version applicability,
minimal NVD evidence, engine/resolver versions, source versions, and evaluation timestamp. Formula
v1 snapshots remain immutable; formula v2 snapshots carry their own complete input fingerprint and
breakdown. No historical snapshot is reinterpreted.

## Verdict

**SPRINT 3 COMPLETE — READY FOR FINAL PRODUCT HARDENING**
