# #6097 configurable-rule audit

Date: 2026-10-21

## Authority

- Current upstream release: [`eslint-plugin-vue` v10.12.0](https://github.com/vuejs/eslint-plugin-vue/releases/tag/v10.12.0), published 2026-10-26.
- Release commit: [`f2a0cb56c4778487b6b182fe4be5817f858990e7`](https://github.com/vuejs/eslint-plugin-vue/tree/f2a0cb56c4778487b6b182fe4be5817f858990e7).
- The first audit used v10.11.1 (`ec9c53a6fbed637d7b0fbcd6893b0206ae32839f`, published 2026-09-24).
- The committed inventory is [`patina-eslint-vue-configurable-audit.json`](../../../tests/_fixtures/patina-eslint-vue-configurable-audit.json).

The audit enumerates each unique top-level rule module at the release commit and treats a non-empty
`meta.schema` as configurable. That produces 253 rules, of which 112 accept options.

## Result

Of the 112 configurable upstream rules, the existing compatibility map classifies 58 as mapped,
52 as unimplemented, and 2 as intentional divergences. Thirteen mapped Patina rules expose typed
project configuration today; 45 mapped rules still have no typed option surface.

The machine check verifies the complete configurable-rule list, status totals, mapped Patina rule
IDs, and their public JSON Schema entries. The smallest high-value next slice is
`vue/v-slot-style`: it is strongly recommended, fixable, already implements authored ranges and
safe fixes, and has both uniform and position-specific behavior internally.

The stacked implementation updates that count to 14 configured mapped rules and leaves 44 mapped
rules without typed options. It exposes both upstream-compatible uniform values and the richer
position-specific object while rejecting bare `v-slot` for named slots at the config boundary.

The next cohort adds `vue/html-quotes`, a strongly recommended, fixable rule, bringing the typed
surface to 15 configured mapped rules and leaving 43. It accepts `"double"` or `"single"` as a
compact style and an object form with `style` plus upstream's `avoidEscape` policy.

The following cohort adds `vue/mustache-interpolation-spacing`, another strongly recommended,
fixable rule, bringing the typed surface to 16 configured mapped rules and leaving 42. It accepts
`"always"` or `"never"` and preserves the rule's independent, safe fixes for both delimiters.

The real-project false-positive cohort adds the upstream-compatible `allowlist` for
`vue/no-bare-strings-in-template`, bringing the current audited surface to 18 configured mapped
rules and leaving 43. The same delivery adds application allowances for the Patina-specific
`vue/warn-custom-directive` mapping and the accessibility rules `a11y/anchor-has-content` and
`a11y/label-has-for`; those three do not change this eslint-plugin-vue-only census.

The compatibility-port cohort maps nine more configurable upstream rules. Four of the mapped
counterparts expose typed options, bringing the audited surface to 70 mapped rules, including 22
with typed options, and leaving 39 unimplemented configurable rules.

The button-policy cohort adds the upstream `button`, `submit`, and `reset` booleans for
`vue/html-button-has-type`. All three default to `true`; disabling a value reports that otherwise
valid static type as forbidden while missing, empty, invalid, and populated dynamic bindings keep
their upstream boundaries. This brings the audited surface to 23 mapped rules with typed options
and leaves 47 mapped configurable rules without typed options. The upstream rule is not fixable,
so this cohort has no autofix behavior to configure.

The iteration-variable cohort adds upstream's `allowUsingIterationVar` boolean for
`vue/no-use-v-if-with-v-for`. It defaults to `false`; `true` permits only same-element `v-if`
conditions that reference a sibling `v-for` variable, while unrelated conditions remain
reportable. This brings the audited surface to 24 mapped rules with typed options and leaves 46
mapped configurable rules without typed options. The upstream rule is not fixable, so the option
does not introduce an automatic rewrite.

The component-name exemption cohort adds upstream's exact `ignores` array for
`vue/multi-word-component-names`. Authored names extend the built-in `App` and Vue component
exemptions, PascalCase entries also admit their kebab-case spelling, and near matches remain
reportable. This brings the audited surface to 25 mapped rules with typed options and leaves 45
mapped configurable rules without typed options. The upstream rule is not fixable, so configured
exemptions only select which authored names receive diagnostics.

## Provider boundary

The executable differential provider remains pinned to 10.9.2. Moving it to 10.11.1 changes
reviewed real-project observations, so that move requires separate corpus review rather than a
metadata-only refresh. The only rule added since the pinned provider is
`vue/no-shadow-native-events`; it has no options, so it does not change the 112-rule configurable
surface audited here.

The checked-in compatibility scorecard advances independently to 10.12.0. That metadata advance
does not move the executable differential provider.

## 10.12.0 refresh

The audit now follows the scorecard to 10.12.0, which has 256 rules. The three rules added since
10.11.1 are `vue/no-content-outside-block`, `vue/no-define-model-default`, and
`vue/no-props-shadow`. None of them accepts options, so the configurable surface is still the same
112 rules. A rule that wraps an ESLint core or Stylistic rule (`wrapCoreRule`) inherits that
rule's schema only when ESLint loads it. Its own module declares no schema, so the census counts
it as not configurable, as the first audit did. All four rules added since the pinned provider
are now mapped.

`vue/block-lang` now counts as a rule with typed options. Its `script`, `template`, and `style`
entries accept upstream's `lang` and `allowNoLang`. This brings the audited surface to 42 mapped
rules with typed options and leaves 46 mapped configurable rules without them. The status totals
are unchanged: 88 mapped, 23 unimplemented, and 1 intentional divergence.

## Template cohort 1

Six template rules now accept their eslint-plugin-vue options under the same names and defaults:
`vue/no-duplicate-attributes` (`allowCoexistClass`, `allowCoexistStyle`),
`vue/no-multiple-template-root` (`disallowComments`), `vue/no-lone-template`
(`ignoreAccessible`), `vue/no-static-inline-styles` (`allowBinding`), `vue/no-unused-components`
(`ignoreWhenBindingPresent`), and `vue/no-v-html` (`ignorePattern`). This brings the audited surface
to 48 mapped rules with typed options and leaves 40 mapped configurable rules without them.

The pinned fixture `tests/_fixtures/differential/linter/template-rule-options` records the ranges
eslint-plugin-vue 10.12.0 reports for 21 source and option rows, and the Patina rows must report
the same ranges. Two boundaries are recorded rather than matched. `ignorePattern` uses Rust regex
syntax, so a JavaScript-only construct such as a lookbehind fails to compile and ignores nothing
instead of failing the configuration. With `ignoreWhenBindingPresent: false`, a template that only
the HTML recovery lane can read still withholds unused conclusions, because that lane cannot tell a
literal `is` from a dynamic one.

Options are applied once, when the linter is configured, by replacing the registered rule. A linter
with no template rule options never reaches that code, and the rules' default fields reproduce the
previous behavior.
