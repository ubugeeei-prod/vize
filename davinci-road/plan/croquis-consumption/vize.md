<!-- GENERATED FILE — do not edit by hand.
     Regenerate: rust-script tools/commands/davinci/croquis-consumers.rs --write
     Verify:     rust-script tools/commands/davinci/croquis-consumers.rs --check -->

# Croquis consumption: `vize`

One shard of the [Croquis consumption matrix](../croquis-consumption.md): every fact the matrix records about `crates/vize/src`. The method and the product set live on that page.

## Resolved product sites

| product                               | kind  | module         | files | sites |
| ------------------------------------- | ----- | -------------- | ----: | ----: |
| `Analyzer`                            | type  | `analyzer`     |     1 |     1 |
| `AnalyzerOptions`                     | type  | `analyzer`     |     1 |     1 |
| `Croquis`                             | type  | `croquis`      |     1 |     1 |
| `EffectGraphScript`                   | type  | `effect_graph` |     1 |     2 |
| `build_effect_graph_from_sfc_scripts` | type  | `effect_graph` |     1 |     1 |
| `Croquis.component_usages`            | field | `croquis`      |     1 |     1 |

## Non-product `vize_croquis` imports

| item              | files | sites |
| ----------------- | ----: | ----: |
| `BlockLocation`   |     1 |     1 |
| `SfcDescriptor`   |     1 |     2 |
| `SfcParseOptions` |    11 |    14 |
| `to_pascal_case`  |     1 |     1 |

## Naive grep disagreements (resolved/grep)

| product                               | resolved | grep |
| ------------------------------------- | -------: | ---: |
| `Analyzer`                            |        1 |    2 |
| `AnalyzerOptions`                     |        1 |    2 |
| `Croquis`                             |        1 |    3 |
| `EffectGraphScript`                   |        2 |    3 |
| `Scope`                               |        0 |    1 |
| `build_effect_graph_from_sfc_scripts` |        1 |    2 |
| `Croquis.bindings`                    |        0 |    2 |
| `Croquis.scopes`                      |        0 |    1 |
| `Croquis.types`                       |        0 |    1 |
