# vize_patina

`vize_patina` lints Vue Single File Components.

## Highlights

- Vue-focused lint rules covering correctness, style, accessibility, security, Vapor, Musea, and type-aware checks
- Built-in presets: `happy-path`, `opinionated`, `essential`, `incremental`, `nuxt`
- Human-readable and machine-readable reporting helpers
- Locale support through `vize_carton::i18n::Locale`

## Key Entry Points

- `Linter`
- `LintPreset`
- `LintResult`
- `format_results`
- `format_summary`
- `OutputFormat`

## Related Crates

- `vize` exposes Patina through `vize lint`
- `oxlint-plugin-vize` bridges Patina diagnostics into Oxlint
- `vize_maestro` reuses Patina for editor diagnostics

## License

MIT
