---
title: Rules
---

# Rules

[All lint rules](./all.md) lists every rule with its purpose, scope, configuration, and Bad/Good
examples on the same page. Each pair explains why the Bad example triggers a finding and how the
Good example changes it. Use the index to jump to a rule without opening another page.

With Vite+, use `defineConfig` from `@vizejs/vite-plugin/vite-plus`, configure `vue.linter`,
and run `vp run lint` for Vize and Oxlint. Existing script-name collisions use `vp run vize:lint`.
Built-in `vp lint` uses its own upstream checker.

## Pages

- [All lint rules](./all.md): searchable catalogue with every purpose, scope, configuration,
  and full Bad/Good example pair inline.
- [Rule Options](./options.md): typed `vue.linter.ruleOptions` shapes, defaults, replacement
  behavior, and examples for every configurable rule.
- [ESLint migration map](./migration.md): mapped names, differences, unsupported rules, and
  literal Vite+ configuration changes.
- [Vue rules](./vue.md): SFC template structure, Vue directives, component conventions, and
  single-file Vue correctness checks.
- [Type and script rules](./type-and-script.md): TypeScript checker-backed diagnostics and Vapor
  script restrictions.
- [HTML rules](./html.md): HTML validity and semantic markup checks.
- [Accessibility rules](./accessibility.md): ARIA, keyboard interaction, labels, landmarks, and
  accessible media checks.
- [SSR rules](./ssr.md): server rendering and hydration hazards.
- [Vapor rules](./vapor.md): Vapor-only template constraints.
- [Ecosystem rules](./ecosystem.md): preset-backed checks for Nuxt, Vue Router, Pinia, vue-i18n,
  Vue Test Utils, and Void Vue.
- [Musea and CSS rules](./musea-and-css.md): Musea art-block checks and style diagnostics.
- [Cross-file rules](./cross-file.md): project-graph diagnostics emitted by
  `vp run lint` with `crossFile: true`, including complete typed Router projects.

## Presets

`essential` contains correctness rules that should almost always be enabled. `happy-path` adds
practical hygiene checks for day-to-day Vue development. `ecosystem` starts from the broad default
bundle and adds Vue Router, Vue I18n, Pinia, Vue Test Utils, Nuxt, and Void Vue checks. `nuxt`
includes Nuxt-oriented SSR expectations and Vapor expectations. `opinionated` is the broadest
built-in preset.

`incremental` starts empty. Use it when a host wants to opt into specific rules without inheriting a
larger preset.

### On by default

| Preset | Rules | What is on |
| --- | ---: | --- |
| `essential` | 81 | Template and directive validity, Vue 3 removals (`vue/no-deprecated-*`, `script/no-deprecated-*`), script macro, `nextTick` and Options API correctness, unused components, `vue/no-v-html`, `vue/no-unsafe-url`, `html/id-duplication` |
| `happy-path` (default) | 141 | `essential`, plus template formatting conventions, accessibility, HTML conformance, SSR hazards, unused properties, and typed props and emits |
| `ecosystem` | 149 | `happy-path`, plus the ecosystem library checks |
| `nuxt` | 197 | `opinionated`, adjusted for Nuxt auto-imports |
| `opinionated` | 198 | `happy-path`, plus stricter conventions, typed props and the `css/*` style rules |
| `incremental` | 0 | Nothing |

Counts exclude cross-file rules. The Presets column of [All lint rules](./all.md) shows each rule's
presets. The Vue 3 removal checks are off when `compiler.compatibility.vueVersion` is `2` or `2.7`.

`vue/no-use-v-else-with-v-for` warns about accepted Vue syntax whose conditional and loop share an element; put the conditional on a wrapper for clarity. `script/no-deep-destructure-in-props` is a depth convention enabled by `opinionated` and `nuxt`. It does not read the compiler’s `propsDestructure` setting: nested bindings can compile when reactive destructuring is disabled or `withDefaults` is used. Both rules remain warnings and can be disabled by name. `essential` and the empty `incremental` bundle are unchanged.

## Type-Aware Configuration

Rules that need semantic information read the TypeScript project through `tsconfig.json`. Prefer
putting shared environment names in `compilerOptions.types` or project references instead of keeping
a separate `globals` list in Vize config.
