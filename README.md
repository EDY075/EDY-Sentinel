# EDY Sentinel

EDY Sentinel is a local-first Windows security observability desktop. Sprint 0 establishes a production-shaped foundation: real Windows telemetry, typed Tauri IPC, versioned SQLite persistence, a responsive premium shell, and four persisted themes.

No security score, alert, or operational metric is simulated. A missing collector is reported as unavailable; future engines remain explicitly not implemented.

## Current capabilities

- Real hostname, user, Windows product/version/build, architecture, and uptime
- Real CPU, GPU, memory, and volume details
- Real Windows adapters, IPv4/IPv6 addresses, default gateway, and DNS resolvers
- Local SQLite snapshots and settings with schema migrations
- Sentinel Blue, Cyber Green, Terminal, and Spectrum themes
- Compact/expanded navigation, command palette, toast, loading, empty, partial, and error states
- Least-privilege Tauri application with a restrictive CSP and no external service dependency

## Requirements

- Windows 10/11 x64
- WebView2 Runtime
- Node.js 20+ and pnpm 10+
- Rust stable MSVC toolchain
- Visual Studio 2022 Build Tools with Desktop development with C++ and Windows SDK

## Run

```powershell
pnpm install
pnpm tauri dev
```

## Verify and build

```powershell
pnpm verify
pnpm check:rust
pnpm tauri build
```

The database is stored under the operating system application-data directory as `sentinel.db`; it is not created in the repository. See [DEVELOPMENT.md](DEVELOPMENT.md) for the complete workflow and [ARCHITECTURE.md](ARCHITECTURE.md) for module boundaries.

## Privacy

Telemetry never leaves the device in Sprint 0. The application has no API keys, analytics, cloud integration, scanner, or artificial intelligence module.

## Scope boundary

Threat intelligence services, advanced network scanning, CVE matching, AI, and a computed Security Score are intentionally outside Sprint 0. See [ROADMAP.md](ROADMAP.md).
