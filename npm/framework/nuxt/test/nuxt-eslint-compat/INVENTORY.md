# `@nuxt/eslint` compatibility inventory

Ground truth for the Nuxt lint config port. The corpus in `fixtures/corpus.json`
is run through the real `@nuxt/eslint` and `@nuxt/eslint-config` by `oracle.mjs`,
which records the result in `fixtures/nuxt-eslint-output.json`.

- `npm/framework/nuxt/src/lint/oracle.test.ts` reads the recording offline and
  holds Vize's implementation to it.
- `tests/tooling/nuxt-eslint-oracle.test.ts` re-derives the recording from the
  installed packages in CI, so an upstream bump fails loudly.
- Re-record with
  `node npm/framework/nuxt/test/nuxt-eslint-compat/oracle.mjs --write`.

Pinned upstream: `@nuxt/eslint@1.16.0`, `@nuxt/eslint-config@1.16.0` (catalog
`nuxt-eslint-oracle` in `pnpm-workspace.yaml`).

## Engine

Vize does not vendor ESLint. Patina runs through oxlint via
`oxlint-plugin-vize`, and its rule ids are already `eslint-plugin-vue`
compatible, so the port reproduces `@nuxt/eslint`'s _semantics and surface_ on
Vize's own engine. The plan this phase produces is therefore engine-neutral:
it names rules with their eslint-compatible ids, and the emitter that turns a
plan into an oxlint config (where Patina rules gain oxlint's `vize/` plugin
prefix) is a later phase.

## Config items

The blocks the port owns, in emission order. Order is observable — a later item
overrides an earlier one — so it is part of the contract.

| Config item            | Applies to                                                | Rule                                 | Patina status  |
| ---------------------- | --------------------------------------------------------- | ------------------------------------ | -------------- |
| `nuxt/ignores`         | whole project                                             | — (7 ignore globs)                   | supported      |
| `nuxt/setup`           | every linted file                                         | — (declares the `$fetch` global)     | supported      |
| `nuxt/vue/single-root` | layouts, pages, server components                         | `vue/no-multiple-template-root`      | supported      |
| `nuxt/rules`           | every linted file                                         | `nuxt/prefer-import-meta`            | not ported yet |
| `nuxt/pages`           | pages                                                     | `nuxt/no-page-meta-runtime-values`   | not ported yet |
| `nuxt/nuxt-config`     | `nuxt.config`                                             | `nuxt/no-nuxt-config-test-key`       | not ported yet |
| `nuxt/sort-config`     | `nuxt.config`                                             | `nuxt/nuxt-config-keys-order`        | not ported yet |
| `nuxt/disables/routes` | app/error, layouts, pages, nested and prefixed components | `vue/multi-word-component-names` off | supported      |

Items outside this table belong to `@nuxt/eslint-config`'s generic
JavaScript/TypeScript/Vue/import/stylistic/tooling presets. They are a separate
phase, so the recording captures their **names and order** (`configNames`) but
not their rule bodies — a preset appearing, disappearing, or moving still fails
the oracle.

## Intentional divergences

| Difference                                                           | Why                                                                                                                                                                                           |
| -------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| The ignore block is named `nuxt/ignores`; upstream leaves it unnamed | Every block needs a stable identity for the emitter to address and for users to override. Applied in exactly one place in `oracle.test.ts`.                                                   |
| `features` carries only feature keys                                 | Upstream serialises the whole module option object into `features`, so `configFile`, `autoInit`, `rootDir` and `devtools` leak into it. Vize keeps module options and feature flags separate. |

## Directory defaults

Defaults for a hand-written config, where directory lists may be absent.
A generated config always supplies every list, so these only apply to the
standalone entry point.

| Case                    | Behaviour                                                                            |
| ----------------------- | ------------------------------------------------------------------------------------ | --- | ------------------------------------------------------------------------------------ |
| `defaults/empty`        | No dirs at all: `root` becomes `[".", "./app"]` and everything else derives from it. |
| `defaults/src-only`     | Declaring `src` derives every per-feature directory from it.                         |
| `defaults/root-only`    | Declaring `root` makes `src` follow `root` rather than the built-in pair.            |
| `defaults/empty-arrays` | Present-but-empty lists stay empty: an empty array is truthy, so the `               |     | =`defaults never fire. This is why a generated config keeps`servers`and`root` empty. |
| `defaults/partial`      | Only missing lists are defaulted; declared ones pass through untouched.              |

Derived paths are built by interpolation (`` `${src}/pages` ``), not by joining,
so a `.` or `./app` source directory keeps its leading segment.

## Project states

How Nuxt project state reduces to the directory lists every glob is built from.

| Case                           | Behaviour                                                                                                                                    |
| ------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------- |
| `dirs/default`                 | Nuxt 4 defaults: a single layer with `srcDir` `app/`.                                                                                        |
| `dirs/srcdir-is-root`          | Nuxt 3 layout where `srcDir` equals `rootDir`, so `src` is `""` and every directory is top level.                                            |
| `dirs/two-layers`              | A base layer extended by the app layer; every list gains a second entry, in layer order.                                                     |
| `dirs/dir-overrides`           | `nuxt.options.dir` renames pages, layouts, plugins, middleware and modules. `composables`/`utils` are **not** configurable and stay literal. |
| `dirs/imports-dirs`            | Extra auto-import directories join `composables`; a `~/` prefix is stripped rather than alias-resolved.                                      |
| `dirs/components-string-array` | `components` as a bare array of directory strings.                                                                                           |
| `dirs/components-prefixed`     | `components` as objects; a `prefix` also adds the directory to `componentsPrefixed`, which exempts one-word names.                           |
| `dirs/components-true`         | `components: true` keeps the default `components/` directory, same as omitting it.                                                           |

## Feature flags

| Case                              | Behaviour                                                                                            |
| --------------------------------- | ---------------------------------------------------------------------------------------------------- |
| `features/defaults`               | `standalone` on, `stylistic`/`tooling`/`formatters` off, `typescript` from package detection.        |
| `features/standalone-false`       | Drops the baseline blocks; only the Nuxt-aware ones remain, and `nuxt/ignores` is dropped with them. |
| `features/stylistic`              | Enabling stylistic rules also switches `nuxt.config` key sorting on.                                 |
| `features/stylistic-sort-opt-out` | An explicit `nuxt.sortConfigKeys: false` overrides that default.                                     |
| `features/sort-config-opt-in`     | Key sorting can be enabled without stylistic rules.                                                  |
| `features/typescript-false`       | Drops the TypeScript blocks from the baseline.                                                       |
| `features/typescript-true`        | Adds them regardless of package detection.                                                           |
| `features/tooling`                | Adds the module-author rule blocks (jsdoc, unicorn, regexp).                                         |

`features.typescript` defaults to whether the `typescript` package resolves from
`@nuxt/eslint-config`. The recording stores that probe's answer as
`typeScriptDetected` so the offline test stays on the same branch as the
recording rather than guessing.
