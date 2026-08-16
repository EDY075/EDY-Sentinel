# Development

## Toolchain check

```powershell
node --version
pnpm --version
rustc --version
cargo --version
git --version
pnpm tauri info
```

Tauri on Windows requires WebView2 and Visual Studio Build Tools with MSVC and a Windows SDK.

## Commands

| Command | Purpose |
|---|---|
| `pnpm dev` | Run the Vite frontend only; real telemetry shows an explicit desktop-only error |
| `pnpm tauri dev` | Run the complete desktop app |
| `pnpm lint` | Run Oxlint |
| `pnpm typecheck` | Check TypeScript project references |
| `pnpm test:rust` | Run Rust unit tests |
| `pnpm check:rust` | Run Clippy with warnings denied |
| `pnpm build` | Build the frontend |
| `pnpm verify` | Lint, typecheck, Rust tests, frontend build |
| `pnpm tauri build` | Build Windows bundles |

## Adding a collector

1. Define a DTO with explicit optional fields in Rust.
2. Implement a small Windows adapter without shelling out or parsing localized command output.
3. Preserve partial failure as an issue rather than inventing values.
4. Add a narrow command only when aggregation cannot use an existing one.
5. Mirror the serialized contract in TypeScript and add unit tests.
6. Document source, privilege requirements, and data semantics.

## Adding a migration

Add the next ordered SQL file and register it in `MIGRATIONS`. Migrations are append-only, transactional, and must never silently discard existing data.

## Final hygiene checks

```powershell
rg -n --hidden --glob '!node_modules/**' --glob '!src-tauri/target/**' '(api[_-]?key|secret|token|password)\s*[:=]'
rg -n --hidden --glob '!node_modules/**' --glob '!src-tauri/target/**' 'C:\\Users\\|/home/'
git status --short --ignored
git diff --check
```

Review every match. Documentation terms and `.env.example` comments may be intentional; real values and personal paths are not.
