# EDY Sentinel v1 release checklist

Local final version: `1.0.0`

Public release: blocked until Authenticode requirements pass.

## Source and version

- [x] Sprint 4B committed independently.
- [x] npm, Cargo, Tauri, Settings, MSI, and NSIS versions agree on `1.0.0`.
- [x] Product name, publisher identity, descriptions, copyright, executable, architecture, and icons
  inspected.
- [x] Worktree contains no database, screenshot, log, dump, installer, executable, certificate,
  private key, token, secret, or personal temporary path.

## Functional and data regression

- [ ] Clean installer passed, but first runtime with an empty isolated Windows profile still requires
  a disposable user/VM; changing process `APPDATA` does not redirect the Tauri Known Folder.
- [x] Restart persistence validated on the real release without resetting application data.
- [x] Upgrade from 0.1.0 preserved the install location and byte-identical main/cache databases.
- [x] Silent uninstall removed installer-owned files/registration and preserved application data.
- [x] Local telemetry, Inventory, baseline, Detections, score, and cached vulnerability evidence
  have no provider-network dependency; unavailable-cache regressions pass.
- [ ] A completely network-isolated provider attempt remains a disposable-user/VM acceptance gate.
- [x] Missing and unavailable cache fixtures degrade without blocking startup.
- [x] 31 Confirmed, 1 Possible, 1 KEV Confirmed, score 86, formula 2, Possible impact 0.
- [x] Six enabled v1 rules; trigger/no-trigger, deduplication, reopen, evidence, severity, and
  confidence regressions pass.

## Desktop acceptance

- [x] English and Português (Brasil) on the real Tauri release lineage.
- [x] Sentinel Blue, Cyber Green, Terminal, and Spectrum on the real Tauri release lineage.
- [x] Configured 1440x900 release-lineage evidence.
- [ ] Physical 1920x1080, 1600x900, 1366x768, and 1280x720 final captures.
- [x] Overview, Processes, Network, Inventory, Detections, Security Score, and Settings reviewed.
- [ ] System/Services, Activity/Security Events, and individual CVE Detail remain in the final
  release-environment visual matrix.
- [x] Keyboard navigation, command-palette focus, dialogs/drawers, ARIA contracts, tooltips,
  reduced-motion CSS, and theme contrast reviewed; UIA geometry limitation is recorded.
- [x] Eleven real screenshots are stored outside Git.
- [ ] Individual CVE Detail capture remains unavailable through the current WebView2 geometry
  boundary.

## Technical gates

- [x] lint, typecheck, frontend/i18n tests, React production build.
- [x] cargo fmt, check all targets, clippy with warnings denied, Rust/regression suites.
- [x] npm audit and cargo audit reviewed.
- [x] Tauri EXE, MSI, and NSIS generated locally as `UNSIGNED BUILD`.
- [x] SQLite integrity and foreign-key checks pass.
- [x] Performance compared with Sprint 4A/4B evidence.
- [x] `git diff --check` and final secret/artifact scan pass.

## Public signing gate

- [ ] Legitimate publisher certificate/private key injected from protected release infrastructure.
- [ ] SHA-256 Authenticode signature on application EXE, MSI, and NSIS.
- [ ] Trusted timestamp on all three artifacts.
- [ ] Post-signature verification passes and immutable public hashes are recorded.
- [ ] Public release approval, tag, push, and GitHub Release performed by an authorized release
  operator only after every gate above is complete.
