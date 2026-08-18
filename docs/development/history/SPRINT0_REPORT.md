# EDY Sentinel — Sprint 0 Final Report

Date: 2026-08-16

Version: 0.1.0

Scope: Foundation only

## Outcome

Sprint 0 is complete. EDY Sentinel is a working Tauri 2 Windows desktop application with real system/network telemetry, local SQLite persistence, a modular Rust core, a responsive React shell, four functional themes, security boundaries, documentation, tests, and x64 Windows installers.

No Security Score, alert, threat-intelligence result, vulnerability, device, or metric is fabricated. The Overview states that the analysis engine is not configured.

## Created

- Tauri 2 + React 19 + TypeScript + Rust workspace
- Premium three-zone shell: compact/expanded sidebar, topbar, and scroll-safe content
- Command palette with only available actions
- Toasts plus loading, empty, partial, and error states
- Reusable status, severity-ready badge, metric, dialog, drawer, tooltip, skeleton, and chart primitives
- Four token-based themes: Sentinel Blue, Cyber Green, Terminal, Spectrum
- Theme persistence through validated Rust IPC and SQLite
- Branded EDY Sentinel Windows application icon
- Versioned SQLite migration and seven requested domain tables
- Narrow Tauri capabilities and restrictive CSP
- Complete architecture, development, security, roadmap, changelog, and contributor documentation

## Final architecture

```text
React presentation
  -> src/lib/tauri.ts (typed IPC wrappers)
Tauri command boundary
  -> input validation and orchestration
Rust models and collectors
  -> Windows Registry / WMI / IP Helper / sysinfo
Rust persistence repository
  -> SQLite in Tauri app_data_dir
```

Frontend never receives SQL, Registry paths, WMI queries, filesystem paths, or shell execution capability. Rust is authoritative for collection and persistence.

## Directory structure

```text
EDY-Sentinel/
├── src/
│   ├── components/ui/
│   ├── features/overview/
│   ├── features/theme/
│   ├── lib/
│   └── types/
├── src-tauri/
│   ├── capabilities/
│   ├── icons/
│   ├── migrations/
│   └── src/
│       ├── collectors/
│       └── persistence/
├── docs/adr/
├── README.md
├── ARCHITECTURE.md
├── ROADMAP.md
├── SECURITY.md
├── DEVELOPMENT.md
├── CHANGELOG.md
└── AGENTS.md
```

## Dependencies added

Frontend/runtime: React, React DOM, `@tauri-apps/api`, Lucide React.

Frontend/build: TypeScript, Vite, React Vite plugin, Oxlint, `@tauri-apps/cli`.

Rust: Tauri, Serde/Serde JSON, Chrono, Rusqlite (bundled SQLite), Sysinfo, IPConfig, Winreg, and WMI.

No visualization framework, Three.js, analytics SDK, cloud SDK, or integration SDK was added.

## Commands

- `pnpm dev`: frontend-only development, with an honest desktop-only collection error
- `pnpm tauri dev`: complete Windows desktop development app
- `pnpm lint`: Oxlint
- `pnpm typecheck`: TypeScript check
- `pnpm test:rust`: Rust unit tests
- `pnpm check:rust`: Clippy with warnings denied
- `pnpm build`: frontend production build
- `pnpm verify`: lint, typecheck, Rust tests, and frontend build
- `pnpm tauri build`: x64 Windows executable, MSI, and NSIS installer

## Real data collected

- Hostname and current Windows user
- Windows product, edition, display version, and build
- Architecture and uptime
- CPU model, logical/physical cores, and reported frequency
- GPU model, adapter memory when reported, and driver version
- Total and used RAM
- Volumes, mount points, file systems, capacity, available bytes, and removable state
- Network interfaces and friendly names
- IPv4 and IPv6 addresses
- Primary gateway and DNS resolvers

On the validation host, the collector found 1 GPU, 2 volumes, 5 addressed interfaces, a gateway, and DNS. Hostname/user matched independent Windows values. Memory values were within a valid real range.

## SQLite

- Database location class: per-user Tauri application-data directory
- Schema version: 1
- Migration ledger: `schema_migrations`
- Tables: `system_snapshots`, `network_snapshots`, `devices`, `alerts`, `security_events`, `settings`, `integrations`
- Foreign keys, WAL, and busy timeout enabled
- Actual release-app run increased persisted snapshots from 3 to 4
- Theme round-trip and invalid severity constraints covered by unit tests

