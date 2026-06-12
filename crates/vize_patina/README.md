# vize_patina

`vize_patina` lints Vue Single File Components.

## Highlights

- Vue-focused lint rules covering correctness, style, accessibility, security, Vapor, Musea, and type-aware checks
- Built-in presets: `happy-path`, `opinionated`, `essential`, `incremental`, `ecosystem`, `nuxt`
- Human-readable and machine-readable reporting helpers
- Locale support through `vize_carton::i18n::Locale`

## Key Entry Points

- `Linter`
- `LintPreset`
- `LintResult`
- `format_results`
- `format_summary`
- `OutputFormat`

## Rule IR (template + JSX)

The `markup` module is a zero-copy, rule-facing IR that lets one rule body run
over **both** Vue templates and JSX/TSX without materializing a synthetic AST.
Wrappers borrow the live parser nodes (`vize_relief` for templates, OXC for
JSX/TSX) and the original source; names and values are `&str` slices, so nothing
allocates unless a rule asks for normalized owned data.

Core types:

- `MarkupDocument` — document over a template root or a JSX/TSX program, with an
  optional `vize_croquis::Croquis` for semantic / type-aware rules.
- `MarkupElement`, `MarkupAttribute`, `MarkupText` — element / attribute / text
  facades; `MarkupDirective` covers Vue `v-*` directives **and** directive-like
  JSX attributes.
- `MarkupBinding` (+ `MarkupBindingKind`) — the normalized binding view (plain
  attribute, `v-bind`, event, `v-model`, custom directive) with event/model
  **modifiers**, so a rule reasons about bindings the same way on either backend.
- `MarkupConditional` / `MarkupList` — `v-if` / `v-for` scopes.

Author a rule by implementing `MarkupRule` (default-empty `enter_*` hooks) and
drive it with `MarkupDocument::visit_with`, which projects the hooks from either
backend through a `MarkupContext` wrapping the usual `LintContext`. All
diagnostics and fixes report through `ByteRange`s that map to the original
syntax. `a11y/img-alt`, `vue/require-v-for-key`, `vapor/prefer-static-class`,
and `vapor/no-vue-lifecycle-events` ship a `MarkupRule` entry point alongside the
legacy `Rule` impl; full per-rule migration is tracked in follow-up work.

## Related Crates

- `vize` exposes Patina through `vize lint`
- `oxlint-plugin-vize` bridges Patina diagnostics into Oxlint
- `vize_maestro` reuses Patina for editor diagnostics

## License

MIT
