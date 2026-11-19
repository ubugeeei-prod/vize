# Handoff: feat/6871-query-imports (Q6) and feat/6871-query-vue (Q3)

Delete this file before merge.

## State

- Stack: `main` ← #13379 Q2 (`feat/6871-query-model`, in the merge queue) ←
  `feat/6871-query-imports` (Q6, `fcc686c4f4`) ← `feat/6871-query-vue` (Q3, `d15363b928`, plus this file).
- Q6 code is complete: `tests/tooling/davinci-query-imports.test.ts`,
  `docs/davinci/plan/query-import-allowlist.json` (27 entries), `levels.query` is kind `both` in
  `docs/davinci/completion-gates.md` and `tools/commands/davinci/completion-gates.ts`, plus a
  gates-tool test, decision 9 in `docs/davinci/decisions/2026-11-19-6871-query-model.md`, and
  Patina's `FileIssue` import moved to `vize_l2::query`. `node --test` on both tooling tests passes (11 tests).
  CI clippy on patina and maestro passes.
- Q3 code is complete: `davinci/vize_l2/src/query/vue.rs`, its unit tests, and
  `davinci/vize_l2/tests/query_vue_corpus.rs`. On 1,476 corpus setups, 11,563 rows are answered once
  each. 2,918 rows have no File declaration yet: Props 2,299, SetupMaybeRef 595, PropsAliased 19,
  LiteralConst 4, SetupConst 1 (destructured, enum and props-destructure locals). The storage
  policy test and vize_l2 clippy pass.

## Next steps

1. After #13379 merges: `git rebase origin/main` on each branch (Q2 drops out), push, and open
   Q6 as `test(l2): forbid product imports of L2 internals (#6871)`. Open Q3 after Q6 merges.
2. Measurements, using the #13379 format:
   - Q6: interleaved `level-instruction-counts.yml` base/head runs plus twins, and the PR
     observation. The only Rust change is an import path, so expect identical rows and a
     same-size release binary.
   - Q3: the same, plus a release-profile `vize` size check under
     `/private/tmp/claude-501/vize-release-build.lock`. `Model` gains two `Option<&_>` fields,
     which the `NativeSfc` call path constructs.
   - Only dispatch while `gh run list --status queued | wc -l` ≤ 40.
3. Remove this file in the Q3 PR before enabling auto-merge.
