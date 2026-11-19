<!-- GENERATED FILE — do not edit by hand.
     Regenerate: node tools/support/compat/levels/croquis-consumers.ts --write
     Verify:     node tools/support/compat/levels/croquis-consumers.ts --check -->

# Croquis consumption: `vize_atelier_core`

One shard of the [Croquis consumption matrix](../croquis-consumption.md): resolved consumption and non-product imports in `crates/vize_atelier_core/src`. The method and product set live on that page; fresh naive-grep diagnostics are printed by `--check` and `--summary`.

## Resolved product sites

| product              | kind  | module    | files | sites |
| -------------------- | ----- | --------- | ----: | ----: |
| `Croquis`            | type  | `croquis` |     1 |     2 |
| `ScopeBinding`       | type  | `scope`   |     1 |     1 |
| `ScopeChain`         | type  | `scope`   |     2 |     2 |
| `ScopeKind`          | type  | `scope`   |     2 |     2 |
| `VForScopeData`      | type  | `scope`   |     1 |     1 |
| `VSlotScopeData`     | type  | `scope`   |     1 |     1 |
| `Croquis.bindings`   | field | `croquis` |     5 |    11 |
| `Croquis.reactivity` | field | `croquis` |     1 |     4 |

## Non-product `vize_croquis` imports

| item                       | files | sites |
| -------------------------- | ----: | ----: |
| `BindingType`              |     2 |    20 |
| `ReactiveKind`             |     1 |    21 |
| `is_global_allowed`        |     1 |     6 |
| `used_component_name_list` |     1 |     1 |
