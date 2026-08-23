# Typed editor oracle matrix

This matrix is the external-behavior ledger for #4585. It keeps the typed editor
surface honest without turning #4585 into an umbrella implementation PR.

| Slice                                                                                                                              | Status    | Evidence                                                          | Follow-up    |
| ---------------------------------------------------------------------------------------------------------------------------------- | --------- | ----------------------------------------------------------------- | ------------ |
| CLI authored script diagnostics report `const count: string = 0` at the authored script span.                                      | Covered   | `crates/vize/tests/check_text_diagnostics_cli.rs`                 | #4587        |
| LSP authored script diagnostics report and clear `const a: string = 1`.                                                            | Covered   | `tests/tooling/lsp-authored-script-diagnostics.test.ts`           | #4587        |
| CLI fallthrough diagnostics use authored template ranges, not the leading script token.                                            | Covered   | `crates/vize_canon/src/sfc_typecheck/tests/fallthrough_ranges.rs` | #4586        |
| LSP fallthrough diagnostics publish and clear on authored template usage.                                                          | Covered   | `tests/tooling/lsp-fallthrough-attrs.test.ts`                     | #4586        |
| LSP hover and definition cover script bindings, template identifiers, component tags, and slot names.                              | Covered   | `tests/tooling/lsp-hover-type-backed.test.ts`                     | #4588, #4592 |
| Vue JSX intrinsic globals suppress TS7026 while component props stay strict.                                                       | Covered   | `tests/snapshots/check/jsx-intrinsic-globals-oracle.ts`           | #4590        |
| Packaged VS Code host smoke rejects empty hover and `Ref<unknown>`/`ComputedRef<unknown>`/`MaybeRef<unknown>` for reactive values. | Covered   | `editors/vscode/test/suite/real-scenario.cjs`                     | #4589        |
| Imported SFC hovers show props, emits, slots, model, and marker-free component contracts in LSP script surfaces.                   | Known gap | pending PR #4608                                                  | #4591        |
| Component `v-model` template navigation resolves to child `defineModel` declarations.                                              | Known gap | pending PR #4607                                                  | #4592        |
| Non-VSCode editor-host smoke rejects reactive hover degradation.                                                                   | Known gap | pending PR #4609                                                  | #4589        |
