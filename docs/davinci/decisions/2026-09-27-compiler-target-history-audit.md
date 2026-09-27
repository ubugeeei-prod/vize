# SSR and Vapor history witness audit

Read-only source audit for [#6880](https://github.com/ubugeeei-prod/vize/issues/6880),
pinned to main `9aaa1fe458a09e0d0c6604dc8835ccf7c737d943`. No product build was
performed. These classifications identify what existing assertions protect;
they are not a completed semantic history census or execution certificates.

## Snapshot and byte boundaries

Insta 1.48.0 text snapshots call `trim_end()` and normalize CRLF to LF. SSR
snapshot tests compare code after that normalization, often without separate
preamble/map values. Vapor's `src/tests.rs` and `src/tests_slot_outlets.rs`
additionally trim every line and remove blank lines before snapshotting.
`src/tests_insertion_state.rs` removes imports and indentation from its direct
body comparisons. None of those assertions alone is a fixed raw module-byte
reference. Keep every shape/runtime witness while adding raw output pins.

Native/retained equality compares two current implementations. It catches
divergence but cannot protect a fixed historical output if both change. Runtime
traces protect behavior; they do not pin compiler output bytes.

## SSR reviewed families

Nineteen SSR fixes were inspected. Existing strict string references include
`tests/component_spread_props.rs` (six code outputs for #3737),
`tests/vue_ssr_render.rs::aligned_ssr_shapes_compile_to_the_pinned_output`
(16 preamble/code modules for #6352), and SFC `tests/ssr_slot_scope.rs`
(12 whole modules for #6903). These references have distinct entrypoints,
options and payload boundaries. Do not extrapolate their coverage to other
inputs, preambles/maps omitted by code-only tests, or the historical issue's
unreconciled snapshot denominator.

Three immediate raw-output fixture slices retain the original default
`compile_ssr` options, with no synthetic outer wrapper:

| Fix               | Existing witness                                                                                                                                              | Missing fixed payload                                                                   |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| #990 `f5aa652fc`  | `src/lib.rs::test_ssr_v_model_select_marks_matching_option_selected`, `<select v-model="x"><option value="a">A</option><option value="b">B</option></select>` | Whole preamble/code; existing helper and selected-attribute substrings are partial      |
| #3701 `619614e63` | `tests/event_props.rs`, model/listener in both orders, modifiers, object spread and dynamic-key boundaries (five inputs)                                      | Whole preamble/code; current counts and substrings are partial                          |
| #2487 `dcd679b5d` | `src/lib.rs::test_ssr_keyed_template_v_for_*`, three keyed iteration/fallback inputs                                                                          | Whole preamble/code in push and vnode branches; current Fragment substrings are partial |

The #2487 typed-slot witness additionally uses `is_ts=true`; do not apply its
options to the three default-option keyed-loop cases. #6908 preserves public
option literal compatibility and compares two live outputs; it is useful
source compatibility evidence, not a fixed historical payload.

## Vapor reviewed families

Twenty distinct Vapor fix commits were inspected. No fixed raw full-module
byte witness was found for these particular families. Other direct literal
code assertions exist in `tests/davinci_vapor_artifact.rs`, but their event
fallback and tolerant-expression inputs cannot be substituted for these fixes.

The default raw-template options below mean `VaporCompilerOptions::default()`,
prefix/retained/SSR/inline false, no binding metadata, Standard syntax and
default custom-element/experimental options. Variants are stated explicitly.

| Fix family and commits                                                             | Existing input/witness                                                                                                                                  | Preserved coverage and missing pin                                                                                                                                   |
| ---------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Destructured aliases, keys, object index: `6812891dd`, `0de7787ae`, `0895f6324`    | `src/tests.rs::test_compile_v_for_destructured_aliases_resolve_source_paths`; `tests/fixtures/vapor/v-for.pkl` destructured-key and object-index inputs | Parser/substrings and authored inputs; add each complete raw code result                                                                                             |
| Named/nested/bare slot outlets: `b7a657e12`, `d90df360f`, `e42bcb923`, `93da7a4d4` | Named fallback, nested outlet, bare/named/props outlet tests in `src/tests.rs` and `src/tests_slot_outlets.rs`; `v-slot.pkl`                            | Normalized snapshots and module syntax; preserve imports/indentation/blank lines in new raw references                                                               |
| Directives/once/memo: `faa404cc0`, `866be7b96`                                     | `v-focus:[placement].lazy`, `v-cloak`, `v-once`, `v-memo="[]"` in `src/tests.rs`                                                                        | Normalized code and exact unsupported-memo diagnostics; add supported raw code results                                                                               |
| Delegated events: `3b0400f76`                                                      | `tests/nested_delegate_events.rs` if/loop/slot fallback inputs, prefix=true                                                                             | Exact extracted `_delegateEvents` lines and mounted acceptance; add imports and every other module byte                                                              |
| Dynamic events/models: `be7c77a8e`, `21c1b961c`, `8c9e4642d`                       | SFC `tests/vapor_runtime_contracts/{events,models}.rs`; mounted model prefix false/true tests                                                           | Runtime traces and normalized model snapshots; add corresponding full modules with the original distinct options                                                     |
| Mounted branch/fallback: `1679959a3`                                               | `davinci_mounted_behavior::mounted_branch_and_slot_fallback_update_together`                                                                            | Fixed Lean behavior JSON and runtime trace; add compiler output without changing behavior tests                                                                      |
| Hydration/insertion: `dfea03941`, `8a6c88b34`, `af1079858`                         | Root ordering, sibling navigation and insertion-state cases; retained=false/true variants where present                                                 | Creation/helper shapes and normalized bodies; add complete modules for each original variant                                                                         |
| KeepAlive: `339f29e3c`                                                             | Source-map and `davinci_keep_alive_parity` cases, prefix/source-map=true, filename `Foo.vue`                                                            | Current lane equality, normalized code/map, fixed lifecycle/identity traces; add historical raw result pins                                                          |
| Structural slots: `9e10e2496`, `5911c74b9`                                         | `src/l3/tests/structural_slots.rs`, `tests_source_map/structural_slots.rs`                                                                              | Current lane equality, admission rejection, maps and branch/reorder/teardown traces; add supported raw modules without treating rejected shapes as native acceptance |

The SFC runtime-contract helper uses `vapor=true`, scope `probe`, Standard
syntax, default matcher/codegen and `SeparateTemplate` output. Preserve that
entrypoint and all options; a raw-template default fixture is a different case.

Capture actual complete results, record source/artifact identity, compare a
repeat, then freeze expectations. Prioritize aliases/keys/index, delegated
if/loop/fallback events, and named/nested/directive slot cases as separate small
PRs. Hydration, KeepAlive and structural-slot references follow with their
existing behavior acceptance intact. Whole-history/target/dialect review and
fresh Actions remain required; no unreviewed fix or native compiler is certified.
