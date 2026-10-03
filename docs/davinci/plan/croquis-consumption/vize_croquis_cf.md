<!-- GENERATED FILE — do not edit by hand.
     Regenerate: rust-script tools/commands/davinci/croquis-consumers.rs --write
     Verify:     rust-script tools/commands/davinci/croquis-consumers.rs --check -->

# Croquis consumption: `vize_croquis_cf`

One shard of the [Croquis consumption matrix](../croquis-consumption.md): resolved consumption and non-product imports in `crates/vize_croquis_cf/src`. The method and product set live on that page; fresh naive-grep diagnostics are printed by `--check` and `--summary`.

## Resolved product sites

| product                                | kind  | module              | files | sites |
| -------------------------------------- | ----- | ------------------- | ----: | ----: |
| `Analyzer`                             | type  | `analyzer`          |    21 |    42 |
| `AnalyzerOptions`                      | type  | `analyzer`          |    15 |    21 |
| `ComponentUsage`                       | type  | `croquis::template` |    14 |    38 |
| `Croquis`                              | type  | `croquis`           |    42 |    97 |
| `EffectGraphScript`                    | type  | `effect_graph`      |     1 |     1 |
| `EffectGraphSummary`                   | type  | `effect_graph`      |     9 |    19 |
| `ElementIdKind`                        | type  | `croquis::template` |     1 |     3 |
| `EventHandlerScopeData`                | type  | `scope`             |     1 |     1 |
| `EventListener`                        | type  | `croquis::template` |     6 |    10 |
| `PassedProp`                           | type  | `croquis::template` |     8 |    17 |
| `ScopeData`                            | type  | `scope`             |     8 |     8 |
| `ScopeId`                              | type  | `scope`             |    15 |    21 |
| `ScopeKind`                            | type  | `scope`             |     7 |     8 |
| `SlotUsage`                            | type  | `croquis::template` |     2 |     2 |
| `TemplateExpression`                   | type  | `croquis`           |     1 |     1 |
| `TemplateExpressionKind`               | type  | `croquis`           |     1 |     1 |
| `build_effect_graph_from_script`       | type  | `effect_graph`      |     1 |     2 |
| `build_effect_graph_from_script_setup` | type  | `effect_graph`      |     1 |     1 |
| `build_effect_graph_from_sfc_scripts`  | type  | `effect_graph`      |     1 |     1 |
| `note_mounted_resources`               | type  | `mounted`           |     1 |     1 |
| `Croquis.element_ids`                  | field | `croquis`           |     1 |     1 |
| `Croquis.invalid_exports`              | field | `croquis`           |     1 |     2 |
| `Croquis.macros`                       | field | `croquis`           |    14 |    25 |
| `Croquis.re_export_forwards`           | field | `croquis`           |     1 |     1 |
| `Croquis.reactivity`                   | field | `croquis`           |     2 |     2 |
| `Croquis.scopes`                       | field | `croquis`           |    12 |    16 |
| `Croquis.setup_context`                | field | `croquis`           |     2 |     3 |
| `Croquis.template_expressions`         | field | `croquis`           |     2 |     4 |
| `Croquis.template_info`                | field | `croquis`           |     5 |    14 |
| `Croquis.type_exports`                 | field | `croquis`           |     1 |     1 |
| `Croquis.types`                        | field | `croquis`           |     1 |     1 |

## Non-product `vize_croquis` imports

| item                         | files | sites |
| ---------------------------- | ----: | ----: |
| `Agreement`                  |     1 |     1 |
| `Bindings`                   |     3 |     4 |
| `CroquisFacts`               |     2 |     2 |
| `Demand`                     |     1 |     6 |
| `EmitDefinition`             |     3 |     3 |
| `FactConsumer`               |     1 |     4 |
| `FactGroup`                  |     1 |     1 |
| `InjectEntry`                |     4 |     8 |
| `InjectPattern`              |     3 |    13 |
| `MacroKind`                  |     1 |     1 |
| `PropDefinition`             |     3 |     3 |
| `ProvideEntry`               |     2 |     7 |
| `ProvideKey`                 |     9 |    52 |
| `RaceConditionRisk`          |     2 |     4 |
| `RaceConditionRiskKind`      |     2 |     3 |
| `ReactiveKind`               |     5 |    33 |
| `ReactivityLossKind`         |     2 |    18 |
| `SetupContextViolation`      |     1 |     1 |
| `SetupContextViolationKind`  |     2 |    11 |
| `SfcParseOptions`            |     2 |     2 |
| `SourceFact`                 |     1 |     1 |
| `TypeDefinitions`            |     2 |     3 |
| `ViolationSeverity`          |     1 |     6 |
| `component_identity`         |     1 |     1 |
| `component_usage_list`       |     9 |    14 |
| `composable_calls`           |     2 |     2 |
| `inject_entries`             |     6 |    16 |
| `parse_sfc`                  |     1 |     1 |
| `parse_sfc_without_css_vars` |     1 |     1 |
| `provide_entries`            |     4 |     7 |
| `race_risks`                 |     1 |     1 |
| `reactivity_count`           |     3 |     3 |
| `reactivity_is_reactive`     |     1 |     5 |
| `reactivity_lookup`          |     2 |     2 |
| `reactivity_losses`          |     2 |     2 |
| `reactivity_sources`         |     2 |     2 |
| `to_pascal_case`             |     1 |     2 |
| `used_component_contains`    |     1 |     1 |
| `used_component_name_list`   |     5 |     7 |
