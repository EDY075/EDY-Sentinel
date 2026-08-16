# EDY Sentinel — Post-Sprint 0 stabilization review

Date: 2026-08-16  
Scope: Sprint 0 foundation only. Sprint 1 features and external integrations remain out of scope.

## Baseline and migration

- Baseline branch: `main`
- Baseline commit: `da372aa93d31131d04e61b3f5047bfacc1c812f2`
- Git history, refs, configuration, source, migrations, assets, documentation and dependency declarations were preserved.
- The source project was retained unchanged.
- The copied pnpm links were not portable across drives; the copied `node_modules` directory was archived outside the repository and dependencies were restored from the frozen lockfile.
- The migrated repository and the source baseline both passed `git fsck --full`.

## Corrections applied

### Telemetry and persistence

- System collection no longer fails merely because local snapshot persistence fails. A persistence failure is returned as a factual collection issue while the valid Windows telemetry remains visible.
- The frontend no longer combines telemetry and database status in a single `Promise.all`. A database-status failure cannot overwrite a healthy system snapshot with a false collection error.
- Host, CPU, GPU, memory, storage and network completeness checks now generate partial-collection issues instead of allowing an incomplete snapshot to claim that every collector is ready.
- Product language now describes local observation accurately: `Local collectors active` replaces protection language that implied a prevention engine.
- The Security Score remains intentionally empty and now states `Security analysis engine not initialized`; no synthetic score was introduced.

### Visual foundation and accessibility

- Tooltips now render, are associated with their controls and remain available in compact navigation.
- Theme selection exposes menu/radio semantics, focuses the selected option and closes on an outside pointer action.
- Dialogs now trap focus, support Escape and restore focus to the invoking control.
- Small, auditable transitions were added to controls, cards, menus, dialogs and toasts; the existing reduced-motion override remains authoritative.
- Primary-action and subtle-text contrast was corrected in all four themes.

## Theme review

| Theme | Subtle text / panel | Primary text / accent | Result |
|---|---:|---:|---|
| Sentinel Blue | 6.05:1 | 5.94:1 | Pass |
| Cyber Green | 7.26:1 | 8.34:1 | Pass |
| Terminal | 7.20:1 | 9.71:1 | Pass |
| Spectrum | 7.35:1 | 8.44:1 | Pass |

All themes preserve the same hierarchy, spacing and component structure. Terminal intentionally applies the monospaced interface font without introducing truncation in the required viewport matrix.

## Runtime verification

The desktop runtime was executed from the migrated repository with real Windows collection and local SQLite persistence. The latest snapshot matched the operating system in 15 checks:

- hostname, signed-in user and architecture;
- Windows build;
- CPU model and logical-core availability;
- GPU identity;
- total/used memory validity;
- local volumes;
- uptime;
- primary IPv4, gateway, DNS and interface inventory.

System and network snapshot counts advanced together after refresh and again after launching the release executable. Theme selection round-tripped through SQLite and was restored to Sentinel Blue after QA.

## Responsive matrix

| Viewport | Sidebar | Horizontal overflow | Expected vertical scroll |
|---|---:|---:|---:|
| 1920×1080 | 238 px expanded | 0 px | No |
| 1600×900 | 238 px expanded | 0 px | Yes |
| 1440×900 | 238 px expanded | 0 px | Yes |
| 1366×768 | 238 px expanded | 0 px | Yes |
| 1280×720 | 238 px expanded | 0 px | Yes |
| 1440×900 | 68 px compact | 0 px | Yes |

Manual resize was represented by applying these viewport changes sequentially to the live Tauri WebView2 target. No visible element crossed the viewport bounds. Compact mode exposed nine accessible tooltips.

## Quality gates

| Gate | Result |
|---|---|
| `pnpm install --frozen-lockfile` | Pass |
| `pnpm lint` | Pass |
| `pnpm typecheck` | Pass |
| Frontend tests | N/A — no frontend test script or suite exists in Sprint 0 |
| `cargo fmt -- --check` | Pass |
| Rust Clippy with `-D warnings` | Pass |
| Rust tests | Pass — 3 passed, 0 failed |
| React production build | Pass |
| Tauri environment inspection | Pass |
| Tauri release build | Pass — executable, MSI and NSIS bundles |
| Release executable smoke test | Pass — real snapshot persisted |
| pnpm production dependency audit | Pass — no known vulnerabilities |
| Git integrity and whitespace checks | Pass |

The optional RustSec `cargo audit` subcommand is not installed in this environment, so no RustSec CLI result is claimed. Clippy, tests, lockfile review and the successful release build completed; adding RustSec automation remains a recommended future CI hardening item rather than Sprint 1 product scope.

## Security and repository hygiene

- No credential-like values were found in tracked files or Git history using targeted token/private-key/secret patterns.
- No personal filesystem paths were found in tracked files or Git history.
- No database, environment, log, certificate or private-key files are tracked.
- The 49 tracked raster files are application icon assets only.
- Screenshots remain outside Git because they contain real endpoint telemetry.
- Build outputs remain outside the repository or under ignored directories.

## Prioritized findings

Resolved before closure:

1. contrast failures across all themes;
2. missing tooltips and compact-navigation identification;
3. false telemetry failure when persistence/status failed;
4. incomplete snapshots overstating collector health;
5. inaccurate protection language;
6. missing focus management in the command dialog and theme menu.

Known follow-up, not a blocker for closing Sprint 0:

1. no automated frontend component/E2E suite exists;
2. the CSS breakpoint below the Tauri minimum width still does not provide a full mobile drawer; this is outside the supported desktop window range;
3. RustSec dependency auditing should be automated in CI when that workflow is introduced.

## Decision

Sprint 0 is stable after these corrections and verification gates. This review intentionally stops here; Sprint 1 has not been started.
