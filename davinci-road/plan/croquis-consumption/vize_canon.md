<!-- GENERATED FILE — do not edit by hand.
     Regenerate: rust-script tools/commands/davinci/croquis-consumers.rs --write
     Verify:     rust-script tools/commands/davinci/croquis-consumers.rs --check -->

# Croquis consumption: `vize_canon`

One shard of the [Croquis consumption matrix](../croquis-consumption.md): every fact the matrix records about `crates/vize_canon/src`. The method and the product set live on that page.

## Resolved product sites

| product                        | kind  | module              | files | sites |
| ------------------------------ | ----- | ------------------- | ----: | ----: |
| `Analyzer`                     | type  | `analyzer`          |    42 |   160 |
| `AnalyzerOptions`              | type  | `analyzer`          |    42 |   156 |
| `BindingMetadata`              | type  | `croquis`           |     2 |     5 |
| `ComponentUsage`               | type  | `croquis::template` |    20 |    44 |
| `Croquis`                      | type  | `croquis`           |    90 |   212 |
| `EventHandlerScopeData`        | type  | `scope`             |     7 |     9 |
| `EventListener`                | type  | `croquis::template` |     2 |     3 |
| `MacroTracker`                 | type  | `macros`            |     1 |     1 |
| `NonScriptSetupScopeData`      | type  | `scope`             |     1 |     3 |
| `OptionGroup`                  | type  | `croquis`           |     2 |     7 |
| `PassedProp`                   | type  | `croquis::template` |    10 |    23 |
| `Scope`                        | type  | `scope`             |    14 |    25 |
| `ScopeChain`                   | type  | `scope`             |     2 |     7 |
| `ScopeData`                    | type  | `scope`             |    19 |    35 |
| `ScopeId`                      | type  | `scope`             |     7 |    22 |
| `ScopeKind`                    | type  | `scope`             |    17 |    59 |
| `SlotUsage`                    | type  | `croquis::template` |     1 |     1 |
| `SpreadProp`                   | type  | `croquis::template` |     4 |     6 |
| `TemplateExpression`           | type  | `croquis`           |    11 |    22 |
| `TemplateExpressionKind`       | type  | `croquis`           |     6 |    14 |
| `TypeExport`                   | type  | `croquis`           |     2 |     9 |
| `TypeExportKind`               | type  | `croquis`           |     2 |     9 |
| `VForScopeData`                | type  | `scope`             |     1 |     4 |
| `VSlotScopeData`               | type  | `scope`             |     1 |     4 |
| `Croquis.binding_spans`        | field | `croquis`           |     6 |     9 |
| `Croquis.bindings`             | field | `croquis`           |    26 |    45 |
| `Croquis.component_usages`     | field | `croquis`           |    16 |    20 |
| `Croquis.import_statements`    | field | `croquis`           |     4 |     5 |
| `Croquis.invalid_exports`      | field | `croquis`           |     1 |     1 |
| `Croquis.macros`               | field | `croquis`           |    29 |    75 |
| `Croquis.options_descriptor`   | field | `croquis`           |     2 |     2 |
| `Croquis.pattern_diagnostics`  | field | `croquis`           |     2 |     4 |
| `Croquis.re_exports`           | field | `croquis`           |     1 |     1 |
| `Croquis.reactivity`           | field | `croquis`           |     2 |     3 |
| `Croquis.scopes`               | field | `croquis`           |    31 |    59 |
| `Croquis.setup_context`        | field | `croquis`           |     1 |     1 |
| `Croquis.template_expressions` | field | `croquis`           |    12 |    14 |
| `Croquis.template_info`        | field | `croquis`           |     2 |     4 |
| `Croquis.type_exports`         | field | `croquis`           |    10 |    20 |
| `Croquis.types`                | field | `croquis`           |     5 |     9 |
| `Croquis.undefined_refs`       | field | `croquis`           |     6 |     7 |
| `Croquis.used_components`      | field | `croquis`           |     7 |     9 |

