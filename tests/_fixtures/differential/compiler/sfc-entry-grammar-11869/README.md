# Root patterned-template SFC attribute grammar

`Layout.vue.txt` is the original regression input: quoted root-template metadata
is adjacent to `v-match` with no separating spaces. Both it and the exact
`vite.config.mjs.txt` are authenticated and materialized by the configured
compiler adapter. The configuration enables `vue.experimentals.patternedTemplate`.

`module.expected.txt` is the complete SSR module from the public v0.691.0 Darwin
CLI for `spaced-control.vue.txt`, using the same configuration. That control adds
only the spaces before `v-match` and `data-note`. Its complete public packet is
archived in `spaced-control.public-691.json`: CSS is null, errors/warnings and
macro artifacts are empty, and script language is `js`. No candidate output is
rewritten, normalized or partially compared.

Capture command (with the selected source materialized as `Layout.vue`):

```sh
vize build Layout.vue --format json --output out --ssr --config vite.config.mjs
```

The public CLI fails on the original adjacent input with:

```text
Missing whitespace between attributes; inferred a new attribute boundary.
```

The reference therefore records a transformed-input control, not pre-fix
success for the regression input. After the fix, the original input must match
the whole expected module and all adapter facets. The adjacent root-header
behavior is additionally tested directly across VDOM, SSR and Vapor.
