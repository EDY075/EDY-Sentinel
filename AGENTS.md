# EDY Sentinel contributor rules

1. Never display a fabricated security value, score, alert, device, or network fact.
2. Keep Windows collection and SQLite access in Rust; React is a presentation client.
3. Expose narrow typed Tauri commands. Never add arbitrary shell, SQL, Registry, WMI, file, or path access.
4. Treat partial collection as a first-class state. Preserve optionality and record the source.
5. Run as a standard user unless a future feature has a documented, reviewed reason to elevate.
6. Evolve SQLite with append-only, transactional, numbered migrations.
7. Store integration secrets in Windows Credential Manager; never in React, SQLite payloads, logs, `.env`, or Git.
8. Use semantic CSS tokens in all four themes. Do not add hardcoded component colors.
9. Maintain keyboard focus, reduced-motion support, responsive layouts, and Windows scaling behavior.
10. Run lint, typecheck, Rust tests, Clippy, frontend build, Tauri build, secret scan, and visual validation before release.
11. Do not begin Sprint 1 modules while Sprint 0 validation has unresolved failures.
