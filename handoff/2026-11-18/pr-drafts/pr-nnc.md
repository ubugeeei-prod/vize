`vue/no-negated-condition` now reports the same node as eslint-plugin-vue 10.12.0 (ESLint's core `no-negated-condition`, wrapped for template expressions):

- The range covers the whole `ConditionalExpression` (`!a ? b : c`), or the whole `IfStatement` in a handler (`if (!a) x(); else y()`). Before, it covered only the test.
- Parentheses are still looked through, but TypeScript wrappers are not, as in ESTree. `(!a as boolean) ? b : c` is no longer reported; `!(a as boolean) ? b : c` still is.

In the Real Project Matrix (#12336) this rule caused 497 false positives and 504 false negatives (schedule run 43880794806). The two sides were the same findings with different ranges. A local `lint-divergence-report.ts --all-mapped-rules` run over filebrowser, vuepress, vue-flow, elk, mealie, element, vuetify and naive-ui now shows 0 divergences for this rule.

New tests in `compatibility_template/tests/negated_condition.rs` cover:
- interpolation and attribute ternaries, parenthesized and multi-line;
- `if`/`else` handlers with and without braces;
- `else if` chains and `if` without `else`, which stay quiet;
- the TypeScript-wrapper cases.

Linter cost is unchanged: the same visitor, with a narrower test unwrap.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
