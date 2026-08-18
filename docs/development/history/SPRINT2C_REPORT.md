# Sprint 2C Report — English and Português (Brasil)

Date: 2026-08-16
Base commit: `f55872cb76666c4cd74d7ad09fa8af7803140a75`
Scope: localization and release validation only; Sprint 3 was not started.

## Outcome

Sprint 2C delivers complete English and Português (Brasil) presentation coverage for the
EDY Sentinel desktop interface. Language changes take effect without restarting, persist in
the existing SQLite settings store, and survive a real close/reopen cycle. When no preference
exists, the native Windows user locale is consulted before the official English fallback.

The implementation contains 16 namespaces and 969 leaf keys in each locale, with exact key
and interpolation parity. Plural selection and dates, times, relative time, numbers, percentages,
memory, storage, and durations use locale-aware formatting.

## Preserved security contracts

- Detection Engine source, six v1 rule definitions/evaluators, Security Score formula, baseline
  engine, collectors, factual-event queries, detection queries, and SQLite migrations are unchanged.
- Rule IDs and versions, entity keys, hashes, paths, PIDs, process/service names, IP addresses,
  ports, collector values, evidence values, and database IDs remain invariant.
- Localization maps known stable values only for display and uses the raw value as the safe
  fallback for a future unknown value.
- No threat claim, severity escalation, response action, external service, or new telemetry was added.

## Runtime and persistence

Startup priority is saved preference → native Windows user locale → English. Portuguese locale
variants normalize to `pt-BR`; English variants normalize to `en`; unsupported locales fall back
to English. `document.documentElement.lang` is updated at runtime. A failed preference write rolls
the interface back and presents a localized error.

Real Tauri close/reopen validation passed in both directions: English persisted after restart,
then Português (Brasil) persisted after a second restart. The final local preference was restored
to Português (Brasil), with Sentinel Blue as the final theme.

## Visual verification

The real Tauri WebView was inspected in both languages at:

- 1920×1080
- 1600×900
- 1440×900
- 1366×768
- 1280×720

Overview, Detections, Detection Detail, and Settings remained contained without page-level
horizontal overflow. The detection drawer remained fully inside every viewport. Sentinel Blue,
Cyber Green, Terminal, and Spectrum were exercised; each applied its distinct palette without
layout regression.

Eight real 1440×900 screenshots were captured outside Git under the Codex task output folder:

1. Overview — pt-BR / Spectrum
2. Detections — pt-BR / Spectrum
3. Detection Detail — pt-BR / Spectrum
4. Settings — pt-BR / Spectrum
5. Overview — English / Spectrum
6. Detections — English / Spectrum
7. Settings — English / Spectrum
8. Overview — English / Terminal

## Verification gates

- `pnpm lint`
- `pnpm typecheck`
- `pnpm test` — 16 files / 58 tests
- `pnpm build`
- `cargo fmt --check`
- `cargo check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test` — 66 passed / 1 manual smoke ignored
- `pnpm audit`
- `cargo audit` — no vulnerabilities; 17 allowed transitive warnings (unmaintained crates
  plus one GTK/glib unsoundness advisory not selected by the Windows target)
- SQLite `integrity_check` = `ok`; `foreign_key_check` = 0 rows; migrations 1–6 present
- Tauri release build with EXE, MSI, and NSIS artifacts

## Release artifacts

The release executable was opened and inspected at 1440×900. It loaded in pt-BR with the
persisted Sentinel Blue theme, rendered the live Security Score, and had no horizontal overflow.

| Artifact | Size | SHA-256 |
|---|---:|---|
| `src-tauri/target/release/edy-sentinel.exe` | 12,084,736 bytes | `4ADD11D9510EEBD30FD5ED457403349A30AA80D11C359AEC612C03B0E0777C94` |
| `src-tauri/target/release/bundle/msi/EDY Sentinel_0.1.0_x64_en-US.msi` | 4,390,912 bytes | `4FDCEBF07A9D288A2B43629BD888E92373D82594A2177D96931082A6D444A1FF` |
| `src-tauri/target/release/bundle/nsis/EDY Sentinel_0.1.0_x64-setup.exe` | 3,101,650 bytes | `42D94D4DE90AD2692821616FBFC2CC4B1AE63F961D49CBFB71AA84EE06F35A4B` |

## Deferred by design

Production rule calibration, external reputation or threat intelligence, geolocation, CVE
matching, advanced network scanning, native Windows notifications, AI/ML classification,
automated response, firewall/service control, continuous executable content hashing, and Sprint 3
remain outside this delivery.
