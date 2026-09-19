# Pinned patterned-template references

The upstream directory preserves all test and snapshot files changed by the
patterned-template reference implementations, with SHA-256 hashes and MIT notices:

- [vuejs/core #15531](https://github.com/vuejs/core/pull/15531), `53455195068f6916c9864a2490e017b584900874`.
- [vuejs/language-tools #6207](https://github.com/vuejs/language-tools/pull/6207), `e68d41bfef1770f8428f92097abe959b57de62d2`.

`canon-pattern-reference.test.ts` evaluates the pinned spec's fixture-construction
section to run all 140 source files through the real CLI. This includes all 40
coverage cases in nested, top-level, and top-level `lang="html"` forms. Authored
imports and negative directives are retained. Removing all ten template
expected-error comments must expose exactly their ten original diagnostics. The two click assertions use the
installed Vue `HTMLAttributes.onClick` argument type: the reference's hardcoded
`MouseEvent` predates Vue's `PointerEvent` update. The pinned source is unchanged.
`shared.d.ts` is the upstream `exactType` helper, under the included language-tools
MIT license.

`canon-pattern-types.test.ts` runs every upstream helper assertion against Canon,
including removal of both expected-error comments to prove the assertions fail.
Only the upstream namespace/import is adapted.

`lsp-pattern-reference.test.ts` runs all scenarios from the reference's patterned
LSP suite, plus its directive completion case, over production stdio. The literal
fixtures are read from the pinned spec's AST. Rename checks additionally apply
the edits and check the result: expanding `{ const value }` to
`{ value: const renamed }` preserves the original property key.

The remaining compiler, runtime/hydration, and TextMate reference cases are tracked
by [#6176](https://github.com/ubugeeei-prod/vize/issues/6176). Preserving those source
files is an inventory check, not evidence that those suites have all executed.
