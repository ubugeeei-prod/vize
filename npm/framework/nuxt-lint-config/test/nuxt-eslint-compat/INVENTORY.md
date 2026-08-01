# `@nuxt/eslint` compatibility inventory

Ground truth for the Nuxt lint config port. The corpus in `fixtures/corpus.json`
is run through the real `@nuxt/eslint` and `@nuxt/eslint-config` by `oracle.mjs`,
which records the result in `fixtures/nuxt-eslint-output.json`. Rule cases run
through the real `@nuxt/eslint-plugin` rule and record diagnostics, output, and
a second pass proving fix convergence or non-fixable stability.

- `npm/framework/nuxt-lint-config/src/oracle.test.ts` reads the plan recording offline and
  holds Vize's implementation to it.
- `tests/tooling/nuxt-eslint-oracle.test.ts` re-derives the recording from the
  installed packages in CI, so an upstream bump fails loudly.
- Re-record with
  `node npm/framework/nuxt-lint-config/test/nuxt-eslint-compat/oracle.mjs --write`.

Each project case records both the upstream Nuxt-aware config items and the
complete oxlint artifact Vize emits from them. The offline suite compares that
artifact as one byte string, including item order, rule-id mapping, whitespace,
and the trailing newline.

Pinned upstream: `@nuxt/eslint@1.16.0`, `@nuxt/eslint-config@1.16.0`, and
`@nuxt/eslint-plugin@1.16.0` (catalog `nuxt-eslint-oracle` in
`pnpm-workspace.yaml`; the plugin is the module's exact transitive dependency).

## Engine

Vize does not vendor ESLint. Patina runs through oxlint via
`oxlint-plugin-vize`, and its rule ids are already `eslint-plugin-vue`
compatible, so the port reproduces `@nuxt/eslint`'s _semantics and surface_ on
Vize's own engine. The plan this phase produces is therefore engine-neutral:
it names rules with their eslint-compatible ids. The emitter turns that plan
into an oxlint config, adding the `vize/` plugin prefix only at the engine
boundary.

## Config items

The blocks the port owns, in emission order. Order is observable — a later item
overrides an earlier one — so it is part of the contract.

| Config item            | Applies to                                                | Rule                                 | Patina status |
| ---------------------- | --------------------------------------------------------- | ------------------------------------ | ------------- |
| `nuxt/ignores`         | whole project                                             | — (7 ignore globs)                   | supported     |
| `nuxt/setup`           | every linted file                                         | — (declares the `$fetch` global)     | supported     |
| `nuxt/vue/single-root` | layouts, pages, server components                         | `vue/no-multiple-template-root`      | supported     |
| `nuxt/rules`           | every linted file                                         | `nuxt/prefer-import-meta`            | supported     |
| `nuxt/pages`           | pages                                                     | `nuxt/no-page-meta-runtime-values`   | supported     |
| `nuxt/nuxt-config`     | `nuxt.config`                                             | `nuxt/no-nuxt-config-test-key`       | supported     |
| `nuxt/sort-config`     | `nuxt.config`                                             | `nuxt/nuxt-config-keys-order`        | supported     |
| `nuxt/disables/routes` | app/error, layouts, pages, nested and prefixed components | `vue/multi-word-component-names` off | supported     |
| `nuxt/import-globals`  | whole project                                             | — (Nuxt/Nitro auto-import globals)   | supported     |

Items outside this table belong to `@nuxt/eslint-config`'s generic
JavaScript/TypeScript/Vue/import/stylistic/tooling presets. They are a separate
phase, so the recording captures their **names and order** (`configNames`) but
not their rule bodies — a preset appearing, disappearing, or moving still fails
the oracle.

## `nuxt/prefer-import-meta` rule cases

| Case                                                 | Behaviour                                                                          |
| ---------------------------------------------------- | ---------------------------------------------------------------------------------- |
| `prefer-import-meta/static-suffixes`                 | All seven upstream suffixes diagnose and preserve their suffix in `import.meta.*`. |
| `prefer-import-meta/computed-identifier`             | `process[client]` matches the upstream property-node predicate.                    |
| `prefer-import-meta/optional-chain`                  | The complete optional member expression is replaced.                               |
| `prefer-import-meta/nested-member`                   | Only the inner `process.*` member is replaced in a longer chain.                   |
| `prefer-import-meta/shadowed-process`                | The syntax-only upstream rule reports a shadowed `process` parameter.              |
| `prefer-import-meta/assignment-target`               | Assignment targets are diagnosed and fixed.                                        |
| `prefer-import-meta/computed-and-object-near-misses` | String keys, unknown properties, and non-root `process` members stay valid.        |
| `prefer-import-meta/lexical-near-misses`             | Strings, comments, other identifier spellings, and object keys stay valid.         |
| `prefer-import-meta/multiple-lines`                  | Multiple findings retain exact non-overlapping ranges and fix together.            |

## `nuxt/no-page-meta-runtime-values` rule cases

| Case                                                       | Behaviour                                                                                             |
| ---------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| `no-page-meta-runtime-values/eager-context-apis`           | All 30 direct context APIs pinned by upstream report at the eager macro level.                        |
| `no-page-meta-runtime-values/composable-name-boundaries`   | Only direct identifiers matching `/^use[A-Z]/` report beyond the fixed API list.                      |
| `no-page-meta-runtime-values/this-and-await`               | Eager `this` and complete `await` expressions report; a context call below `await` also reports.      |
| `no-page-meta-runtime-values/callbacks-are-lazy`           | Arrow, function-expression, and object-method bodies admit runtime values, `this`, and `await`.       |
| `no-page-meta-runtime-values/direct-callee-only`           | Member, sequence, and `new` callees stay valid; optional and nested direct calls retain diagnostics.  |
| `no-page-meta-runtime-values/shadowed-identifiers`         | Shadowed API and composable identifiers still report because the upstream rule is syntax-only.        |
| `no-page-meta-runtime-values/macro-boundaries`             | Direct sequential macros form boundaries; outside values and member/misspelled macro callees do not.  |
| `no-page-meta-runtime-values/eager-nested-structures`      | Computed keys, arrays, spreads, nested objects, and nested macro arguments stay eager.                |
| `no-page-meta-runtime-values/function-parameters-are-lazy` | Function default parameters are lazy together with their bodies; an adjacent eager API still reports. |
| `no-page-meta-runtime-values/optional-macro-call`          | Optional direct macro calls form a boundary; empty and identifier-only arguments stay valid.          |

## `nuxt/no-nuxt-config-test-key` rule cases

| Case                                                  | Behaviour                                                                                   |
| ----------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| `no-nuxt-config-test-key/direct-object-booleans`      | Boolean `true` and `false` values on direct identifier `test` keys both report.             |
| `no-nuxt-config-test-key/call-first-argument`         | Any default-exported call inspects only its first argument when that argument is an object. |
| `no-nuxt-config-test-key/member-call`                 | A member-expression callee qualifies because the rule does not constrain the callee.        |
| `no-nuxt-config-test-key/computed-identifier-key`     | A computed `[test]` key reports because upstream does not inspect the computed flag.        |
| `no-nuxt-config-test-key/property-key-near-misses`    | Quoted, computed-literal, differently cased, and longer property keys stay valid.           |
| `no-nuxt-config-test-key/value-near-misses`           | Identifiers, numbers, strings, null, calls, shorthands, and methods stay valid.             |
| `no-nuxt-config-test-key/export-shape-near-misses`    | Identifier-backed exports and non-exported wrapper calls are not traversed.                 |
| `no-nuxt-config-test-key/nested-and-later-objects`    | Nested objects and object arguments after the first one stay valid.                         |
| `no-nuxt-config-test-key/parenthesized-object`        | Parentheses preserve the direct default-exported object behaviour.                          |
| `no-nuxt-config-test-key/parenthesized-call-argument` | Parentheses around the exported call and its first object argument are transparent.         |
| `no-nuxt-config-test-key/parenthesized-key-and-value` | Parentheses around computed identifier keys and boolean values preserve matching.           |
| `no-nuxt-config-test-key/escaped-identifier-key`      | Unicode escapes preserve the semantic `test` identifier and bypass no prefilter.            |
| `no-nuxt-config-test-key/multiline-property-range`    | The diagnostic spans the complete property and remains non-fixable across a second pass.    |

## `nuxt/nuxt-config-keys-order` rule cases

| Case                                                 | Behaviour                                                                                                     |
| ---------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `nuxt-config-keys-order/complete-reversed-order`     | The entire ~60-key table, `$` group, and both official-module groups are pinned as one complete fixed output. |
| `nuxt-config-keys-order/already-ordered`             | Ordered properties produce no diagnostic and remain byte-identical.                                           |
| `nuxt-config-keys-order/comments-and-trailing-comma` | Comments, indentation, newline ownership, and trailing-comma insertion match upstream exactly.                |
| `nuxt-config-keys-order/environment-objects`         | Top-level and `$environment` objects report together and converge across repeated fix passes.                 |
| `nuxt-config-keys-order/unknown-and-literal-keys`    | Known keys lead; unknown identifiers and raw literal keys follow upstream locale collation.                   |
| `nuxt-config-keys-order/computed-and-method-keys`    | Computed identifiers, methods, shorthands, and computed literals retain upstream name extraction.             |
| `nuxt-config-keys-order/spread-boundaries`           | Spreads stay fixed and bound independently sorted property segments.                                          |
| `nuxt-config-keys-order/call-and-parentheses`        | A parenthesized first object argument of any default-exported call is inspected.                              |
| `nuxt-config-keys-order/export-shape-near-misses`    | Identifier exports, local calls, and later object arguments are not traversed.                                |
| `nuxt-config-keys-order/short-object-near-misses`    | Empty and single-property objects remain valid and byte-identical.                                            |
| `nuxt-config-keys-order/case-collation`              | Unknown ASCII keys pin localeCompare's case-insensitive, lowercase-first order.                               |

## Dev-server checker

| Case                   | Behaviour                                                                                                                       |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `checker/default`      | An empty object resolves every portable upstream default, including `fix: false`, and always runs outside the main thread.      |
| `checker/true`         | Boolean opt-in resolves identically to the empty object.                                                                        |
| `checker/all-explicit` | `cache`, globs, formatter, startup, warning/error emission and fixing all pass through exactly; worker execution stays enabled. |

## Intentional divergences

| Difference                                                                              | Why                                                                                                                                                                                                                                                                                    |
| --------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| The ignore block is named `nuxt/ignores`; upstream leaves it unnamed                    | Every block needs a stable identity for the emitter to address and for users to override. Applied in exactly one place in `oracle.test.ts`.                                                                                                                                            |
| `features` carries only feature keys                                                    | Upstream serialises the whole module option object into `features`, so `configFile`, `autoInit`, `rootDir` and `devtools` leak into it. Vize keeps module options and feature flags separate.                                                                                          |
| Auto-init creates an `oxlint.config.mts` loader                                         | Oxlint 1.64 does not inherit `globals` from a JSON config in `extends`. The loader returns the current generated object directly and rebases its JS plugin URL, preserving `$fetch` and addon globals.                                                                                 |
| The addon hook is `vize:lint:config:addons` and contributes engine-neutral config items | Upstream's `eslint:config:addons` contributes raw JavaScript and import statements. Those are ESLint implementation details and cannot be represented safely in an oxlint config; the Vize hook preserves ordered, async addon composition without accepting executable config source. |
| `configType` and `eslintPath` are absent from checker options                           | They choose an ESLint implementation. Vize always runs oxlint + Patina, so neither setting has a meaningful equivalent.                                                                                                                                                                |
| Checker `cache` means incremental target selection                                      | Oxlint needs no ESLint cache file. `true` lints only watched changes after startup; `false` reruns the full include set.                                                                                                                                                               |
| Checker `formatter` output is engine-native                                             | Vize renders filtered oxlint diagnostics (`json`, `unix`, or a concise readable form) instead of loading ESLint formatter modules.                                                                                                                                                     |

## Directory defaults

Defaults for a hand-written config, where directory lists may be absent.
A generated config always supplies every list, so these only apply to the
standalone entry point.

| Case                    | Behaviour                                                                                         |
| ----------------------- | ------------------------------------------------------------------------------------------------- |
| `defaults/empty`        | No dirs at all: `root` becomes `[".", "./app"]` and everything else derives from it.              |
| `defaults/src-only`     | Declaring `src` derives every per-feature directory from it.                                      |
| `defaults/root-only`    | Declaring `root` makes `src` follow `root` rather than the built-in pair.                         |
| `defaults/empty-arrays` | Present-but-empty lists stay empty: an empty array is truthy, so the `\|\|=` defaults never fire. |
| `defaults/partial`      | Only missing lists are defaulted; declared ones pass through untouched.                           |

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

## Addons

| Case                    | Behaviour                                                                                                                                                        |
| ----------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `addons/import-globals` | Nuxt and Nitro registries are combined, sorted by `from` then imported `name`, and emitted in full order as readonly globals using each import's alias when set. |

`features.typescript` defaults to whether the `typescript` package resolves from
`@nuxt/eslint-config`. The recording stores that probe's answer as
`typeScriptDetected` so the offline test stays on the same branch as the
recording rather than guessing.

## Dev-server checker

| Case                   | Behaviour                                                                                                                       |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `checker/default`      | An empty object resolves every portable upstream default, including `fix: false`, and always runs outside the main thread.      |
| `checker/true`         | Boolean opt-in resolves identically to the empty object.                                                                        |
| `checker/all-explicit` | `cache`, globs, formatter, startup, warning/error emission and fixing all pass through exactly; worker execution stays enabled. |
