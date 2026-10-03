<!-- GENERATED FILE — do not edit by hand.
     Regenerate: rust-script tools/commands/davinci/croquis-consumers.rs --write
     Verify:     rust-script tools/commands/davinci/croquis-consumers.rs --check -->

# Croquis consumption: `vize_atelier_dom`

One shard of the [Croquis consumption matrix](../croquis-consumption.md): resolved consumption and non-product imports in `crates/vize_atelier_dom/src`. The method and product set live on that page; fresh naive-grep diagnostics are printed by `--check` and `--summary`.

## Resolved product sites

| product              | kind  | module    | files | sites |
| -------------------- | ----- | --------- | ----: | ----: |
| `Croquis`            | type  | `croquis` |     4 |     8 |
| `Croquis.bindings`   | field | `croquis` |     1 |     2 |
| `Croquis.reactivity` | field | `croquis` |     1 |     4 |

## Non-product `vize_croquis` imports

| item                    | files | sites |
| ----------------------- | ----: | ----: |
| `Bindings`              |     1 |     2 |
| `CroquisFacts`          |     1 |     1 |
| `Demand`                |     1 |     2 |
| `FactConsumer`          |     1 |     1 |
| `ReactiveKind`          |     1 |     4 |
| `reactivity_sources`    |     1 |     1 |
| `used_components_empty` |     1 |     1 |