## Validation results

| Gate | Result |
|---|---|
| Environment / `tauri info` | PASS — WebView2 151, MSVC Build Tools 2022, Rust 1.97.1, Node 24.17, pnpm 11.19 |
| Frontend lint | PASS |
| TypeScript typecheck | PASS |
| Rust format | PASS |
| Rust Clippy (`-D warnings`) | PASS |
| Rust tests | PASS — 3 passed, 0 failed |
| Frontend production build | PASS |
| Dependency audit | PASS — no known pnpm vulnerabilities |
| Tauri release build | PASS |
| MSI bundle | PASS |
| NSIS bundle | PASS |
| Real release executable launch | PASS |
| Real SQLite snapshot persistence | PASS |
| Browser console errors/warnings | PASS — none |
| Theme switching | PASS — all four theme IDs applied |
| Responsive widths | PASS — no horizontal overflow at 1920×1080, 1440×900, 1366×768, 1280×720, or 960×640 |
| Secret/personal-path scan | PASS — no credential or personal path value found |
| Mock/encoding scan | PASS — no operational mock marker, U+FFFD, or mojibake found |

The release compiler prints one informational Portuguese MSVC line about creating the import library as Rust's `linker stdout` warning. It is not a code warning or build failure; Clippy remains clean with warnings denied.

## Build outputs

- Standalone x64 executable
- x64 MSI installer
- x64 NSIS setup executable

The build target was redirected to drive D because drive C had only about 60 MB free during the first native compilation. The incomplete Cargo target was preserved recoverably under `D:\CodexArchive\EDY-Sentinel\target-partial-20260816-0209`; no user file was deleted.

## Security review

- No administrator requirement
- No API keys or secrets
- No external HTTP service or analytics
- No arbitrary command, SQL, WMI, Registry, or path IPC
- Rust validates the four allowed theme identifiers
- CSP restricts content to the app, IPC, local assets, and inline theme styles
- `.env`, databases, logs, build output, temporary files, and local artifacts ignored by Git
- Integration schema stores only future `secret_ref`, never credential material

## Problems found and resolutions

1. Rust/MSVC/WebView2 prerequisites were missing. Official Rustup, Visual Studio 2022 Build Tools, Windows SDK, and WebView2 were installed and verified.
2. Initial Vite dev forwarding passed an extra `--`, so Tauri waited on the wrong port. The dev command was corrected to bind explicitly to `127.0.0.1:1420`.
3. Drive C ran out of space during the first Rust test. The ignored partial target was archived on D and all later Cargo output was redirected to D.
4. Windows capture automation could not inspect the Tauri WebView2 window because its border interface returned `0x80004002`. The real executable/process and database were verified directly; visual/theme/responsive validation used the same live React build in the local browser.

## Technical decisions

- Modular local-first monolith instead of premature services
- Registry/WMI/IP Helper/sysinfo adapters; no parsing of localized shell output
- Deterministic primary interface preference: non-loopback IPv4 with a gateway, then a non-loopback fallback
- Byte counts remain raw in Rust; formatting belongs to React
- Partial collector issues remain visible without destroying healthy sections
- SQLite is private to Rust and migration-driven
- Themes use semantic CSS tokens and persist in SQLite
- Future modules are documented seams, not empty fake implementations

## Risks and pending items

- Windows 11 and machines with VPN/virtual adapters need broader hardware-matrix testing.
- WMI can be slow or restricted; a future scheduler should add timeout/cancellation and collector health history.
- The Windows packages are not code-signed, so SmartScreen may warn on distribution.
- Theme switching was visually validated in the local browser and persistence was validated in Rust/SQLite; automated screenshot capture of the Tauri window remains limited by the Windows capture tool.
- Drive C remains critically low on free space; future native builds should keep `CARGO_TARGET_DIR` on D.

## Sprint 1 recommendation

Implement a conservative local snapshot scheduler and an explainable Baseline Engine with retention and change history. Emit evidence-backed local events before adding external threat intelligence, scanners, CVE matching, AI, or a composite Security Score.

Sprint 1 was not started.
