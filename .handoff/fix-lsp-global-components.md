# Handoff: LSP regressions #13384 and #13385 (after #13383)

Status on 2026-11-19 (JST evening): #13383 is fixed on `fix/lsp-art-imported-hover`. The two issues below are reproduced but not fixed.

## #13384: editor ignores project GlobalComponents declarations

Repro specs (stdio, both publish `[]`):
- `.handoff/global-13384.spec.json` (this branch)
- Run with `node tools/commands/fixtures/lsp-fix-history-case.ts print <spec>` after `cargo build --locked -p vize --profile ci`.

Root cause, traced by reading the code (not yet confirmed by experiment):
- Diagnostics open the host through `open_vue_virtual_document_with_borrowed_overlays_and_options` with `editor_virtual_ts_options()`.
  - `reference_paths` holds every workspace `.d.ts` file (`server/state/global_components.rs::collect_workspace_declarations`).
  - The host projection therefore gets `/// <reference path="/abs/src/components.d.ts" />`.
- The editor mirror (`crates/vize_canon/src/corsa_bridge/vue_dependencies_alias/context/build.rs`) registers only the host and its reachable dependencies.
  - `set_declaration_roots(&[source_path])` applies.
  - `editor_source_facts.rs::walked_references` deliberately skips the projection's synthetic references ("would pull in the whole project").
- So `components.d.ts` is read from disk outside the mirror. Its `import('./components/ElBadge.vue')` / `import GlobalBadge from './GlobalBadge.vue'` targets are never generated, the component types are `any`, and no slot or prop diagnostics follow.
- The batch checker works because its program includes `components.d.ts` through the tsconfig and materializes its `.vue` imports.

Likely fix (canon editor mirror):
1. Walk the host's synthetic reference paths, but only for declaration files that augment `GlobalComponents`.
2. Filtering keeps the walk bounded. `collect_workspace_declarations` returns every `.d.ts`, so filter in maestro (`editor_virtual_ts_options`) or in the walk.
3. Register the selected files in the mirror so their relative `.vue` imports land on generated mirror files.
4. Make the host reference point at the mirrored copy.

Watch the editor perf rows: this widens the per-open graph.

## #13385: resolved, not a regression

The editor publishes the authored `component-required-props` diagnostic for the missing prop. The duplicate TS2345 is folded into it on purpose (`ide/diagnostics/native.rs`). The session is pinned in #13413 (`lsp/fix-history/native-root-required-props-13385`), and row 09b4ddde76 is covered there. Close #13385 when #13413 merges.

## Ledger rows

`docs/davinci/plan/lsp-fix-history.tsv` still has 3 `regression-held` rows once #13413 merges: 22a49aca2f, 37d5c18c2a and a0e0efd776 (all #13384). After a fix, register the specs with `lsp-fix-history-case.ts register`, which flips the rows. Then close #6883 once all rows are covered.

## Related in-flight runs (#13413, not this branch)

- Base 44517750227 (`bench/lsp-13383-base`, main 8b09dea8b1) and head 44517582455 for #13413. See `.handoff/fix-lsp-art-imported-hover.md` on that branch.
- This branch has no dispatches. Its next step is the #13384 fix described above.
