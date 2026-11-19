<!-- GENERATED FILE — do not edit by hand.
     Regenerate: node tools/support/compat/levels/croquis-consumers.ts --write
     Verify:     node tools/support/compat/levels/croquis-consumers.ts --check -->

# Croquis consumption: `vize`

One shard of the [Croquis consumption matrix](../croquis-consumption.md): resolved consumption and non-product imports in `crates/vize/src`. The method and product set live on that page; fresh naive-grep diagnostics are printed by `--check` and `--summary`.

## Resolved product sites

| product                               | kind  | module              | files | sites |
| ------------------------------------- | ----- | ------------------- | ----: | ----: |
| `Analyzer`                            | type  | `analyzer`          |     1 |     1 |
| `AnalyzerOptions`                     | type  | `analyzer`          |     1 |     1 |
| `Croquis`                             | type  | `croquis`           |     1 |     1 |
| `EffectGraphScript`                   | type  | `effect_graph`      |     2 |     4 |
| `EffectGraphSummary`                  | type  | `effect_graph`      |     2 |     6 |
| `NuxtTemplateRenderFacts`             | type  | `croquis::template` |     1 |     1 |
| `build_effect_graph_from_sfc_scripts` | type  | `effect_graph`      |     2 |     2 |
| `Croquis.component_usages`            | field | `croquis`           |     1 |     1 |
| `Croquis.nuxt_render_template`        | field | `croquis`           |     5 |     9 |
| `Croquis.nuxt_static_render_script`   | field | `croquis`           |     5 |     7 |
| `Croquis.template_info`               | field | `croquis`           |     2 |     4 |

## Non-product `vize_croquis` imports

| item                                      | files | sites |
| ----------------------------------------- | ----: | ----: |
| `BlockLocation`                           |     1 |     1 |
| `ScriptParseResult`                       |     1 |     5 |
| `ScriptParserOptions`                     |     1 |     1 |
| `SfcDescriptor`                           |     6 |     8 |
| `SfcError`                                |     2 |     4 |
| `SfcParseOptions`                         |    15 |    20 |
| `SfcScriptBlock`                          |     1 |     1 |
| `component_usage_list`                    |     1 |     1 |
| `composable_calls`                        |     1 |     1 |
| `parse_program_for_analysis`              |     1 |     1 |
| `parse_script_setup_with_generic_and_jsx` |     1 |     1 |
| `parse_script_with_options_and_jsx`       |     1 |     1 |
| `to_pascal_case`                          |     1 |     1 |
