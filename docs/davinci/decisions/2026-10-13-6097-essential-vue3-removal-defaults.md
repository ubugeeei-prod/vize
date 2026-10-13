# Vue 3 removal checks in the essential tier (#6097)

Issue: [#6097](https://github.com/ubugeeei-prod/vize/issues/6097). This is the third default slice after [the first](./2026-10-06-lint-default-correctness.md) and [the second](./2026-10-10-lint-default-migrations.md) (both under the closed [#8090](https://github.com/ubugeeei-prod/vize/issues/8090)), which together enabled six removed-syntax template checks by default.

Nine more implemented rules that mirror eslint-plugin-vue's `vue3-essential` removal checks were opt-in only: no bundle enabled them, so a Vue 3 project using a removed Vue 2 API got no diagnostic unless it named the rule. They now belong to `essential` and therefore to every bundle built on it (`happy-path`, `ecosystem`, `nuxt`, `opinionated`). `incremental` stays empty.

| Rule                                          | eslint-plugin-vue rule                     | Reports                                           |
| --------------------------------------------- | ------------------------------------------ | ------------------------------------------------- |
| `vue/no-deprecated-filter`                    | `vue/no-deprecated-filter`                 | `{{ value \| filter }}` pipes                     |
| `vue/no-deprecated-html-element-is`           | `vue/no-deprecated-html-element-is`        | `is` on native elements without the `vue:` prefix |
| `vue/no-deprecated-inline-template`           | `vue/no-deprecated-inline-template`        | the `inline-template` attribute                   |
| `vue/no-deprecated-router-link-tag-prop`      | `vue/no-deprecated-router-link-tag-prop`   | `<router-link tag>`                               |
| `script/no-deprecated-destroyed-lifecycle`    | `vue/no-deprecated-destroyed-lifecycle`    | `destroyed` / `beforeDestroy` hooks               |
| `script/no-deprecated-dollar-listeners-api`   | `vue/no-deprecated-dollar-listeners-api`   | `this.$listeners`                                 |
| `script/no-deprecated-dollar-scopedslots-api` | `vue/no-deprecated-dollar-scopedslots-api` | `this.$scopedSlots`                               |
| `script/no-deprecated-events-api`             | `vue/no-deprecated-events-api`             | `this.$on` / `$off` / `$once`                     |
| `script/no-deprecated-props-default-this`     | `vue/no-deprecated-props-default-this`     | `this` inside a prop default                      |

All nine keep their `error` severity. Each reports code that Vue 3 no longer runs as written, which is the definition of the essential tier. They are inert for a project configured as Vue 2 or 2.7: `with_vue_version` disables them for a legacy version, and the template rules additionally check the dialect. A project can still turn any of them off by name.

This changes default output. A Vue 3 project that still contains one of these constructs gains an error it did not have before.

## Not promoted

- `script/no-deprecated-data-object-declaration` stays opt-in. Options API script rules treat a default-exported object in a plain JS/TS module as a component, so `export default { data: { … } }` in a config or chart-options module would be reported. `lint_script_allows_plain_config_object_with_option_like_keys` covers that case. Promote it once plain modules count as components only through an explicit `defineComponent`-style wrapper.
- `script/require-default-prop`, `script/require-prop-types` and `script/require-explicit-emits` (`vue3-strongly-recommended`) are style and API-contract conventions rather than removed behavior. They are candidates for the `opinionated` bundle, not for the default.
- `vue/no-multiple-template-root` is a Vue 2 restriction and would be wrong for Vue 3.

## Coverage

`crates/vize_patina/src/preset/vue3_removal_defaults_tests.rs` lints one SFC containing every construct and one using the Vue 3 replacements. It asserts that every bundle except `incremental` reports all nine rules for the first and none for the second, that Vue 2 and 2.7 projects get none, that each rule can be disabled alone, and that no bundle reports the plain-module `data` object. The bundle membership snapshot, the eslint-plugin-vue rule map and the generated rule pages record the new membership.

The Real Project Matrix compares these rules against eslint-plugin-vue from the first run after this merges; any divergence it finds is tracked on #6097.
