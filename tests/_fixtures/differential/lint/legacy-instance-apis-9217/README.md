# Legacy instance API references (#9217)

[Issue #9217](https://github.com/ubugeeei-prod/vize/issues/9217) came from the
Real Project Matrix at `d2506ca6b5e31252973f05981829c0367cacf26b`.

`independent.json` preserves 55 authored SFC inputs, source SHA-256 hashes and
complete diagnostics from ESLint 10.4.1, eslint-plugin-vue 10.9.2,
vue-eslint-parser 10.4.1 and @typescript-eslint/parser 8.65.0. Rule and shared utility implementation
hashes identify the provider bytes. The executable provider replay is
`tests/tooling/patina-legacy-instance-api-reference.test.mjs`; the native exact
rule/token check is `crates/vize_patina/tests/legacy_instance_apis.rs`. Neither
changes budgets or adjusts diagnostic ranges to make findings agree. Native
expectations explicitly differ in the five bounded cases in `provider-gaps.json`.

`originals/manifest.json` records immutable repository revisions, the complete
147-file affected source inventory and hashes of 18 original divergence
reports. `sources.json.gz` retains those complete upstream source bytes, their
individual hashes and raw-source URLs. Each project's `*-lint-divergence.json.gz`
is the complete original Matrix report, including unrelated rules and failures.
`originals/independent.json.gz` retains every reference diagnostic on those 147
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

The pinned provider misses TypeScript-wrapped event callees and optional calls.
Patina retains the existing warnings on proven component instances: `as`,
`satisfies` and non-null wrappers erase at runtime, while an optional call merely
avoids a crash without registering or removing any Vue 3 event handler. Complete
unchanged provider packets remain in `independent.json`; `provider-gaps.json`
records the additional exact native property spans separately. The executable
Vue 3 runtime control checks those failures, the absent optional subscriptions,
and the still-working `$emit` on the same mounted component. Wrapped foreign
receivers and wrapped calls outside component regions remain negative controls.

The old wrapped-callee positive is retained. The removed script positives were
unproven/foreign `emitter.$off`, `ctx.$listeners` (including setup context),
unbound `vm.$listeners`, and bare script `$listeners`; none establishes a Vue
instance. Genuine `this.$listeners` remains positive inside component options,
and unbound template references are now checked with lexical scopes.

`expanded-audit.json` records the complete 138-project artifact review for these
rule families, its source identity and the separate scoped-slots follow-up. The
replay recomputes its counts from the complete retained project reports.

Component-region controls also cover non-default registered components,
`Vue.extend`, `new Vue`, imported factories and explicit `@vue/component`
markers. Ordinary objects, indirect exported objects, unrelated constructors,
foreign imported `defineComponent` and foreign `.extend` calls stay outside the
provider's component regions.

Some factory boundaries deliberately follow the provider's syntactic contract:
any identifier receiver's `.component` or `.mixin` call is admitted;
`defineNuxtComponent` is admitted even when lexically shadowed; a named import
aliased to `makeComponent` is not admitted. Explicit positive and negative
packets preserve those decisions instead of guessing factory provenance.

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
`VIZE_LEGACY_API_SCRIPT_PARSER_MANIFEST` to its package manifests;
`VIZE_LEGACY_API_VUE_MANIFEST` can identify the Vue 3 runtime installation. The replay
asserts exact versions and implementation hashes; it never silently substitutes
another provider. Native acceptance of the complete original projects remains
the Matrix's responsibility, separately from the independent provider replay.
