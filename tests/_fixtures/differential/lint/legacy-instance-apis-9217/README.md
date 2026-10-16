# Legacy instance API references (#9217)

[Issue #9217](https://github.com/ubugeeei-prod/vize/issues/9217) came from the
Real Project Matrix at `d2506ca6b5e31252973f05981829c0367cacf26b`.

`independent.json` preserves 43 authored SFC inputs, source SHA-256 hashes and
complete diagnostics from ESLint 10.4.1, eslint-plugin-vue 10.9.2,
vue-eslint-parser 10.4.1 and @typescript-eslint/parser 8.65.0. Rule implementation
hashes identify the provider bytes. The executable provider replay is
`tests/tooling/patina-legacy-instance-api-reference.test.mjs`; the native exact
rule/token check is `crates/vize_patina/tests/legacy_instance_apis.rs`. Neither
changes budgets or adjusts diagnostic ranges to make findings agree.

`originals/manifest.json` records immutable repository revisions, the complete
71-file affected source inventory and hashes of five original divergence
reports. `sources.json.gz` retains those complete upstream source bytes, their
individual hashes and raw-source URLs. Each project's `*-lint-divergence.json.gz`
is the complete original Matrix report, including unrelated rules and failures.
`originals/independent.json.gz` retains every reference diagnostic on those 71
sources with only the two rules in this issue enabled. Source provenance and
upstream licensing remain attached to the repositories linked by the manifest;
these captures are regression evidence, not vendored production dependencies.

The pinned rule source establishes the behavior:

- [Events API](https://github.com/vuejs/eslint-plugin-vue/blob/v10.9.2/lib/rules/no-deprecated-events-api.js)
  checks nonoptional calls on `this` or a direct lexical `const alias = this`,
  and reports the property token. An unrelated event hub such as
  `this.$root.$on` or `this.$modal.event.$on` remains silent. Optional member
  access is distinguishable from an optional call.
- [Listeners API](https://github.com/vuejs/eslint-plugin-vue/blob/v10.9.2/lib/rules/no-deprecated-dollar-listeners-api.js)
  checks this-bound script member properties and unbound template references.
  Template loop, slot and expression-local bindings suppress only their own
  lexical references; comments, text, literal attributes and `v-pre` do not
  create references. The script selector accepts Identifier properties,
  including `this[$listeners]`, but excludes string-literal computed keys.
- [`isThis`](https://github.com/vuejs/eslint-plugin-vue/blob/v10.9.2/lib/utils/index.js)
  accepts only a direct const binding initialized with `this`; mutable bindings,
  alias chains and shadowed names are not proof of the component instance.

Component-region controls also cover non-default registered components,
`Vue.extend`, `new Vue`, imported factories and explicit `@vue/component`
markers. Ordinary objects, indirect exported objects and foreign factories do
not gain component-instance diagnostics merely by sharing method names.

Five affected Element SFCs contain JSX render functions. JavaScript SFC scripts
must admit JSX without rejecting the whole script and losing its event checks.
Explicit TypeScript keeps its angle-assertion grammar; TSX opts into JSX.

Replay the reference with the pinned benchmark dependencies installed:

```sh
node --test tests/tooling/patina-legacy-instance-api-reference.test.mjs
cargo test -p vize_patina --test legacy_instance_apis
```

For a separate pinned dependency installation, set
`VIZE_LEGACY_API_PROVIDER_MANIFEST` and
`VIZE_LEGACY_API_SCRIPT_PARSER_MANIFEST` to its package manifests. The replay
asserts exact versions and implementation hashes; it never silently substitutes
another provider. Native acceptance of the complete original projects remains
the Matrix's responsibility, separately from the independent provider replay.
