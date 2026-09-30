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
| `Croquis.template_info`               | field | `croquis`      |     1 |     3 |

## Non-product `vize_croquis` imports

| item                                      | files | sites |
| ----------------------------------------- | ----: | ----: |
| `BlockLocation`                           |     1 |     1 |
| `ScriptParseResult`                       |     1 |     5 |
| `ScriptParserOptions`                     |     1 |     1 |
| `SfcDescriptor`                           |     3 |     5 |
| `SfcParseOptions`                         |    12 |    15 |
| `SfcScriptBlock`                          |     1 |     1 |
| `component_usage_list`                    |     1 |     1 |
| `parse_program_for_analysis`              |     1 |     1 |
| `parse_script_setup_with_generic_and_jsx` |     1 |     1 |
| `parse_script_with_options_and_jsx`       |     1 |     1 |
| `to_pascal_case`                          |     1 |     1 |

## Naive grep disagreements (resolved/grep)

| product                               | resolved | grep |
| ------------------------------------- | -------: | ---: |
| `Analyzer`                            |        1 |    2 |
| `AnalyzerOptions`                     |        1 |    2 |
| `Croquis`                             |        1 |    4 |
| `EffectGraphScript`                   |        2 |    3 |
| `Scope`                               |        0 |    1 |
| `Span`                                |        0 |    6 |
| `build_effect_graph_from_sfc_scripts` |        1 |    2 |
| `Croquis.bindings`                    |        0 |    4 |
| `Croquis.import_statements`           |        0 |    1 |
| `Croquis.scopes`                      |        0 |    2 |
| `Croquis.template_info`               |        3 |    4 |
| `Croquis.types`                       |        0 |    5 |
