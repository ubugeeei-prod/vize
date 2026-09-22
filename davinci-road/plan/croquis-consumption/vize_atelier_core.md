<!-- GENERATED FILE — do not edit by hand.
     Regenerate: rust-script tools/commands/davinci/croquis-consumers.rs --write
     Verify:     rust-script tools/commands/davinci/croquis-consumers.rs --check -->

# Croquis consumption: `vize_atelier_core`

One shard of the [Croquis consumption matrix](../croquis-consumption.md): every fact the matrix records about `crates/vize_atelier_core/src`. The method and the product set live on that page.

## Resolved product sites

| product            | kind  | module    | files | sites |
| ------------------ | ----- | --------- | ----: | ----: |
| `Croquis`          | type  | `croquis` |     3 |    17 |
| `ScopeBinding`     | type  | `scope`   |     1 |     1 |
| `ScopeChain`       | type  | `scope`   |     2 |     2 |
| `ScopeKind`        | type  | `scope`   |     2 |     2 |
| `VForScopeData`    | type  | `scope`   |     1 |     1 |
| `VSlotScopeData`   | type  | `scope`   |     1 |     1 |
| `Croquis.bindings` | field | `croquis` |     1 |     2 |

## Non-product `vize_croquis` imports

| item                | files | sites |
| ------------------- | ----: | ----: |
| `BindingType`       |     1 |    11 |
| `ReactiveKind`      |     1 |     4 |
| `is_event_local`    |     1 |     1 |
| `is_global_allowed` |     4 |    10 |

## Naive grep disagreements (resolved/grep)

| product              | resolved | grep |
| -------------------- | -------: | ---: |
| `BindingMetadata`    |        0 |   24 |
| `Croquis`            |       17 |   25 |
| `ReactivityTracker`  |        0 |    1 |
| `Scope`              |        0 |    4 |
| `ScopeBinding`       |        1 |    2 |
| `ScopeChain`         |        2 |    3 |
| `ScopeKind`          |        2 |    3 |
| `Span`               |        0 |   42 |
| `Symbol`             |        0 |    5 |
| `VForScopeData`      |        1 |    2 |
| `VSlotScopeData`     |        1 |    2 |
| `Croquis.bindings`   |        2 |   35 |
| `Croquis.hoists`     |        0 |    8 |
| `Croquis.reactivity` |        0 |    1 |
| `Croquis.scopes`     |        0 |    4 |
