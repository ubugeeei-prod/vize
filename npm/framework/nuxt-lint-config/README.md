# `@vizejs/nuxt-lint-config`

The engine-neutral, shareable Nuxt lint preset core used by `@vizejs/nuxt`.
It ports the project-aware directory, feature, and rule-block decisions from
`@nuxt/eslint-config` without requiring Nuxt or ESLint at runtime.

```ts
import {
  buildNuxtLintPlan,
  resolveNuxtLintDirs,
  resolveNuxtLintFeatures,
} from "@vizejs/nuxt-lint-config";

const dirs = resolveNuxtLintDirs({ src: ["src"] });
const features = resolveNuxtLintFeatures({ stylistic: true }, () => true);
const plan = buildNuxtLintPlan(features, dirs);
```

The result is an ordered, engine-neutral plan using eslint-compatible rule
identifiers. `@vizejs/nuxt` consumes the same exports and emits them for
Vize's oxlint integration, so the module and standalone entry point cannot
drift into separate implementations.

Pass an optional third argument (`2`, `3`, or `4`) to `buildNuxtLintPlan` when
the Nuxt major is known. Nuxt 2 omits the `import.meta` migration and
`definePageMeta` rule blocks. Omitting the argument preserves the modern
Nuxt 3/4 plan; the Nuxt module supplies the detected runtime major.

Compatibility is pinned to the real `@nuxt/eslint` and
`@nuxt/eslint-config` packages by the committed differential oracle under
`test/nuxt-eslint-compat`.
