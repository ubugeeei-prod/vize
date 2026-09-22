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

## FN-1 — class (a) undefined-template-ref: `vue/no-undefined-refs` is unreachable

**Measured recall:** 0/49 on the corpus shard, 0/3 on the miniature set —
identity-verified, every miss listed by exact location and identifier.

**Witness (registration gap, verified against the sources and the CLI):**
the rule is implemented
(`crates/vize_patina/src/rules/vue/no_undefined_refs.rs`), exported, and
listed in `SEMANTIC_TEMPLATE_RULES`
(`crates/vize_patina/src/linter/engine/rule_sets.rs`), but no
`RuleRegistry` preset constructor (`crates/vize_patina/src/rule.rs`) and no
opt-in registration path (`register_opt_in` in
`crates/vize_patina/src/rules/vue.rs`) ever instantiates it. Config-enabling
`"vue/no-undefined-refs": "warn"` is a no-op: `with_additional_rules` only
adds rule _names_ to the enabled set, it cannot summon an unregistered rule
instance (verified with a live `vize lint` run; a config-enabled opt-in rule
fires, this rule does not).

The P0-13 plan sentence "current Patina must flag 100% of seeded class-(a)
instances" is therefore falsified by the pilot — which is precisely the
assumption-testing job the oracle exists to do. The identity-assertion
_mechanism_ is proven green both ways in CI (a synthetic diagnostic set
matching the manifest passes; a same-count wrong-location set fails listing
the exact miss).

**Disposition:** `justified-with-witness` for default-preset recall;
config-enable is fixed.

- Default presets still do not instantiate the rule, so the seeded-defect
  pilot remains 0/3 (`tests/_fixtures/davinci-fpfn/expected/assert-report.json`).
  Putting it on a default preset is a separate FP-audit change.
- `#4636` / `fix/patina-undefined-refs-dispatch` registers the rule on the
  opt-in path. `"vue/no-undefined-refs": "warn"` now instantiates and runs
  the rule. The engine rule-name set is no longer a dead gate entry.

The CI expectation still pins the measured default-preset 0/3. Flip that
pin in the same change that adds the rule to a default preset.

## FN-2 — class (b) unused-binding: `unused_bindings` has no lint consumer

**Measured recall:** 0/130 on the corpus shard, 0/90 on the matrix stubs,
0/4 on the miniature set.

**Witness:** `vize_croquis` computes `unused_bindings`
(`crates/vize_croquis/src/croquis.rs`), but no lint rule consumes it —
`vue/no-unused-vars` covers only `v-for`/`v-slot` variables
(`crates/vize_patina/src/rules/vue/no_unused_vars.rs`). This is the gap the
P0-13 plan documents by design; the pilot turns it into a measured number.

Caveat recorded for the future flip: the seeded identifier
(`__davinci_seeded_unused`, mandated by the P0-13 spec) is
underscore-prefixed, and existing unused-checks treat `_`-prefixed names as
intentionally unused — when a consumer lands, the seed name must be
revisited together with this entry.

**Disposition:** `deferred-with-issue` — the consumer arrives with the rule
SDK / fact-channel work (assurance doctrine, precision tiers); explicitly
not a phase-0 gate per the P0-13 plan text.

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
- `script/define-emits-declaration`
- `script/define-macros-order`
- `script/define-props-declaration`
- `script/define-props-destructuring`
- `script/no-arrow-functions-in-watch`
- `script/no-async-in-computed`
- `script/no-boolean-default`
- `script/no-deep-destructure-in-props`
- `script/no-deprecated-data-object-declaration`
- `script/no-deprecated-destroyed-lifecycle`
- `script/no-deprecated-dollar-listeners-api`
- `script/no-deprecated-dollar-scopedslots-api`
- `script/no-deprecated-events-api`
- `script/no-deprecated-props-default-this`
- `script/no-dupe-keys`
- `script/no-export-in-script-setup`
- `script/no-get-current-instance`
- `script/no-import-compiler-macros`
- `script/no-internal-imports`
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
- `script/no-with-defaults`
- `script/prefer-define-options`
- `script/prefer-import-from-vue`
- `script/prefer-use-attrs`
- `script/prefer-use-slots`
- `script/require-default-prop`
- `script/require-function-return-type`
- `script/require-prop-type-constructor`
- `script/require-prop-types`
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
- `vue/component-definition-name-casing`
- `vue/component-name-in-template-casing`
- `vue/html-button-has-type`
- `vue/html-self-closing`
- `vue/multi-word-component-names`
- `vue/mustache-interpolation-spacing`
- `vue/no-array-index-key`
- `vue/no-boolean-attr-value`
- `vue/no-deprecated-filter`
- `vue/no-deprecated-functional-template`
- `vue/no-deprecated-html-element-is`
- `vue/no-deprecated-inline-template`
- `vue/no-deprecated-router-link-tag-prop`
- `vue/no-deprecated-scope-attribute`
- `vue/no-deprecated-slot-attribute`
- `vue/no-deprecated-slot-scope-attribute`
- `vue/no-deprecated-v-bind-sync`
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
- `vue/no-reserved-component-names`
- `vue/no-root-v-if`
- `vue/no-script-non-standard-lang`
- `vue/no-src-attribute`
- `vue/no-static-inline-styles`
- `vue/no-template-lang`
- `vue/no-template-shadow`
- `vue/no-template-target-blank`
- `vue/no-unsandboxed-iframe`
- `vue/no-unused-vars`
- `vue/no-use-v-else-with-v-for`
- `vue/no-use-v-if-with-v-for`
- `vue/no-useless-mustaches`
- `vue/no-useless-template-attributes`
- `vue/no-useless-v-bind`
- `vue/no-v-for-template-key-on-child`
- `vue/no-v-text`
- `vue/no-v-text-v-html-on-component`
- `vue/prefer-props-shorthand`
- `vue/prefer-true-attribute-shorthand`
- `vue/prop-name-casing`
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
- `vue/valid-template-root`
- `vue/valid-v-bind`
- `vue/valid-v-cloak`
- `vue/valid-v-for`
- `vue/valid-v-html`
- `vue/valid-v-if`
- `vue/valid-v-memo`
- `vue/valid-v-model`
- `vue/valid-v-on`
- `vue/valid-v-once`
- `vue/valid-v-show`
- `vue/valid-v-slot`
- `vue/valid-v-text`
- `vue/warn-custom-block`
- `vue/warn-custom-directive`
<!-- /p4-15a-unclassed -->
