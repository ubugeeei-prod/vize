---
title: Rules
---

# Rules

Vize diagnostics are documented as rules, not as one large matrix. Each rule page keeps the
detection behavior close to the Bad/Good examples so the reference can be read like an ESLint rule
manual.

## Pages

- [Vue rules](./vue.md): SFC template structure, Vue directives, component conventions, and
  single-file Vue correctness checks.
- [Accessibility rules](./accessibility.md): ARIA, keyboard interaction, labels, landmarks, and
  accessible media checks.
- [Type and script rules](./type-and-script.md): TypeScript checker-backed diagnostics and Vapor
  script restrictions.
- [HTML, SSR, and Vapor rules](./html-ssr-vapor.md): HTML validity, server rendering hazards, and
  Vapor-only template constraints.
- [Cross-file analyzer rules](./cross-file.md): project-graph diagnostics emitted by
  `vize lint --cross-file`.
- [Musea and CSS rules](./musea-and-css.md): Musea art-block checks and style diagnostics.

## Presets

`essential` contains correctness rules that should almost always be enabled. `happy-path` adds
practical hygiene checks for day-to-day Vue development. `nuxt` includes Nuxt-oriented SSR and Vapor
expectations. `opinionated` is the broadest built-in preset.

`incremental` starts empty. Use it when a host wants to opt into specific rules without inheriting a
larger preset.

## Type-Aware Configuration

Rules that need semantic information read the TypeScript project through `tsconfig.json`. Prefer
putting shared environment names in `compilerOptions.types` or project references instead of keeping
a separate `globals` list in Vize config.
