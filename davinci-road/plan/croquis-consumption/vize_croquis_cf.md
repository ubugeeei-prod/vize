<!-- GENERATED FILE — do not edit by hand.
     Regenerate: rust-script tools/commands/davinci/croquis-consumers.rs --write
     Verify:     rust-script tools/commands/davinci/croquis-consumers.rs --check -->

# Croquis consumption: `vize_croquis_cf`

One shard of the [Croquis consumption matrix](../croquis-consumption.md): every fact the matrix records about `crates/vize_croquis_cf/src`. The method and the product set live on that page.

## Resolved product sites

| product                                | kind  | module              | files | sites |
| -------------------------------------- | ----- | ------------------- | ----: | ----: |
| `Analyzer`                             | type  | `analyzer`          |    19 |    41 |
| `AnalyzerOptions`                      | type  | `analyzer`          |    13 |    20 |
| `ComponentUsage`                       | type  | `croquis::template` |    12 |    36 |
| `Croquis`                              | type  | `croquis`           |    40 |    95 |
| `EffectGraphScript`                    | type  | `effect_graph`      |     1 |     1 |
| `EffectGraphSummary`                   | type  | `effect_graph`      |     9 |    19 |
| `ElementIdKind`                        | type  | `croquis::template` |     1 |     3 |
| `EventHandlerScopeData`                | type  | `scope`             |     1 |     1 |
| `EventListener`                        | type  | `croquis::template` |     6 |    10 |
| `PassedProp`                           | type  | `croquis::template` |     8 |    17 |
| `ScopeData`                            | type  | `scope`             |     9 |     9 |
| `ScopeId`                              | type  | `scope`             |    12 |    18 |
| `ScopeKind`                            | type  | `scope`             |     7 |     9 |
| `SlotUsage`                            | type  | `croquis::template` |     2 |     2 |
| `build_effect_graph_from_script`       | type  | `effect_graph`      |     1 |     2 |
| `build_effect_graph_from_script_setup` | type  | `effect_graph`      |     1 |     1 |
| `build_effect_graph_from_sfc_scripts`  | type  | `effect_graph`      |     1 |     1 |
| `Croquis.component_usages`             | field | `croquis`           |    17 |    30 |
| `Croquis.element_ids`                  | field | `croquis`           |     1 |     1 |
| `Croquis.invalid_exports`              | field | `croquis`           |     1 |     2 |
| `Croquis.macros`                       | field | `croquis`           |    14 |    25 |
| `Croquis.provide_inject`               | field | `croquis`           |    10 |    26 |
| `Croquis.race_conditions`              | field | `croquis`           |     1 |     1 |
| `Croquis.reactivity`                   | field | `croquis`           |    11 |    22 |
| `Croquis.scopes`                       | field | `croquis`           |    13 |    18 |
| `Croquis.setup_context`                | field | `croquis`           |     1 |     1 |
| `Croquis.template_expressions`         | field | `croquis`           |     1 |     3 |
| `Croquis.template_info`                | field | `croquis`           |     5 |    14 |
| `Croquis.type_exports`                 | field | `croquis`           |     1 |     1 |
| `Croquis.used_components`              | field | `croquis`           |    16 |    30 |

## Non-product `vize_croquis` imports

| item                         | files | sites |
| ---------------------------- | ----: | ----: |
| `Bindings`                   |     3 |     4 |
| `CroquisFacts`               |     2 |     2 |
| `Demand`                     |     1 |     4 |
| `EmitDefinition`             |     3 |     3 |
| `FactConsumer`               |     1 |     2 |
| `InjectEntry`                |     4 |     8 |
| `InjectPattern`              |     4 |    21 |
| `MacroKind`                  |     1 |     1 |
| `PropDefinition`             |     3 |     3 |
| `ProvideEntry`               |     2 |     7 |
| `ProvideKey`                 |     7 |    34 |
| `RaceConditionRisk`          |     2 |     4 |
| `RaceConditionRiskKind`      |     2 |     3 |
| `ReactiveKind`               |     7 |    37 |
| `ReactiveSource`             |     1 |     1 |
| `ReactivityLossKind`         |     2 |    18 |
| `SetupContextViolation`      |     1 |     1 |
| `SetupContextViolationKind`  |     2 |    11 |
| `SfcParseOptions`            |     2 |     2 |
| `ViolationSeverity`          |     1 |     6 |
| `parse_sfc`                  |     1 |     1 |
| `parse_sfc_without_css_vars` |     1 |     1 |

## Naive grep disagreements (resolved/grep)

| product                                | resolved | grep |
| -------------------------------------- | -------: | ---: |
| `Analyzer`                             |       41 |   51 |
| `AnalyzerOptions`                      |       20 |   58 |
| `ComponentUsage`                       |       36 |   81 |
| `Croquis`                              |       95 |  122 |
| `EffectGraphScript`                    |        1 |    2 |
| `EffectGraphSummary`                   |       19 |   31 |
| `ElementIdKind`                        |        3 |    4 |
| `EventHandlerScopeData`                |        1 |    2 |
| `EventListener`                        |       10 |   16 |
| `MacroTracker`                         |        0 |    1 |
| `PassedProp`                           |       17 |   25 |
| `ProvideInjectTracker`                 |        0 |    1 |
| `ScopeId`                              |       18 |   27 |
| `ScopeKind`                            |        9 |   10 |
| `SlotUsage`                            |        2 |    7 |
| `Span`                                 |        0 |   36 |
| `Symbol`                               |        0 |   27 |
| `SymbolId`                             |        0 |    6 |
| `build_effect_graph_from_script`       |        2 |    3 |
| `build_effect_graph_from_script_setup` |        1 |    2 |
| `build_effect_graph_from_sfc_scripts`  |        1 |    2 |
| `Croquis.bindings`                     |        0 |    9 |
| `Croquis.component_usages`             |       30 |   35 |
| `Croquis.macros`                       |       25 |   27 |
| `Croquis.provide_inject`               |       26 |   34 |
| `Croquis.race_conditions`              |        1 |    4 |
| `Croquis.setup_context`                |        1 |    4 |
| `Croquis.template_info`                |       14 |   15 |
| `Croquis.used_components`              |       30 |   34 |
