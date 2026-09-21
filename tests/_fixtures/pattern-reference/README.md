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

`pattern-selection-reference.test.ts` runs the selection-semantics half of
`compiler-core/__tests__/patterns.spec.ts`, its match table and all nine scenarios,
through a render function compiled by the real CLI: every arm reports its index and
bindings from a rendered interpolation. The grammar half of that spec is covered by
the shared parser's unit tests in `crates/vize_armature/src/patterns/tests.rs`.

`pattern-runtime-reference.test.ts` runs every case of `vue/__tests__/vMatch.spec.ts`
against Vize's client and server output with the installed Vue runtime: attribute
fallthrough through `Transition` and `KeepAlive`, keyed arms, client and server
parity, and hydration across reactive arm changes.

`pattern-vapor-parity-reference.test.ts` runs every case of
`runtime-vapor/__tests__/vMatch.spec.ts` on both renderers from one reactive source.
One scenario, remounting an explicitly keyed `<template>` arm, runs on the VDOM
renderer only: Vapor has no keyed fragment yet.

`pattern-grammar-reference.test.ts` reads the reference's TextMate snapshot for both
patterned fixtures and compares every character of every `v-match` / `v-when`
attribute with the grammar the VS Code extension ships. The two grammars name the
attribute shell and the embedded guard expression differently, so those are compared
by role; the scopes inside a pattern must be identical.

The remaining compiler snapshot, SFC, Vapor hydration, and `Transition` e2e
reference cases are tracked by [#6176](https://github.com/ubugeeei-prod/vize/issues/6176).
Preserving those source files is an inventory check, not evidence that those suites
have all executed.
