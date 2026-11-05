---
title: Rules
description: Turn on Vize's lint rules with a preset, then adjust single rules.
---

# Rules

Vize's linter checks Vue single-file components: templates, scripts, styles, and the links
between files. Pick a preset, then turn single rules up, down, or off.

## Turn on linting

With Vite+, import `defineConfig` from `@vizejs/vite-plugin/vite-plus`, configure `vue.linter`,
and run `vp run lint`. That command runs Vize's rules and Oxlint together.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      preset: "happy-path",
      rules: {
        "vue/no-v-html": "off",
        "a11y/alt-text": "error",
      },
    },
  },
});
```

Each rule takes `off`, `warn`, or `error`. Rules with settings read them from
`vue.linter.ruleOptions`; see [Rule Options](./options.md).

If your project already has a script named `lint`, run `vp run vize:lint` instead. The built-in
`vp lint` command is Vite+'s own checker and does not run Vize's rules.

## Presets

A preset is a starting set of rules. `happy-path` is the default.

| Preset | Rules | What it turns on |
| --- | ---: | --- |
| `essential` | 85 | Template and directive validity, Vue 3 removals (`vue/no-deprecated-*`, `script/no-deprecated-*`), script macro, `nextTick` and Options API correctness, lifecycle hooks, watchers and `expose` registered after `await`, imports from `vue` instead of `@vue/*`, unused components, `vue/no-v-html`, `vue/no-unsafe-url`, `html/id-duplication` |
| `happy-path` (default) | 147 | `essential`, plus template formatting conventions (including `v-bind` shorthand and hyphenated component events), accessibility, HTML conformance, SSR hazards, unused properties, declared emits, and typed props and emits |
| `ecosystem` | 155 | `happy-path`, plus the ecosystem library checks |
| `nuxt` | 213 | `opinionated`, adjusted for Nuxt auto-imports |
| `opinionated` | 213 | `happy-path`, plus stricter conventions, typed props, type-based `defineProps`/`defineEmits`, `<script setup>` macro order, camelCase event names, `useTemplateRef`, `defineSlots`, prop default and emit hygiene, `defineOptions`, and the `css/*` style rules |
| `incremental` | 0 | Nothing. Start here to turn on only the rules you name. |

Counts exclude cross-file rules. When `compiler.compatibility.vueVersion` is `2` or `2.7`, the
Vue 3 removal checks are off.

## Find a rule

| Page | What it covers |
| --- | --- |
| [All rules](./all.md) | Every rule in one index, grouped by category, with the smallest preset that turns it on |
| [Vue](./vue.md) | SFC templates, directives, components, blocks, and Vue 3 migration |
| [Type and script](./type-and-script.md) | `<script setup>`, compiler macros, reactivity, and type-aware checks |
| [HTML](./html.md) | HTML validity and semantic markup |
| [Accessibility](./accessibility.md) | ARIA, keyboard interaction, labels, landmarks, and media alternatives |
| [SSR](./ssr.md) | Server rendering and hydration hazards |
| [Vapor](./vapor.md) | Constraints for components compiled in Vapor mode |
| [Ecosystem and Nuxt](./ecosystem.md) | Nuxt, Vue Router, Pinia, Vue I18n, Vue Test Utils, and Void Vue |
| [Musea and CSS](./musea-and-css.md) | Musea `.art.vue` stories and plain CSS |
| [petite-vue](./petite-vue.md) | HTML pages that use petite-vue |
| [Cross-file](./cross-file.md) | Checks across components, providers, and routers, turned on with `crossFile: true` |
| [Rule Options](./options.md) | Typed `ruleOptions` and their defaults |
| [Migrating from ESLint](./migration.md) | `eslint-plugin-vue` rule names mapped to Vize, with differences |

Every rule page starts with what the rule checks, its default severity and presets. Then it shows a
Bad example that the rule reports and a Good example that it accepts, each with a one-line reason,
and the configuration that turns the rule on. The examples are the same files the linter's test
suite runs.

## Type-aware rules

Rules under `type/` read type information from the TypeScript project in `tsconfig.json`. They
need `typeAware: true` in `vue.linter` as well as a preset or a `rules` entry that turns them on. Declare shared environment names in
`compilerOptions.types` or with project references, not in a separate `globals` list in the Vize
configuration.
