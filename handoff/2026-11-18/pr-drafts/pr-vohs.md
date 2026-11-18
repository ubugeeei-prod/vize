## Summary

`vue/v-on-handler-style` has a documented Patina default that differs from eslint-plugin-vue's. Patina reports every inline function and statement, while upstream defaults to `["method", "inline-function"]`, which accepts a handler that a method reference cannot express. The Real Project Matrix (#12336) ran Patina's default against upstream's, so each such handler counted as a false positive (977 findings).

The rule map now lists matrix-only overrides under `matrixRuleOptions`, and the lint divergence report passes them to Patina through a generated `linter.ruleOptions` config (`--config` in place of `--no-config`). That config is removed after the run.

An override is refused unless:

- the rule's source documents the default difference on the named line;
- the module doc header spells the upstream options;
- a named test in that source keeps Patina's own default reporting the extra shapes, here `reports_handler_that_closes_over_v_for_variables`.

The report schema is unchanged. The lint-report parity helper maps the removed generated config back to `--no-config`, so the frozen reporters compare as before.

## Tests

- `lint-matrix-rule-options.test.ts`: the only override is v-on-handler-style, and the config text is exact. A missing doc line, header or guard is refused.
- The rule-map successor ledger gains `inverseMatrixRuleOptions12336`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
