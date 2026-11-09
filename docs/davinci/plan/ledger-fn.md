# Davinci — false-negative ledger

> [!NOTE]
> The FN oracle's triage record (assurance doctrine: "Seeded-defect recall —
> the FN oracle"). Every defect class the seeded-defect generator injects is
> asserted by **identity, not count**: the manifest records each injection's
> file, span, and expected rule id, and the `--assert` mode of
> `tools/commands/davinci/seed-defects.rs` compares the exact diagnostic set against
> it. A measured miss lands here with a disposition (`fixed` /
> `justified-with-witness` / `deferred-with-issue`) and is never left
> ambient.

## Pilot scan scope (P0-13, measured 2026-08-14)

Tool output quoted verbatim (`scope-proof` lines are printed by
`tools/commands/davinci/seed-defects.rs`); the corpus-shard run is local/nightly,
the committed miniature set runs in CI via
`tests/tooling/davinci-fpfn-pilots.test.ts`.

- Corpus shard (`splitpanes+layoutit-grid+cssgridgenerator`):
  `scope-proof: files-scanned=130 class-a-eligible=49 class-a-injections=49 class-b-injections=130`
- Matrix stubs (`matrix-gen`, element-kind × directive plane):
  `scope-proof: files-scanned=90 class-a-eligible=0 class-a-injections=0 class-b-injections=90`
  (stubs are template-only, so class (a) has no script binding to rename)
- Committed miniature set (`tests/_fixtures/davinci-fpfn`):
  `scope-proof: files-scanned=4 class-a-eligible=3 class-a-injections=3 class-b-injections=4`

## FN-1 — class (a) undefined-template-ref: fixed by default registration

**Measured recall:** 3/3 on the committed miniature set after default-preset
registration, identity-verified by exact rule, severity, and location. The
historical corpus-shard result was 0/49; a fresh hydrated shard measurement is
still required before replacing that historical number.

**Historical witness (the former registration gap, verified against the
sources and the CLI):** the rule was implemented
(`crates/vize_patina/src/rules/vue/no_undefined_refs.rs`), exported, and
listed in `SEMANTIC_TEMPLATE_RULES`
(`crates/vize_patina/src/linter/engine/rule_sets.rs`), but no former
`RuleRegistry` preset constructor (`crates/vize_patina/src/rule.rs`) and no
opt-in registration path (`register_opt_in` in
`crates/vize_patina/src/rules/vue.rs`) instantiated it. Config-enabling
`"vue/no-undefined-refs": "warn"` was therefore a no-op: `with_additional_rules`
only added rule _names_ to the enabled set and could not summon an
unregistered rule instance.

The P0-13 pilot initially falsified its 100% expectation; the now-green exact
identity assertion proves the registration fix without weakening the oracle.

**Disposition:** `fixed` for default-preset recall and config enablement.

- The default preset now instantiates the rule, so the seeded-defect pilot is
  3/3 (`tests/_fixtures/davinci-fpfn/expected/assert-report.json`).
- `#4636` / `fix/patina-undefined-refs-dispatch` registers the rule on the
  opt-in path. `"vue/no-undefined-refs": "warn"` now instantiates and runs
  the rule. The engine rule-name set is no longer a dead gate entry.

## FN-2 — class (b) unused-binding: `unused_bindings` has no lint consumer

**Measured recall (2026-09-26):** 130/130 on the corpus shard, 90/90 on
the matrix stubs, 4/4 on the miniature set, under explicit opt-in. The
prior measurements were 0/130, 0/90 and 0/4.

**Witness:** the previous `unused_bindings` field was empty producer
storage; it had no production population or lint consumer. P4-3c now
populates it on demand from the existing script AST's resolved OXC
references, subtracts template/component/directive/style reads, and
projects it through `UnusedBindings`. `vue/no-unused-setup-bindings` is
the opt-in Sound consumer. Invalid semantics, direct eval and unknown
external blocks cannot justify an unused report.

