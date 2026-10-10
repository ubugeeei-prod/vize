<!-- GENERATED FILE — do not edit by hand.
     Regenerate: rust-script tools/commands/davinci/croquis-consumers.rs --write
     Verify:     rust-script tools/commands/davinci/croquis-consumers.rs --check -->

# Croquis consumption: `vize_atelier_core`

One shard of the [Croquis consumption matrix](../croquis-consumption.md): resolved consumption and non-product imports in `crates/vize_atelier_core/src`. The method and product set live on that page; fresh naive-grep diagnostics are printed by `--check` and `--summary`.

## Resolved product sites

| product            | kind  | module    | files | sites |
| ------------------ | ----- | --------- | ----: | ----: |
| `Croquis`          | type  | `croquis` |     4 |    18 |
| `ScopeBinding`     | type  | `scope`   |     1 |     1 |
| `ScopeChain`       | type  | `scope`   |     2 |     2 |
| `ScopeKind`        | type  | `scope`   |     2 |     2 |
| `VForScopeData`    | type  | `scope`   |     1 |     1 |
| `VSlotScopeData`   | type  | `scope`   |     1 |     1 |
| `Croquis.bindings` | field | `croquis` |     4 |    10 |

## Non-product `vize_croquis` imports

| item                | files | sites |
| ------------------- | ----: | ----: |
| `BindingType`       |     1 |    11 |
| `ReactiveKind`      |     1 |     4 |
| `is_event_local`    |     1 |     1 |
| `is_global_allowed` |     4 |    10 |
