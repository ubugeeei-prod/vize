<!-- GENERATED FILE — do not edit by hand.
     Regenerate: node tools/support/compat/levels/croquis-consumers.ts --write
     Verify:     node tools/support/compat/levels/croquis-consumers.ts --check -->

# Croquis consumption: `vize_canon`

One shard of the [Croquis consumption matrix](../croquis-consumption.md): resolved consumption and non-product imports in `crates/vize_canon/src`. The method and product set live on that page; fresh naive-grep diagnostics are printed by `--check` and `--summary`.

## Resolved product sites

| product                                    | kind  | module                        | files | sites |
| ------------------------------------------ | ----- | ----------------------------- | ----: | ----: |
| `Analyzer`                                 | type  | `analyzer`                    |    92 |   269 |
| `AnalyzerOptions`                          | type  | `analyzer`                    |    92 |   265 |
| `ComponentUsage`                           | type  | `croquis::template`           |    41 |   110 |
| `Croquis`                                  | type  | `croquis`                     |   121 |   350 |
| `Drawer`                                   | type  | `drawer`                      |     3 |     3 |
| `DrawerOptions`                            | type  | `drawer`                      |     3 |     3 |
| `EventHandlerScopeData`                    | type  | `scope`                       |     8 |    10 |
| `EventListener`                            | type  | `croquis::template`           |     2 |     4 |
| `MacroTracker`                             | type  | `macros`                      |     2 |     4 |
| `NonScriptSetupScopeData`                  | type  | `scope`                       |     1 |     3 |
| `OptionGroup`                              | type  | `croquis`                     |     2 |     7 |
| `PassedProp`                               | type  | `croquis::template`           |    21 |    59 |
| `Scope`                                    | type  | `scope`                       |    18 |    36 |
| `ScopeBinding`                             | type  | `scope`                       |     1 |     1 |
| `ScopeChain`                               | type  | `scope`                       |     9 |    18 |
| `ScopeData`                                | type  | `scope`                       |    29 |    47 |
| `ScopeId`                                  | type  | `scope`                       |    21 |    50 |
| `ScopeKind`                                | type  | `scope`                       |    29 |   132 |
| `SlotUsage`                                | type  | `croquis::template`           |     2 |     2 |
| `SpreadProp`                               | type  | `croquis::template`           |     4 |     6 |
| `TemplateExpression`                       | type  | `croquis`                     |    20 |    40 |
| `TemplateExpressionKind`                   | type  | `croquis`                     |    15 |    39 |
| `TypeExport`                               | type  | `croquis`                     |     2 |    10 |
| `TypeExportKind`                           | type  | `croquis`                     |     2 |     8 |
| `UndefinedRef`                             | type  | `croquis`                     |     2 |     4 |
| `VForScopeData`                            | type  | `scope`                       |     1 |     4 |
| `VSlotScopeData`                           | type  | `scope`                       |     2 |     5 |
| `normalize_options_template_binding_name`  | type  | `croquis::options_descriptor` |     1 |     1 |
| `Croquis.bindings`                         | field | `croquis`                     |     3 |     8 |
| `Croquis.component_registrations`          | field | `croquis`                     |     1 |     1 |
| `Croquis.component_usages`                 | field | `croquis`                     |     2 |     3 |
| `Croquis.import_statements`                | field | `croquis`                     |     5 |     6 |
| `Croquis.invalid_exports`                  | field | `croquis`                     |     1 |     1 |
| `Croquis.macros`                           | field | `croquis`                     |    37 |   118 |
| `Croquis.options_descriptor`               | field | `croquis`                     |     3 |     3 |
| `Croquis.pattern_diagnostics`              | field | `croquis`                     |     2 |     4 |
| `Croquis.re_exports`                       | field | `croquis`                     |     1 |     1 |
| `Croquis.reactivity`                       | field | `croquis`                     |     1 |     2 |
| `Croquis.scopes`                           | field | `croquis`                     |    54 |   109 |
| `Croquis.setup_context`                    | field | `croquis`                     |     1 |     1 |
| `Croquis.template_component_registrations` | field | `croquis`                     |     1 |     1 |
| `Croquis.template_expression_reads`        | field | `croquis`                     |     1 |     3 |
| `Croquis.template_expressions`             | field | `croquis`                     |    18 |    23 |
| `Croquis.template_info`                    | field | `croquis`                     |     2 |     4 |
| `Croquis.type_exports`                     | field | `croquis`                     |    14 |    27 |
| `Croquis.types`                            | field | `croquis`                     |     6 |    12 |
| `Croquis.used_directives`                  | field | `croquis`                     |     1 |     1 |

