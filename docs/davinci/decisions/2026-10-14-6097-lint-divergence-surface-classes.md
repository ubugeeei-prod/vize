# Lint divergence surface classes (#6097)

Issue: [#6097](https://github.com/ubugeeei-prod/vize/issues/6097).

The Real Project Matrix on main `b197d9de26` scored 121 `vue/attribute-hyphenation` false negatives and 50 `vue/no-unused-components` false positives against eslint-plugin-vue 10.9.2. Reading the findings showed that almost all of them are not parity gaps. The comparator in `tools/commands/fixtures/lint-divergence-report.rs` (helpers in `tools/support/lint_divergence_surface.rs`) now classifies these findings instead of scoring them:

- **camelCase `aria*` / `data*` component props (119 of 121).** `vue/attribute-hyphenation` deliberately exempts names such as `ariaLabel` and `:dataKey` on components. Type checkers (vue-tsc and Vize) read hyphenated `aria-*` / `data-*` names as fallthrough attributes, so `aria-label` would drop the prop's type check and could not satisfy a required `ariaLabel` prop. eslint-plugin-vue's findings for these names are recorded as `intentionalDivergences`, together with the reason.
- **Unused `<script setup>` imports (50 of 50).** Vize reports an imported component that the template never uses. eslint-plugin-vue's rule checks only the Options API `components` option. The sampled findings (misskey, vue-data-ui) are genuinely unused imports. They are recorded as `patinaOutsideBaselineSurface`, like Patina findings inside a pug template.

Every other finding of both rules is still scored, and no budget changes.

The remaining two hyphenation false negatives are not rule bugs either:

- gridea `Main.vue:59` is a matched finding whose column differs by one. The line contains an emoji: Vize reports columns in Unicode code points, while ESLint counts UTF-16 units. The same shift affects three other matched findings in gridea and better-scroll. Aligning the JSON output column unit is tracked separately.
- vue-data-ui reports `fontSize` on an SVG `<text>` element inside a slot. eslint-plugin-vue cannot see the SVG parent and treats `<text>` as a component. Vize knows it is an SVG element.

`tests/tooling/lint-divergence-surface-classes.test.ts` runs the real comparator and ESLint baseline over one SFC containing both classes and an ordinary hyphenation finding. It checks that the classified findings leave the false-positive and false-negative lists and that the ordinary finding is still shared.
