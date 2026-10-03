<!-- GENERATED FILE — do not edit by hand.
     Regenerate: rust-script tools/commands/davinci/croquis-consumers.rs --write
     Verify:     rust-script tools/commands/davinci/croquis-consumers.rs --check -->

# Croquis consumption: `vize_maestro`

One shard of the [Croquis consumption matrix](../croquis-consumption.md): every fact the matrix records about `crates/vize_maestro/src`. The method and the product set live on that page.

## Resolved product sites

| product                        | kind  | module              | files | sites |
| ------------------------------ | ----- | ------------------- | ----: | ----: |
| `Analyzer`                     | type  | `analyzer`          |     3 |     3 |
| `AnalyzerOptions`              | type  | `analyzer`          |     3 |     3 |
| `ComponentShape`               | type  | `croquis`           |     1 |     1 |
| `Croquis`                      | type  | `croquis`           |     6 |    10 |
| `Drawer`                       | type  | `drawer`            |    19 |    22 |
| `DrawerOptions`                | type  | `drawer`            |    19 |    22 |
| `MacroTracker`                 | type  | `macros`            |     1 |     4 |
| `ScopeBinding`                 | type  | `scope`             |     1 |     1 |
| `ScopeData`                    | type  | `scope`             |     3 |     8 |
| `ScopeKind`                    | type  | `scope`             |     9 |    65 |
| `SlotUsage`                    | type  | `croquis::template` |     1 |     1 |
| `Croquis.component_shape`      | field | `croquis`           |     1 |     1 |
| `Croquis.macros`               | field | `croquis`           |     7 |    14 |
| `Croquis.pattern_diagnostics`  | field | `croquis`           |     2 |     3 |
| `Croquis.scopes`               | field | `croquis`           |     8 |    15 |
| `Croquis.template_expressions` | field | `croquis`           |     1 |     2 |

## Non-product `vize_croquis` imports

| item                          | files | sites |
| ----------------------------- | ----: | ----: |
| `AlphaSchema`                 |     1 |     1 |
| `Bindings`                    |     6 |    13 |
| `BindingsTable`               |     2 |     2 |
| `BlockLocation`               |     2 |     2 |
| `CroquisFacts`                |     6 |     6 |
| `Demand`                      |     6 |    14 |
| `EmitContract`                |     1 |     1 |
| `FactConsumer`                |     6 |     8 |
| `FactGroup`                   |     3 |     3 |
| `PropContract`                |     2 |     3 |
| `ReactiveKind`                |     5 |    41 |
| `SfcDescriptor`               |     7 |    25 |
| `SfcScriptBlock`              |     2 |     5 |
| `SfcStyleBlock`               |     2 |     8 |
| `SignatureContract`           |     2 |     4 |
| `SlotContract`                |     1 |     2 |
| `TypeEnvironment`             |     1 |     1 |
| `component_usage_list`        |     5 |     6 |
| `declaration_key`             |     1 |     1 |
| `extract_identifier_refs_oxc` |     1 |     1 |
| `extract_identifiers_oxc`     |     2 |     2 |
| `hyphenate`                   |     1 |     1 |
| `is_kebab_case`               |     1 |     1 |
| `parse_script_setup`          |     2 |     4 |
| `reactivity_lookup`           |     3 |     3 |
| `reactivity_sources`          |     3 |     3 |
| `v_bind_expression_ranges`    |     1 |     1 |

## Naive grep disagreements (resolved/grep)

| product                           | resolved | grep |
| --------------------------------- | -------: | ---: |
| `Analyzer`                        |        3 |    5 |
| `AnalyzerOptions`                 |        3 |    5 |
| `ComponentShape`                  |        1 |    2 |
| `Croquis`                         |       10 |   31 |
| `Drawer`                          |       22 |   38 |
| `DrawerOptions`                   |       22 |   39 |
| `Scope`                           |        0 |    3 |
| `ScopeBinding`                    |        1 |    2 |
| `ScopeData`                       |        8 |   11 |
| `ScopeKind`                       |       65 |   73 |
| `SlotUsage`                       |        1 |    2 |
| `Span`                            |        0 |   54 |
| `Symbol`                          |        0 |    1 |
| `SymbolId`                        |        0 |    5 |
| `TemplateExpression`              |        0 |   14 |
| `Croquis.binding_spans`           |        0 |    1 |
| `Croquis.bindings`                |        0 |    8 |
| `Croquis.component_registrations` |        0 |    1 |
| `Croquis.import_statements`       |        0 |    1 |
| `Croquis.macros`                  |       14 |   24 |
| `Croquis.pattern_diagnostics`     |        3 |    4 |
| `Croquis.reactivity`              |        0 |    1 |
| `Croquis.scopes`                  |       15 |   17 |
| `Croquis.types`                   |        0 |    1 |
