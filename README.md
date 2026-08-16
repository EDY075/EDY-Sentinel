# EDY Sentinel

EDY Sentinel is a local-first Windows security observability desktop. Sprint 1 adds
live process, connection, and service observation to the production-shaped Tauri
foundation, with typed IPC, versioned SQLite persistence, a responsive premium shell,
and four persisted themes.

No security score, alert, or operational metric is simulated. A missing collector is reported as unavailable; future engines remain explicitly not implemented.

## Current capabilities

- Real hostname, user, Windows product/version/build, architecture, and uptime
- Real CPU, GPU, memory, and volume details
- Real Windows adapters, IPv4/IPv6 addresses, default gateway, and DNS resolvers
- Real Windows processes with CPU/memory, ownership, path, command line when
  accessible, thread count, architecture, executable metadata, and signature status
- Real TCP/UDP IPv4/IPv6 bindings and connections correlated to processes by PID
- Real read-only Windows service state, startup configuration, account, binary, and PID
- Live pause/resume, factual change events, first/last seen tracking, and collector health
- Virtualized process/connection/service tables with search, filters, sorting, keyboard
  navigation, and detail drawers
- Local SQLite snapshots, normalized observations, retention, and settings migrations
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

Telemetry never leaves the device in Sprint 1. The application has no API keys,
analytics, cloud integration, scanner, or artificial intelligence module. Process
command lines are shown only when Windows exposes them and are not persisted.

## Scope boundary

Threat intelligence services, geolocation/reputation, active response, advanced
network scanning, CVE matching, AI, and a computed Security Score are intentionally
outside Sprint 1. See [ROADMAP.md](ROADMAP.md).
