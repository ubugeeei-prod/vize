# Two opt-in checks join opinionated

Issue [#9059](https://github.com/ubugeeei-prod/vize/issues/9059). The maintainer asked
for more rules on by default in each preset. `opinionated` is the strict preset,
and it now enables two rules that no preset enabled before.

| Rule                           | Presets               | Corpus findings / projects |
| ------------------------------ | --------------------- | -------------------------- |
| `script/require-prop-types`    | `opinionated`         | 229 / 9                    |
| `vue/no-use-v-else-with-v-for` | `opinionated`, `nuxt` | 21 / 10                    |

`script/require-prop-types` mirrors eslint-plugin-vue's strongly-recommended rule.
It reports array-form props and descriptors without a type, including mistakes
such as `uid: { String, required: true }` in `vue-data-ui`. It joins the other
`opinionated`-only script rules. `vue/no-use-v-else-with-v-for` joins the template
rules that `opinionated` and `nuxt` share.

The corpus is the 34 `tests/_fixtures/_git` projects used for the
[Vue 3 removal slice](./2026-10-14-9059-preset-vue3-removals.md), across every
lintable file type. Every finding was read, and the run found two kinds of false
positive in `require-prop-types`. This change fixes both.

- A descriptor built from a spread, such as
  `offset: { ...useDropdownableControlProps.offset, default: … }` in `vuestic-ui`,
  can carry its type through the spread. It now counts as typed.
- Plain modules with a `props` object were treated as components. Examples are the
  docs API option files in `vuestic-ui` and a mixin in `hoppscotch`. The rule now
  uses the component-module gate from the removal slice: `.vue`, `.jsx` and `.tsx`
  files, plus modules that call `defineComponent`.

## Not affected

n8n's configuration uses `preset: "incremental"`, and its explicit rule list
contains neither rule.
