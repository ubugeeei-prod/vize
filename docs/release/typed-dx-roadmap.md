# Typed DX production roadmap

This is the P0 execution lane for #3957 and #4585. Typed authoring is not
production-ready until every row below has an external behavior oracle and the
implementation passes that oracle in CLI, LSP, and packaged editor contexts
where applicable.

Internal virtual-code snapshots are useful debugging evidence, but they do not
complete a row. Every row must prove the authored user-facing behavior.

## Current evidence

`docs/release/typed-editor-oracle-matrix.md` is the machine-checked external
behavior ledger for this roadmap. Keep this file focused on required invariants
and delivery policy; record per-slice status, evidence paths, and CI wiring in
the matrix.

## P0 matrix

| issue | invariant                                                                                           | required oracle                                                                                                                                              | first fix target                     |
| ----- | --------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------ |
| #4586 | Multi-root fallthrough diagnostics point at authored template ranges, never the first script token. | CLI `vize check` and LSP `publishDiagnostics` assert the same authored template range, with a single-root control.                                           | Diagnostic source mapping            |
| #4587 | Authored script TypeScript diagnostics are never swallowed.                                         | `const a: string = 1` reports in CLI and LSP, then clears on repair with monotonic versions.                                                                 | Checker diagnostic ownership         |
| #4588 | Hover is non-empty and checker-backed on known typed anchors.                                       | Script binding, template binding, prop, emit, slot, ref, and component import hover all return useful typed content.                                         | Hover provenance and request routing |
| #4589 | Editor integrations do not degrade common typed values to `Ref<unknown>`.                           | Packaged editor-host smoke proves precise `ref`, computed, template-ref, props, emits, slots, and component value types.                                     | Packaged editor type surface         |
| #4590 | Supported Vue JSX/TSX entrypoints have JSX intrinsic globals and component typing.                  | CLI and LSP JSX/TSX diagnostics reject invalid props but never emit TS7026 for supported Vue JSX.                                                            | JSX global and component type setup  |
| #4591 | Imported SFCs show useful component contracts instead of internal marker carriers.                  | Hover/display includes props, emits, slots, model, and declaration anchors without dominant `__vizeComponentMarker` leakage.                                 | Component contract display           |
| #4592 | Template hover and navigation are reliable across typed component surfaces.                         | Template identifiers, component tags, props, emits, slots, `v-model`, refs, `v-for`/`v-if`, aliases, barrels, and packages have authored hover/jump targets. | Template semantic links              |

## Delivery order

1. Add or extend the external behavior oracle for one row.
2. Make the oracle fail for the current behavior or document the current failure
   in a reviewed expected-failure ledger that names the P0 issue.
3. Fix exactly that invariant.
4. Promote the oracle into the relevant CI gate.
5. Update the external behavior ledger only with machine evidence.

## PR policy

- One P0 invariant per PR.
- No umbrella implementation PRs.
- Conventional titles.
- PR descriptions must be created from body files and raw-checked for literal
  newline escape corruption.
- Auto-merge only after required checks and review are green.