The seed is now `davinciSeededUnused`. Underscore-prefixed declarations
remain intentionally exempt; the old `__davinci_seeded_unused` did not
exercise this contract. Class (b) runs on an independent mutation tree
with complete baseline/seeded diagnostic multiset checks, preventing a
class-(a) rename from introducing a separate unused declaration.

**Disposition:** `fixed` — [P4-3c evidence](./phase-4-records/p4-3c.md).
The class-(b) measurement explicitly disables `vue/no-undefined-refs` so its
independent unused-binding diagnostic multiset remains isolated from FN-1's
default-preset closure.

## FN-3 — exact/sound rules without a seeded defect class (P4-15a)

**Measured 2026-09-22:** `--check-classes` reads every `Exact` and `Sound` row of
`crates/vize_patina/src/rule_contracts/table.rs`.
`scope-proof: exact=199 sound=0 classed-rules=20 classed-classes=40 unclassed=179 untriaged=0 stale=0`.
The 20 classed rules are the HTML-nesting generator (`vue/permitted-contents`,
recall on `--html-nesting`) and 19 snippet classes.
`seed-defects.rs --exact-classes --assert` on those snippets:
`assert: detected=19/19 unexpected=0 verdict=pass`, identity not count.
This is not TS-37 at 100% — 179 exact rules still have no class. Sound rules: 0.

The check exits **1** while any row below remains. It exits **2** when an
exact/sound rule is in neither the class registry nor this list (untriaged),
or when a listed rule is no longer an unclassed exact/sound rule (stale).
A listed rule is triaged, not waived: the exit stays 1, and the identity
assert is not relaxed to count-only or to an optional span.

**Disposition:** `deferred-with-issue` for each row — no generator yet. Delete a row only by registering its class in the same change.

### Registration census, 2026-11-05

The constant-backed metadata census registers twelve existing rules in the
contract table. Ten have an Exact tier and need entries in the unclassed list
below. Their implementation tests and documented bad/good examples remain
coverage evidence; they do not establish seeded-defect recall. No snippet class
or recall expectation is removed. `--check-classes` continues to report these
rules as unclassed and returns failure until their independent classes exist.

| Rule                                 | Existing coverage and remaining seeded-class scope                                                                               |
| ------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------- |
| `vue/comment-directive`              | Unused suppression reports require `reportUnusedDisableDirectives`; seed both a useful suppression and an unused one.            |
| `vue/match-component-file-name`      | Declared component names are tested against the authored filename; preserve filename and multiple-component controls in a seed.  |
| `vue/no-custom-modifiers-on-v-model` | Component modifier examples cover the Vue 2 compatibility restriction; seed the opt-in configuration and native-element control. |
| `vue/no-ref-object-reactivity-loss`  | AST tests cover same-scope value copies, deferred reads and writes; seed a copy without introducing an unrelated unused binding. |
| `vue/no-setup-props-reactivity-loss` | Root-scope prop-copy tests retain Vue 3.5 reactive destructuring; seed copies and deferred-read controls.                        |
| `vue/no-this-in-before-route-enter`  | Hook tests distinguish `this` from the callback's component argument; seed the function boundary explicitly.                     |
| `vue/no-v-for-template-key`          | Vue 2 key placement has bad/good examples; seed its opt-in convention separately from the conflicting Vue 3 convention.          |
| `vue/prefer-prop-type-boolean-first` | Runtime constructor-order examples cover boolean casting; seed runtime arrays and a TypeScript-union control.                    |
| `vue/require-expose`                 | Options API tests cover explicit expose lists, setup exposure, render functions and mixins; seed those boundaries.               |
| `vue/valid-model-definition`         | Vue 2 model-option tests reject unknown static keys; seed those keys without claiming to validate their values.                  |

The complete public bad/good report packets are retained in
`crates/vize_patina/tests/fixtures/documented-catalogue-additions/reports.json`.
This is triage of a class-registration gap, not a zero-FN claim.

<!-- p4-15a-unclassed -->

