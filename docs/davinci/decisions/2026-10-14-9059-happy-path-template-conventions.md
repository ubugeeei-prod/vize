# Four template conventions join happy-path

Issue [#9059](https://github.com/ubugeeei-prod/vize/issues/9059). The maintainer asked
for more rules on by default in each preset. This slice moves four rules from
`opinionated` and `nuxt` into `happy-path`. `ecosystem` inherits them, and
`opinionated` and `nuxt` keep them.

| Rule                     | eslint-plugin-vue category | Corpus findings / projects |
| ------------------------ | -------------------------- | -------------------------- |
| `vue/use-v-on-exact`     | essential                  | 4 / 3                      |
| `vue/this-in-template`   | recommended                | 2 / 2                      |
| `vue/no-template-shadow` | strongly recommended       | 26 / 8                     |
| `vue/v-bind-style`       | strongly recommended       | 30 / 1                     |

The corpus is the 34 `tests/_fixtures/_git` projects used for the
[Vue 3 removal slice](./2026-10-14-9059-preset-vue3-removals.md). Every finding
was read, and each one is a true positive. For example, `use-v-on-exact` reports a
plain `@keydown` next to a `@keydown.shift` listener. `no-template-shadow` reports
nested `v-for` aliases that reuse an outer name. `happy-path` already enables
`vue/v-on-style` and `vue/v-slot-style`, so `vue/v-bind-style` completes that
group.

`vue/use-v-on-exact` was already in `essential`, so `essential` was not a subset of
`happy-path`. This slice restores that.

## Autofix

The `vue/v-bind-style` fix rebuilt the whole attribute from its argument and
value. It dropped modifiers (`v-bind:title.sync.camel` became `:title`), turned
`v-bind:[key]` into `:key`, and rewrote single-quoted values. The fix now replaces
only the `v-bind:` or `:` prefix. `.prop` shorthand matches neither style, so the
rule leaves it alone. The editor quick fix in the
[#7998 fixture](../../../tests/_fixtures/differential/lsp/bind-style-code-actions/)
now edits only the prefix. `vize lint --fix` still converges to `:title`.

## Not affected

n8n's configuration uses `preset: "incremental"`. Its explicit rule list contains
`vue/use-v-on-exact`, whose behavior does not change, and none of the other three
rules.