## Non-product `vize_croquis` imports

| item                                     | files | sites |
| ---------------------------------------- | ----: | ----: |
| `AuthoredModuleExport`                   |     4 |     5 |
| `AuthoredSourceOffsets`                  |     1 |     2 |
| `BindingType`                            |    22 |    55 |
| `Bindings`                               |     3 |    14 |
| `BindingsTable`                          |     1 |     3 |
| `BlockLocation`                          |     1 |     1 |
| `CroquisFacts`                           |     4 |     7 |
| `DEFINE_ART`                             |     2 |     2 |
| `DEFINE_EMITS`                           |     1 |     1 |
| `DEFINE_EXPOSE`                          |     1 |     1 |
| `DEFINE_MODEL`                           |     1 |     1 |
| `DEFINE_PROPS`                           |     2 |     2 |
| `DEFINE_SLOTS`                           |     1 |     1 |
| `Demand`                                 |     4 |    10 |
| `EventHandlerExpression`                 |     6 |    16 |
| `FactConsumer`                           |     4 |     5 |
| `FactTable`                              |     1 |     4 |
| `MacroCall`                              |     1 |     1 |
| `MacroKind`                              |     4 |     5 |
| `ModelDefinition`                        |     5 |     9 |
| `PatternDiagnostic`                      |     1 |     2 |
| `PropDefinition`                         |     8 |    30 |
| `PropsDestructuredBindings`              |     1 |     1 |
| `ReactivityLossKind`                     |     1 |    11 |
| `SfcDescriptor`                          |    15 |    32 |
| `SfcError`                               |     2 |     2 |
| `SfcParseOptions`                        |    19 |    27 |
| `SfcTemplateBlock`                       |     1 |     1 |
| `SharedParseScope`                       |     1 |     1 |
| `UndefinedRefs`                          |     3 |     7 |
| `ViolationSeverity`                      |     1 |     3 |
| `WITH_DEFAULTS`                          |     1 |     1 |
| `classify_event_handler`                 |     5 |     8 |
| `classify_event_handler_with_identifier` |     1 |     1 |
| `component_usage_list`                   |     7 |     9 |
| `component_usage_slice`                  |    18 |    23 |
| `dynamic_component_alias`                |     5 |     6 |
| `extract_identifier_refs_checked`        |     2 |     3 |
| `extract_identifier_refs_oxc`            |     3 |     4 |
| `extract_identifier_uses_checked`        |     1 |     1 |
| `extract_identifiers_oxc`                |     5 |     6 |
| `has_single_setup_binding_frame`         |     1 |     3 |
| `homogeneous_index_property`             |     1 |     1 |
| `is_dynamic_component_alias`             |     3 |     6 |
| `is_event_local`                         |     1 |     1 |
| `is_js_global`                           |     2 |     2 |
| `is_keyword`                             |     1 |     1 |
| `is_render_local`                        |     1 |     1 |
| `is_runtime_builtin_component`           |     1 |     1 |
| `is_vue_builtin`                         |     1 |     1 |
| `literal_boolean_type`                   |     1 |     1 |
| `parse_program_for_analysis`             |     4 |     4 |
| `parse_script`                           |     1 |     1 |
| `parse_script_setup`                     |     1 |     1 |
| `reactivity_losses`                      |     1 |     1 |
| `strip_js_comments`                      |     3 |     4 |
| `strip_js_comments_with_offsets`         |     3 |     3 |
| `to_pascal_case`                         |     2 |     2 |
| `used_component_contains`                |     1 |     1 |
| `used_component_name_list`               |     5 |     6 |
| `used_components_empty`                  |     1 |     1 |
| `with_shared_parse`                      |     7 |     9 |
| `with_shared_program_for_analysis`       |     1 |     4 |