- `a11y/alt-text`
- `a11y/anchor-is-valid`
- `a11y/aria-props`
- `a11y/aria-role`
- `a11y/aria-unsupported-elements`
- `a11y/click-events-have-key-events`
- `a11y/heading-levels`
- `a11y/img-alt`
- `a11y/interactive-supports-focus`
- `a11y/media-has-caption`
- `a11y/mouse-events-have-key-events`
- `a11y/no-aria-hidden-on-focusable`
- `a11y/no-distracting-elements`
- `a11y/no-redundant-roles`
- `a11y/no-role-presentation-on-focusable`
- `a11y/no-static-element-interactions`
- `a11y/placeholder-label-option`
- `a11y/role-has-required-aria-props`
- `a11y/tabindex-no-positive`
- `css/no-id-selectors`
- `css/no-important`
- `css/no-v-bind-performance`
- `css/prefer-logical-properties`
- `css/require-font-display`
- `ecosystem/nuxt-prefer-nuxt-link`
- `ecosystem/router-link-require-to`
- `ecosystem/void-link-require-href`
- `ecosystem/void-link-valid-method`
- `ecosystem/vue-router-prefer-named-link`
- `ecosystem/vue-router-prefer-named-push`
- `ecosystem/vue-test-utils-no-html-snapshot`
- `html/deprecated-attr`
- `html/no-dupe-style-properties`
- `html/no-duplicate-class`
- `html/no-duplicate-dt`
- `html/no-empty-palpable-content`
- `musea/no-empty-variant`
- `musea/require-component`
- `musea/require-title`
- `musea/unique-variant-names`
- `musea/valid-variant`
- `nuxt/no-nuxt-config-test-key`
- `nuxt/nuxt-config-keys-order`
- `nuxt/prefer-import-meta`
- `petite-vue/no-unsupported-directive`
- `petite-vue/valid-v-effect`
- `petite-vue/valid-v-scope`
- `script/component-options-name-casing`
- `script/consistent-props-emits-key-quotes`
- `script/define-macros-order`
- `script/define-props-destructuring`
- `script/no-arrow-functions-in-watch`
- `script/no-async-in-computed`
- `script/no-boolean-default`
- `script/no-computed-properties-in-data`
- `script/no-deep-destructure-in-props`
- `script/no-deprecated-data-object-declaration`
- `script/no-deprecated-delete-set`
- `script/no-deprecated-destroyed-lifecycle`
- `script/no-deprecated-dollar-listeners-api`
- `script/no-deprecated-dollar-scopedslots-api`
- `script/no-deprecated-events-api`
- `script/no-deprecated-model-definition`
- `script/no-deprecated-props-default-this`
- `script/no-deprecated-vue-config-keycodes`
- `script/no-dupe-keys`
- `script/no-export-in-script-setup`
- `script/no-expose-after-await`
- `script/no-get-current-instance`
- `script/no-import-compiler-macros`
- `script/no-internal-imports`
- `script/no-lifecycle-after-await`
- `script/no-multiple-slot-args`
- `script/no-next-tick`
- `script/no-options-api`
- `script/no-required-prop-with-default`
- `script/no-reserved-identifiers`
- `script/no-reserved-keys`
- `script/no-reserved-props`
- `script/no-restricted-globals`
- `script/no-restricted-members`
- `script/no-top-level-ref-in-script`
- `script/no-watch-after-await`
- `script/no-with-defaults`
- `script/one-component-per-file`
- `script/order-in-components`
- `script/prefer-define-options`
- `script/prefer-import-from-vue`
- `script/prefer-use-attrs`
- `script/prefer-use-slots`
- `script/require-default-prop`
- `script/require-define-component`
- `script/require-function-return-type`
- `script/require-prop-type-constructor`
- `script/require-prop-types`
- `script/require-render-return`
- `script/require-slots-as-functions`
- `script/require-symbol-provide`
- `script/require-typed-object-prop`
- `script/require-typed-ref`
- `script/valid-define-emits`
- `script/valid-define-options`
- `script/valid-define-props`
- `type/require-typed-emits`
- `type/require-typed-props`
- `vapor/no-inline-template`
- `vapor/no-vue-lifecycle-events`
- `vapor/prefer-static-class`
- `vapor/require-vapor-attribute`
- `vue/a11y-img-alt`
- `vue/attribute-hyphenation`
- `vue/attribute-order`
- `vue/comment-directive`
- `vue/component-definition-name-casing`
- `vue/component-name-in-template-casing`
- `vue/enforce-style-attribute`
- `vue/html-button-has-type`
- `vue/html-self-closing`
- `vue/match-component-file-name`
- `vue/multi-word-component-names`
- `vue/mustache-interpolation-spacing`
- `vue/no-array-index-key`
- `vue/no-boolean-attr-value`
- `vue/no-content-outside-block`
- `vue/no-custom-modifiers-on-v-model`
- `vue/no-deprecated-filter`
- `vue/no-deprecated-functional-template`
- `vue/no-deprecated-html-element-is`
- `vue/no-deprecated-inline-template`
- `vue/no-deprecated-router-link-tag-prop`
- `vue/no-deprecated-scope-attribute`
- `vue/no-deprecated-slot-attribute`
- `vue/no-deprecated-slot-scope-attribute`
- `vue/no-deprecated-v-bind-sync`
- `vue/no-deprecated-v-is`
- `vue/no-deprecated-v-on-native-modifier`
- `vue/no-deprecated-v-on-number-modifiers`
- `vue/no-empty-component-block`
- `vue/no-inline-style`
- `vue/no-invalid-html-attribute`
- `vue/no-multiple-objects-in-class`
- `vue/no-multiple-template-root`
- `vue/no-negated-v-if-condition`
- `vue/no-non-component-keep-alive-child`
- `vue/no-preprocessor-lang`
- `vue/no-ref-object-reactivity-loss`
- `vue/no-reserved-component-names`
- `vue/no-root-v-if`
- `vue/no-script-non-standard-lang`
- `vue/no-src-attribute`
- `vue/no-static-inline-styles`
- `vue/no-template-lang`
- `vue/no-template-shadow`
- `vue/no-template-target-blank`
- `vue/no-this-in-before-route-enter`
- `vue/no-unsandboxed-iframe`
- `vue/no-unused-vars`
- `vue/no-use-v-else-with-v-for`
- `vue/no-use-v-if-with-v-for`
- `vue/no-useless-mustaches`
- `vue/no-useless-template-attributes`
- `vue/no-useless-v-bind`
- `vue/no-v-for-template-key`
- `vue/no-v-for-template-key-on-child`
- `vue/no-v-text`
- `vue/no-v-text-v-html-on-component`
- `vue/prefer-prop-type-boolean-first`
- `vue/prefer-props-shorthand`
- `vue/prefer-true-attribute-shorthand`
- `vue/prop-name-casing`
- `vue/require-expose`
- `vue/require-scoped-style`
- `vue/require-toggle-inside-transition`
- `vue/scoped-event-names`
- `vue/sfc-element-order`
- `vue/single-style-block`
- `vue/slot-name-casing`
- `vue/this-in-template`
- `vue/use-unique-element-ids`
- `vue/use-v-on-exact`
- `vue/v-bind-style`
- `vue/v-on-event-hyphenation`
- `vue/v-on-handler-style`
- `vue/v-on-style`
- `vue/v-slot-style`
- `vue/valid-attribute-name`
- `vue/valid-model-definition`
- `vue/valid-template-root`
- `vue/valid-v-bind`
- `vue/valid-v-cloak`
- `vue/valid-v-else-if`
- `vue/valid-v-for`
- `vue/valid-v-html`
- `vue/valid-v-if`
- `vue/valid-v-is`
- `vue/valid-v-memo`
- `vue/valid-v-model`
- `vue/valid-v-on`
- `vue/valid-v-once`
- `vue/valid-v-pre`
- `vue/valid-v-show`
- `vue/valid-v-slot`
- `vue/valid-v-text`
- `vue/warn-custom-block`
- `vue/warn-custom-directive`
- `vue/max-template-complexity`

<!-- /p4-15a-unclassed -->
