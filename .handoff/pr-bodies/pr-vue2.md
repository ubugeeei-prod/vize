## Summary

The Real Project Matrix (#12336) ran Patina with no config, so its 33 Vue 2 and 2.7 projects were linted as Vue 3. A user's project would select its dialect with `vue.version`. The matrix config (from the v-on-handler-style PR this one stacks on) now also carries `vue.version`:

- `"2"` for 29 projects and `"2.7"` for 4 (habitica, vue-material, vue-grid-layout, tdesign), whenever the compatibility ledger's evidence (pinned packages, test oracles, registry notes) proves only that major;
- nothing for mixed-major projects such as gogocode, vant-demo, vue-cropper, vuelidate and vue-devtools-v6, which keep Patina's default.

The Vue 2 dialect turns off Patina's Vue 3 migration rules, such as `vue/no-deprecated-filter`, `vue/no-deprecated-slot-attribute` and `script/no-deprecated-destroyed-lifecycle`, because they flag correct Vue 2 code. The eslint-plugin-vue baseline still applies them, so their upstream findings in those projects are classified with that reason. The rule list is parsed from the legacy block of `Linter::with_vue_version` (`vue2_dialect.ts`), so it cannot drift from Patina. Patina has no findings for those rules there, so nothing local can hide behind the classification.

## Local measurement

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

## Tests

- `lint-matrix-rule-options.test.ts`: vue2-elm maps to `"2"` and habitica to `"2.7"`; airi, gogocode and a missing fixture map to nothing; the config text for a 2.7 project is exact.
- `lint-vue2-dialect.test.ts`: the rule list holds the migration rules and not ordinary ones, and only the legacy block is read, so the Vapor block's `script/no-next-tick` stays out.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
