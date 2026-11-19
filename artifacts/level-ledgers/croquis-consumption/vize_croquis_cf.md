<!-- GENERATED FILE — do not edit by hand.
     Regenerate: node tools/support/compat/levels/croquis-consumers.ts --write
     Verify:     node tools/support/compat/levels/croquis-consumers.ts --check -->

# Croquis consumption: `vize_croquis_cf`

One shard of the [Croquis consumption matrix](../croquis-consumption.md): resolved consumption and non-product imports in `crates/vize_croquis_cf/src`. The method and product set live on that page; fresh naive-grep diagnostics are printed by `--check` and `--summary`.

## Resolved product sites

| product                                | kind  | module              | files | sites |
| -------------------------------------- | ----- | ------------------- | ----: | ----: |
| `Analyzer`                             | type  | `analyzer`          |    25 |    48 |
| `AnalyzerOptions`                      | type  | `analyzer`          |    19 |    27 |
| `ComponentUsage`                       | type  | `croquis::template` |    19 |    44 |
| `Croquis`                              | type  | `croquis`           |    49 |   119 |
| `EffectGraphScript`                    | type  | `effect_graph`      |     2 |     2 |
| `EffectGraphSummary`                   | type  | `effect_graph`      |    10 |    21 |
| `ElementIdKind`                        | type  | `croquis::template` |     1 |     3 |
| `EventHandlerScopeData`                | type  | `scope`             |     1 |     1 |
| `EventListener`                        | type  | `croquis::template` |     6 |    10 |
| `NuxtPageLayout`                       | type  | `croquis::template` |     1 |     2 |
| `PassedProp`                           | type  | `croquis::template` |     9 |    20 |
| `ScopeData`                            | type  | `scope`             |     9 |     9 |
| `ScopeId`                              | type  | `scope`             |    20 |    27 |
| `ScopeKind`                            | type  | `scope`             |     8 |    25 |
| `SlotUsage`                            | type  | `croquis::template` |     2 |     2 |
| `SpreadProp`                           | type  | `croquis::template` |     1 |     1 |
| `TemplateExpression`                   | type  | `croquis`           |     1 |     1 |
| `TemplateExpressionKind`               | type  | `croquis`           |     2 |     2 |
| `build_effect_graph_from_script`       | type  | `effect_graph`      |     1 |     2 |
| `build_effect_graph_from_script_setup` | type  | `effect_graph`      |     1 |     1 |
| `build_effect_graph_from_sfc_scripts`  | type  | `effect_graph`      |     2 |     2 |
| `note_mounted_resources`               | type  | `mounted`           |     1 |     1 |
| `Croquis.bindings`                     | field | `croquis`           |     1 |     1 |
| `Croquis.component_usages`             | field | `croquis`           |     1 |     1 |
| `Croquis.element_ids`                  | field | `croquis`           |     1 |     1 |
| `Croquis.invalid_exports`              | field | `croquis`           |     1 |     2 |
| `Croquis.macros`                       | field | `croquis`           |    14 |    30 |
| `Croquis.nuxt_page_layout`             | field | `croquis`           |     1 |     1 |
| `Croquis.nuxt_render_template`         | field | `croquis`           |     1 |     1 |
| `Croquis.nuxt_static_render_script`    | field | `croquis`           |     1 |     1 |
| `Croquis.re_export_forwards`           | field | `croquis`           |     1 |     1 |
| `Croquis.reactivity`                   | field | `croquis`           |     2 |     2 |
| `Croquis.scopes`                       | field | `croquis`           |    13 |    21 |
| `Croquis.setup_context`                | field | `croquis`           |     2 |     4 |
| `Croquis.template_expressions`         | field | `croquis`           |     4 |     7 |
| `Croquis.template_info`                | field | `croquis`           |     6 |    15 |
| `Croquis.type_exports`                 | field | `croquis`           |     1 |     1 |
| `Croquis.types`                        | field | `croquis`           |     1 |     1 |
| `Croquis.used_directives`              | field | `croquis`           |     1 |     1 |

## Non-product `vize_croquis` imports

| item                          | files | sites |
| ----------------------------- | ----: | ----: |
| `Agreement`                   |     1 |     1 |
| `BindingType`                 |     1 |     6 |
| `Bindings`                    |     3 |     4 |
| `CroquisFacts`                |     2 |     2 |
| `Demand`                      |     1 |     6 |
| `EmitDefinition`              |     3 |     3 |
| `FactConsumer`                |     1 |     4 |
| `FactGroup`                   |     1 |     1 |
| `InjectEntry`                 |     5 |     9 |
| `InjectPattern`               |     4 |    14 |
| `LocalComponentBinding`       |     1 |    11 |
| `MacroKind`                   |     1 |     2 |
| `PropDefinition`              |     3 |     3 |
| `ProvideEntry`                |     5 |    11 |
| `ProvideKey`                  |    10 |    53 |
| `ProviderId`                  |     1 |     1 |
| `RaceConditionRisk`           |     2 |     4 |
| `RaceConditionRiskKind`       |     2 |     3 |
| `ReactiveKind`                |     5 |    41 |
| `ReactivityLossKind`          |     3 |    28 |
| `SetupContextViolation`       |     1 |     1 |
| `SetupContextViolationKind`   |     2 |    11 |
| `SfcParseOptions`             |     2 |     2 |
| `SourceFact`                  |     1 |     1 |
| `TypeDefinitions`             |     2 |     4 |
| `ViolationSeverity`           |     1 |     6 |
| `component_identity`          |     1 |     1 |
| `component_reference_name`    |     1 |     1 |
| `component_usage_list`        |    10 |    16 |
| `composable_calls`            |     3 |     3 |
| `extract_identifiers_checked` |     1 |     1 |
| `inject_entries`              |     7 |    17 |
| `local_component_binding`     |     1 |     3 |
| `parse_sfc`                   |     1 |     1 |
| `parse_sfc_without_css_vars`  |     1 |     1 |
| `provide_entries`             |     4 |     7 |
| `race_risks`                  |     1 |     1 |
| `reactivity_count`            |     3 |     3 |
| `reactivity_is_reactive`      |     1 |     5 |
| `reactivity_lookup`           |     2 |     2 |
| `reactivity_losses`           |     3 |     3 |
| `reactivity_sources`          |     2 |     2 |
| `to_pascal_case`              |     1 |     3 |
| `used_component_contains`     |     1 |     1 |
| `used_component_name_list`    |     5 |     7 |
