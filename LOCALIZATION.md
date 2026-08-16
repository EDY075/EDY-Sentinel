# EDY Sentinel Localization

EDY Sentinel supports two official interface languages:

- `en` — English (official fallback)
- `pt-BR` — Português do Brasil

Localization is a presentation concern. Persisted values, Rust enums, rule IDs, evidence, and detection or score logic remain language-neutral.

Stable entity types, evidence field names, baseline-context field names, and collector sources are localized only for display; their underlying values remain unchanged.

## Runtime architecture

The frontend uses `i18next` with `react-i18next`. Startup calls `initializeI18n()` before React is rendered, preventing an initial flash in the wrong language.

Language selection follows this order:

1. the preference saved in the existing SQLite `settings` table;
2. the Windows user locale read through `GetUserDefaultLocaleName` in the desktop app;
3. `en`.

Browser development mode uses `navigator.languages` / `navigator.language` in place of the native Windows command.

Any `pt-*` locale resolves to `pt-BR`. Any `en-*` locale resolves to `en`. Other locales use the `en` fallback.

The Settings view changes the language immediately and persists it through the narrow Tauri commands `get_language` and `set_language`. If persistence fails, the visible language is rolled back and a localized error is shown. Browser development mode uses local storage instead of Tauri.

## Source structure

Core files live in `src/i18n/`:

- `index.ts` — initialization, runtime switching, active-language access;
- `language.ts` — supported-language type, normalization, and initial detection;
- `format.ts` — locale-aware `Intl` helpers;
- `resources.ts` — typed resource registry and namespace list;
- `locales/en/` and `locales/pt-BR/` — domain catalogs.

Catalogs are separated into these namespaces:

`common`, `navigation`, `settings`, `errors`, `shell`, `overview`, `processes`, `connections`, `services`, `baseline`, `telemetry`, `events`, `detections`, `rules`, `score`, and `security`.

Components use `useTranslation(namespace)`. Presentation helpers receive an i18next `t` function when they must localize stable domain values such as severity, confidence, status, or event type.

## Adding or changing translations

1. Add the same key and interpolation variables to both locale files.
2. Keep the key semantic; do not use the English sentence as the key.
3. If a new domain is needed, add one catalog per locale and register the namespace in `resources.ts`.
4. Use stable Rust values only as lookup keys. Never send a translated enum to Tauri or persist it.
5. Run the resource-parity tests, lint, typecheck, frontend tests, and build.

English is not an unreviewed source dump: its wording, capitalization, and terminology are maintained as an official locale alongside pt-BR.

## Pluralization

Use i18next plural suffixes and pass the numeric `count`:

```ts
processes_one: '{{count}} process observed'
processes_other: '{{count}} processes observed'
```

When the number needs locale formatting, also pass `formattedCount`, but retain the raw numeric `count` so i18next selects the correct plural rule. Do not concatenate a manual `s` or choose plural text with an inline conditional.

## Dates and numbers

Use the helpers exported from `src/i18n` or `Intl.NumberFormat`, `Intl.DateTimeFormat`, and `Intl.RelativeTimeFormat` with the active i18next language. Do not hard-code `en-US` or `pt-BR` in a feature component.

Timestamps remain stored and transported in their canonical format. Only their rendered presentation is localized.

## Terminology glossary

| English | Português (Brasil) |
|---|---|
| Security Score | Pontuação de Segurança |
| Security Events | Eventos de Segurança |
| Detections | Detecções |
| Evidence | Evidências |
| Severity | Severidade |
| Confidence | Confiança |
| Processes | Processos |
| Active Connections | Conexões Ativas |
| Windows Services | Serviços do Windows |
| Primary Route | Rota Principal |
| Digital Signature | Assinatura Digital |
| Signer | Signatário |
| Company | Empresa |
| Coverage | Cobertura |
| Recommended Action | Ação Recomendada |
| Behavioral baseline | Linha de base comportamental |

`Baseline` may remain visible as a concise technical term where the surrounding pt-BR text makes its meaning clear.

## Content that must not be translated

Preserve exact technical values, including:

- `EDY Sentinel`;
- rule IDs and versions, such as `EDY-PROC-001`;
- hashes, IP addresses, ports, PIDs, paths, and executable names;
- real process and service names;
- API names;
- real company and signer values;
- entity keys, evidence values, correlation keys, and database IDs;
- strong-confirmation phrases required by the stable backend contract.

Detection and rule explanatory copy is localized by stable Rule ID. Unknown future rules retain their backend text until their catalog entries are added; they never change evaluation semantics.

## Error and fallback policy

User-facing errors are localized and do not interpolate raw Rust error strings into pt-BR screens. Raw technical values remain available at the backend boundary for diagnostics without becoming interface logic.

If a translation key is missing in `pt-BR`, i18next falls back to `en`. Resource-parity tests are intended to catch that condition before release.

## Verification

Tests cover locale normalization and startup priority, English fallback and missing keys, runtime switching, Tauri/browser persistence, catalog parity, plural rules, date and number formatting, localized severity labels, score states, all six rule IDs, and security presentation copy.

The 16 namespaces contain 969 leaf keys per locale with exact key and interpolation parity.

Sprint 2C visual verification used the real Tauri WebView in both languages at 1920×1080,
1600×900, 1440×900, 1366×768, and 1280×720. Sentinel Blue, Cyber Green, Terminal, and
Spectrum were all exercised without page-level horizontal overflow. Eight evidence screenshots
were generated outside the repository and are intentionally not tracked by Git.

## Known contract limitation

Security Score breakdown entries currently do not expose `ruleId`. The v1 UI therefore localizes the six known breakdown titles through a closed mapping of stable Rust titles and safely preserves unknown future titles. Adding `ruleId` to that DTO would remove this presentation-only fallback without changing the score formula.
