# TypeScript dependency gates

Paired decisions: [#6828](https://github.com/ubugeeei-prod/vize/issues/6828#issuecomment-6057680567)
and [#6830](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-6057681954).
This is a bounded child of the
[owned TypeScript migration](./2026-10-08-owned-typescript-migration.md).

## Decision and ownership

Move `require-needs-success.mjs` and `require-rust-tier.mjs` to erasable
TypeScript, then migrate every live import and executable caller of these two
providers. Move-only commit `68503bfb7bbc1bca09b45e81a75dc85b2f650a52`
contains exactly two 100% byte-identical renames above the original first-slice
head `0bb7d231e70c318240337f35c74957488adde735`. Dependent delivery must retain
that move/type separation when replayed on the actual predecessor. After genuine
replay, move-only commit `fd4fd9b763` is directly above PR #8304 predecessor
`cbdad9a837b2ca1d755a6e371b3c94bca4e1ce35`; both renames remain 100%.

The providers accept unknown external event, plan and JSON values. Object
guards and records with unknown values preserve the original success and
refusal boundaries. The Rust no-crates return still precedes needs validation.
PR and merge-group required job sets, allowed skips, sorting, exit codes and
whole output text remain unchanged. The missing-`NEEDS_JSON` message still
contains its original `.mjs` spelling because it is part of that output.

The separate `tsconfig.ci-gates.json` owns only the two providers and
`typescript-ci-gates.test.ts`. It inherits strict, no-emit, erasable syntax from
`tsconfig.node.json` and runs through the existing native TypeScript 7.0.2
checker. `check:repo` appends a separate invocation of this project; the first
migration project and its default invocation retain their exact scope.

## Runtime and consumer graph

Check's aggregate report and the reusable Rust source report execute the actual
`.ts` entrypoints. The Rust report moves its existing pinned `setup-vp` before
the gate and runs that setup unconditionally, including PR, no-Rust and failed
Rust cases. It introduces no Action, job or transpilation stage. All subsequent worker selection, archive identity, receipt and queue
conditions retain their original authority.

The existing Check Node 22/24 matrix adds the new gate test to its current
command. Original importer coverage remains in these files:

- `github-workflows-check-gate.test.ts`
- `github-workflows-contracts-gate.test.ts`
- `github-workflows-js-tiers.test.mjs`
- `github-workflows-merge-queue.test.ts`
- `github-workflows-source-selection.test.ts`
- `level-instruction-workflow.test.ts`
- `lsp-native-navigation-workflow.test.ts`
- `release/release-integration-catalog.test.ts`
- `rust-test-archive.test.mjs`
- `davinci-canonical-corpus-workflow.test.ts`

The canonical selector imports the real TypeScript gate and adds explicit
literals for both new provider paths. Its original `.mjs` selector remains
byte-identical, preserving historical deletion and rename selection. Original
test assertions and corpus inputs remain; runtime ordering and both new path
controls add assertions rather than replacing old cases.

## Proof and limits

The independently authored `fixtures/ci-gates/packets.json` was frozen before
new gate execution: SHA-256
`9812ae543092e989bdab6ffe05ebb089b272a5da57adfff03f43e861deac635d`.
It retains 24 whole JSON exported-function result or refusal packets; two
independently authored primitive/boxed Symbol controls retain the original
string-coercion refusal and prevent lint-warning suppression. Six real Node
subprocess cases retain complete status/stdout/stderr for success, red reports,
missing input and the Rust early return. Native compiler controls separately
require a valid typed witness to pass and type mismatch, implicit parameter and
non-erasable enum witnesses to fail. This does not freeze dynamic stack text or
claim every possible CLI malformed-input packet.

Actual Node **22.18.0** and **24.14.0** each passed all **42 tests** in the
11-file run: 39 original tests and 3 new tests, with zero skipped tests. This
proves the existing MJS importers can consume the real TypeScript providers;
it does not raise the public Node 22 floor or qualify stripping on Node 22.0.
The focused three-file native project and formatting checks pass locally.
Whole observations, the pre-execution source ledger and native refusal output
are retained under `target/typescript-ci-gates/` before assertions. The initial
local runs each retained one unchanged Fresco test failure because this new
worktree lacked its workspace-local TypeScript dependency. Linking the existing
read-only Fresco dependency installation resolved it; both raw failed runs are
retained and no manifest, lock, test or assertion was weakened.

The JS-tier and Rust archive providers and their original `.mjs` test sources,
and the canonical selector's own provider graph, remain unfinished migration
work. Their gate imports are updated and exercised, but they are outside the
strict three-file project. Migrate each provider with its full caller graph in
a later bounded child; do not add `allowJs`, declaration stubs, compiler
suppression or a wider implicit project to grant them TypeScript credit.

Exact-head Actions, genuine native Stack membership, protected queue admission
and actual merge remain separate delivery requirements. Local proof is not
terminal delivery, runtime acceptance of unrelated product slices, or a
performance improvement claim. Root owns queue admission and tracks the true
ordered predecessor after its source checks pass.
