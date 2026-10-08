<!-- GENERATED FILE — do not edit by hand.
     Regenerate: rust-script tools/commands/davinci/croquis-consumers.rs --write
     Verify:     rust-script tools/commands/davinci/croquis-consumers.rs --check -->

# Croquis consumption: `vize_canon`

One shard of the [Croquis consumption matrix](../croquis-consumption.md): resolved consumption and non-product imports in `crates/vize_canon/src`. The method and product set live on that page; fresh naive-grep diagnostics are printed by `--check` and `--summary`.

## Resolved product sites

| product                        | kind  | module              | files | sites |
| ------------------------------ | ----- | ------------------- | ----: | ----: |
| `Analyzer`                     | type  | `analyzer`          |    47 |   175 |
| `AnalyzerOptions`              | type  | `analyzer`          |    47 |   171 |
| `ComponentUsage`               | type  | `croquis::template` |    22 |    48 |
| `Croquis`                      | type  | `croquis`           |    94 |   225 |
| `EventHandlerScopeData`        | type  | `scope`             |     7 |     9 |
| `EventListener`                | type  | `croquis::template` |     2 |     3 |
| `MacroTracker`                 | type  | `macros`            |     1 |     2 |
| `NonScriptSetupScopeData`      | type  | `scope`             |     1 |     3 |
| `OptionGroup`                  | type  | `croquis`           |     2 |     7 |
| `PassedProp`                   | type  | `croquis::template` |    11 |    25 |
| `Scope`                        | type  | `scope`             |    14 |    25 |
| `ScopeChain`                   | type  | `scope`             |     2 |     7 |
| `ScopeData`                    | type  | `scope`             |    19 |    35 |
| `ScopeId`                      | type  | `scope`             |     8 |    23 |
| `ScopeKind`                    | type  | `scope`             |    17 |    59 |
| `SlotUsage`                    | type  | `croquis::template` |     1 |     1 |
| `SpreadProp`                   | type  | `croquis::template` |     4 |     6 |
| `TemplateExpression`           | type  | `croquis`           |    11 |    22 |
| `TemplateExpressionKind`       | type  | `croquis`           |     6 |    14 |
| `TypeExport`                   | type  | `croquis`           |     2 |     9 |
| `TypeExportKind`               | type  | `croquis`           |     2 |     9 |
| `UndefinedRef`                 | type  | `croquis`           |     1 |     1 |
| `VForScopeData`                | type  | `scope`             |     1 |     4 |
| `VSlotScopeData`               | type  | `scope`             |     1 |     4 |
| `Croquis.import_statements`    | field | `croquis`           |     4 |     5 |
| `Croquis.invalid_exports`      | field | `croquis`           |     1 |     1 |
| `Croquis.macros`               | field | `croquis`           |    29 |    83 |
| `Croquis.options_descriptor`   | field | `croquis`           |     2 |     2 |
| `Croquis.pattern_diagnostics`  | field | `croquis`           |     2 |     4 |
| `Croquis.re_exports`           | field | `croquis`           |     1 |     1 |
| `Croquis.reactivity`           | field | `croquis`           |     1 |     2 |
| `Croquis.scopes`               | field | `croquis`           |    31 |    60 |
| `Croquis.setup_context`        | field | `croquis`           |     1 |     1 |
| `Croquis.template_expressions` | field | `croquis`           |    12 |    14 |
| `Croquis.template_info`        | field | `croquis`           |     2 |     4 |
| `Croquis.type_exports`         | field | `croquis`           |    11 |    21 |
| `Croquis.types`                | field | `croquis`           |     5 |     9 |

## Non-product `vize_croquis` imports

| item                           | files | sites |
| ------------------------------ | ----: | ----: |
| `BindingType`                  |    11 |    28 |
| `Bindings`                     |     3 |    12 |
| `BindingsTable`                |     1 |     3 |
| `CroquisFacts`                 |     4 |     7 |
| `DEFINE_EMITS`                 |     1 |     1 |
| `DEFINE_EXPOSE`                |     1 |     1 |
| `DEFINE_MODEL`                 |     1 |     1 |
| `DEFINE_PROPS`                 |     2 |     2 |
| `DEFINE_SLOTS`                 |     1 |     1 |
| `Demand`                       |     4 |    10 |
| `EventHandlerExpression`       |     2 |     5 |
| `FactConsumer`                 |     4 |     5 |
| `FactTable`                    |     1 |     4 |
| `MacroCall`                    |     1 |     1 |
| `MacroKind`                    |     3 |     4 |
| `ModelDefinition`              |     5 |     8 |
| `PatternDiagnostic`            |     1 |     2 |
| `PropDefinition`               |     6 |    22 |
| `PropsDestructuredBindings`    |     1 |     1 |
| `ReactivityLossKind`           |     1 |    11 |
| `SfcDescriptor`                |    10 |    26 |
| `SfcError`                     |     2 |     2 |
| `SfcParseOptions`              |    14 |    20 |
| `SfcTemplateBlock`             |     2 |     2 |
| `UndefinedRefs`                |     3 |     7 |
| `ViolationSeverity`            |     1 |     3 |
| `WITH_DEFAULTS`                |     1 |     1 |
| `classify_event_handler`       |     2 |     3 |
| `component_usage_list`         |    18 |    24 |
| `dynamic_component_alias`      |     2 |     2 |
| `extract_identifier_refs_oxc`  |     2 |     3 |
| `extract_identifiers_oxc`      |     5 |     6 |
| `is_dynamic_component_alias`   |     3 |     5 |
| `is_event_local`               |     1 |     1 |
| `is_js_global`                 |     2 |     2 |
| `is_keyword`                   |     1 |     1 |
| `is_render_local`              |     1 |     1 |
| `is_runtime_builtin_component` |     1 |     1 |
| `is_vue_builtin`               |     1 |     1 |
| `parse_program_for_analysis`   |     4 |     7 |
| `reactivity_losses`            |     1 |     1 |
| `strip_js_comments`            |     3 |     3 |
| `to_pascal_case`               |     2 |     2 |
| `used_component_contains`      |     1 |     1 |
| `used_component_name_list`     |     5 |     6 |
| `used_components_empty`        |     2 |     2 |
