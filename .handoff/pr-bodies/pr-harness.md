## Summary

This PR changes the matrix harness only: Patina is untouched. It bundles three harness fixes to the Real Project Matrix (#12336) that sit on top of the isolated fixtures from #13286. Each item is its own commit, so any one can be reverted alone. With the other lane PRs, the local all-branch measurement on the 57 projects shared with run 44093557816 goes from 5230 FP / 3645 FN to 5 FP / 2 FN.

### 1. The baseline reports findings at the lines their nodes occupy

vue-eslint-parser 10.4 can return `<script setup>` nodes whose `loc` disagrees with their `range` when the SFC also has a plain `<script>`. In scalar `packages/api-client/src/v2/components/code-input/CodeInput.vue`, every setup node sits 29 lines below its range. Rules report `loc`, so the baseline put findings on lines that hold other code, and each one split into a false positive and a false negative (24 pairs in scalar alone).

The scalar reproduction is pinned as `tests/_fixtures/lint-baseline-script-locations/TwoScripts.vue.txt` (52 lines, reduced from `CodeInput.vue`: a plain `<script>` followed by `<script setup>`). vue-eslint-parser 10.4 places its `<script setup>` nodes 29 lines below their ranges, past the end of the file. The baseline parser now gives every script node, token and comment the `loc` its own `range` names. Template nodes are untouched.

The use-v-on-exact oracle pins this helper by hash, so its receipt moves to the new digest. Its packets are unchanged.

### 2. vue/v-on-handler-style runs with eslint-plugin-vue's default styles

`vue/v-on-handler-style` has a documented Patina default that differs from eslint-plugin-vue's. Patina reports every inline function and statement, while upstream defaults to `["method", "inline-function"]`, which accepts a handler that a method reference cannot express. The Real Project Matrix (#12336) ran Patina's default against upstream's, so each such handler counted as a false positive (977 findings).

The rule map now lists matrix-only overrides under `matrixRuleOptions`, and the lint divergence report passes them to Patina through a generated `linter.ruleOptions` config (`--config` in place of `--no-config`). That config is removed after the run.

An override is refused unless:

- the rule's source documents the default difference on the named line;
- the module doc header spells the upstream options;
- a named test in that source keeps Patina's own default reporting the extra shapes, here `reports_handler_that_closes_over_v_for_variables`.

The report schema is unchanged. The lint-report parity helper maps the removed generated config back to `--no-config`, so the frozen reporters compare as before.

#### Tests

- `lint-matrix-rule-options.test.ts`: the only override is v-on-handler-style, and the config text is exact. A missing doc line, header or guard is refused.
- The rule-map successor ledger gains `inverseMatrixRuleOptions12336`.

### 3. Vue 2 projects are linted with their Vue 2 dialect

The Real Project Matrix (#12336) ran Patina with no config, so its 33 Vue 2 and 2.7 projects were linted as Vue 3. A user's project would select its dialect with `vue.version`. The matrix config from item 2 now also carries `vue.version`:

- `"2"` for 29 projects and `"2.7"` for 4 (habitica, vue-material, vue-grid-layout, tdesign), whenever the compatibility ledger's evidence (pinned packages, test oracles, registry notes) proves only that major;
- nothing for mixed-major projects such as gogocode, vant-demo, vue-cropper, vuelidate and vue-devtools-v6, which keep Patina's default.

The Vue 2 dialect turns off Patina's Vue 3 migration rules, such as `vue/no-deprecated-filter`, `vue/no-deprecated-slot-attribute` and `script/no-deprecated-destroyed-lifecycle`, because they flag correct Vue 2 code. The eslint-plugin-vue baseline still applies them, so their upstream findings in those projects are classified with that reason. The rule list is parsed from the legacy block of `Linter::with_vue_version` (`vue2_dialect.ts`), so it cannot drift from Patina. Patina has no findings for those rules there, so nothing local can hide behind the classification.

#### Local measurement

Same binary, six Vue 2 projects:

| Project | Before FP / FN | After FP / FN | Upstream migration-rule findings classified |
| --- | --- | --- | --- |
| vue2-elm | 10 / 10 | 10 / 10 | 20 |
| vux | 72 / 110 | 72 / 110 | 346 |
| element | 83 / 77 | 77 / 77 | 146 |
| vue-element-admin | 16 / 17 | 16 / 17 | 164 |
| cube-ui | 42 / 45 | 42 / 45 | 176 |
| habitica (2.7) | 26 / 36 | 26 / 36 | 348 |

element's 6 fewer false positives are `vue/v-on-handler-style` findings that the Vue 2 dialect reads as upstream does.

#### Tests

- `lint-matrix-rule-options.test.ts`: vue2-elm maps to `"2"` and habitica to `"2.7"`; airi, gogocode and a missing fixture map to nothing; the config text for a 2.7 project is exact.
- `lint-vue2-dialect.test.ts`: the rule list holds the migration rules and not ordinary ones, and only the legacy block is read, so the Vapor block's `script/no-next-tick` stays out.

## Tests

- `lint-baseline-script-locations.test.ts`: script nodes, tokens and comments report the lines their ranges name, and template nodes keep theirs.
- `lint-matrix-rule-options.test.ts` and `lint-vue2-dialect.test.ts`, as described above.
- The `lint-divergence*`, isolation and rule-location tooling suites pass with the combined `main.ts` (isolation, matrix config and dialect).

🤖 Generated with [Claude Code](https://claude.com/claude-code)