## Non-product `vize_croquis` imports

| item                           | files | sites |
| ------------------------------ | ----: | ----: |
| `BindingType`                  |    10 |    25 |
| `DEFINE_EMITS`                 |     1 |     1 |
| `DEFINE_EXPOSE`                |     1 |     1 |
| `DEFINE_MODEL`                 |     1 |     1 |
| `DEFINE_PROPS`                 |     2 |     2 |
| `DEFINE_SLOTS`                 |     1 |     1 |
| `EventHandlerExpression`       |     2 |     5 |
| `MacroCall`                    |     1 |     1 |
| `MacroKind`                    |     3 |     4 |
| `ModelDefinition`              |     5 |     8 |
| `PatternDiagnostic`            |     1 |     2 |
| `PropDefinition`               |     6 |    22 |
| `PropsDestructuredBindings`    |     1 |     1 |
| `ReactivityLossKind`           |     1 |    11 |
| `SfcDescriptor`                |    11 |    26 |
| `SfcError`                     |     2 |     2 |
| `SfcParseOptions`              |    13 |    19 |
| `SfcTemplateBlock`             |     2 |     2 |
| `ViolationSeverity`            |     1 |     3 |
| `WITH_DEFAULTS`                |     1 |     1 |
| `classify_event_handler`       |     2 |     3 |
| `extract_identifier_refs_oxc`  |     2 |     3 |
| `extract_identifiers_oxc`      |     5 |     6 |
| `is_dynamic_component_alias`   |     3 |     4 |
| `is_event_local`               |     1 |     1 |
| `is_js_global`                 |     2 |     2 |
| `is_keyword`                   |     1 |     1 |
| `is_render_local`              |     1 |     1 |
| `is_runtime_builtin_component` |     1 |     1 |
| `is_vue_builtin`               |     1 |     1 |
| `parse_program_for_analysis`   |     4 |     7 |
| `strip_js_comments`            |     3 |     3 |
| `to_pascal_case`               |     2 |     2 |

## Naive grep disagreements (resolved/grep)

| product                   | resolved | grep |
| ------------------------- | -------: | ---: |
| `Analyzer`                |      160 |  242 |
| `AnalyzerOptions`         |      156 |  238 |
| `BindingMetadata`         |        5 |    7 |
| `COMPILER_MACRO_NAMES`    |        0 |    1 |
| `ComponentUsage`          |       44 |   64 |
| `Croquis`                 |      212 |  315 |
| `Drawer`                  |        0 |    1 |
| `EventHandlerScopeData`   |        9 |   16 |
| `EventListener`           |        3 |    5 |
| `NonScriptSetupScopeData` |        3 |    4 |
| `OptionGroup`             |        7 |    9 |
| `PassedProp`              |       23 |   34 |
| `Scope`                   |       25 |   58 |
| `ScopeChain`              |        7 |    9 |
| `ScopeData`               |       35 |   53 |
| `ScopeId`                 |       22 |   29 |
| `ScopeKind`               |       59 |   76 |
| `SlotUsage`               |        1 |    3 |
| `Span`                    |        0 |   53 |
| `SpreadProp`              |        6 |   10 |
| `Symbol`                  |        0 |    4 |
| `SymbolId`                |        0 |    3 |
| `TemplateExpression`      |       22 |   38 |
| `TemplateExpressionKind`  |       14 |   20 |
| `TypeExport`              |        9 |   12 |
| `TypeExportKind`          |        9 |   11 |
| `TypeResolver`            |        0 |    5 |
| `VForScopeData`           |        4 |    5 |
| `VSlotScopeData`          |        4 |    5 |
| `Croquis.bindings`        |       45 |   89 |
| `Croquis.macros`          |       75 |   76 |
| `Croquis.scopes`          |       59 |   61 |
| `Croquis.types`           |        9 |   17 |
| `Croquis.undefined_refs`  |        7 |    8 |
