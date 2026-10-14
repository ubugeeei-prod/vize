# Three template conventions join happy-path

Issue [#9059](https://github.com/ubugeeei-prod/vize/issues/9059). The maintainer asked
for more rules on by default in each preset. This slice moves three rules from
`opinionated` and `nuxt` into `happy-path`. `ecosystem` inherits them, and
`opinionated` and `nuxt` keep them.

| Rule                     | eslint-plugin-vue category | Corpus findings / projects |
| ------------------------ | -------------------------- | -------------------------- |
| `vue/use-v-on-exact`     | essential                  | 4 / 3                      |
| `vue/this-in-template`   | recommended                | 2 / 2                      |
| `vue/no-template-shadow` | strongly recommended       | 26 / 8                     |

The corpus is the 34 `tests/_fixtures/_git` projects used for the
[Vue 3 removal slice](./2026-10-14-9059-preset-vue3-removals.md). Every finding
was read, and each one is a true positive. For example, `use-v-on-exact` reports a
plain `@keydown` next to a `@keydown.shift` listener, and `no-template-shadow`
reports nested `v-for` aliases that reuse an outer name.

`vue/use-v-on-exact` was already in `essential`, so `essential` was not a subset of
`happy-path`. This slice restores that.

## `vue/v-bind-style` stays in `opinionated`

`vue/v-bind-style` reports 30 true positives in one project and would complete
the `v-on-style` and `v-slot-style` group in `happy-path`. It stays in `opinionated`
because the default-correctness corpus pins the whole `happy-path` packet for a
`v-bind:title.sync` carrier against a hashed original. Moving the rule would need
a new authority for that packet, so it is left for a separate decision.

Its autofix is fixed in this slice. The fix rebuilt the whole attribute from its
argument and value, so it dropped modifiers (`v-bind:title.sync.camel` became
`:title`), turned `v-bind:[key]` into `:key`, and rewrote single-quoted values. It
now replaces only the `v-bind:` or `:` prefix. `.prop` shorthand matches neither
style, so the rule leaves it alone. The editor quick fix in the
[#7998 fixture](../../../tests/_fixtures/differential/lsp/bind-style-code-actions/)
now edits only the prefix, and `vize lint --fix` still converges to `:title`.

## Not affected

n8n's configuration uses `preset: "incremental"`. Its explicit rule list contains
`vue/use-v-on-exact`, whose behavior does not change, and neither of the other two
rules.
