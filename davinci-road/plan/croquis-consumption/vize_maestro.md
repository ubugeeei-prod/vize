<!-- GENERATED FILE — do not edit by hand.
     Regenerate: rust-script tools/commands/davinci/croquis-consumers.rs --write
     Verify:     rust-script tools/commands/davinci/croquis-consumers.rs --check -->

# Croquis consumption: `vize_maestro`

One shard of the [Croquis consumption matrix](../croquis-consumption.md): every fact the matrix records about `crates/vize_maestro/src`. The method and the product set live on that page.

## Resolved product sites

| product                        | kind  | module              | files | sites |
| ------------------------------ | ----- | ------------------- | ----: | ----: |
| `Analyzer`                     | type  | `analyzer`          |     2 |     2 |
| `AnalyzerOptions`              | type  | `analyzer`          |     2 |     2 |
| `ComponentShape`               | type  | `croquis`           |     1 |     1 |
| `ComponentUsage`               | type  | `croquis::template` |     2 |     5 |
| `Croquis`                      | type  | `croquis`           |     5 |     7 |
| `Drawer`                       | type  | `drawer`            |    19 |    23 |
| `DrawerOptions`                | type  | `drawer`            |    19 |    23 |
| `EventListener`                | type  | `croquis::template` |     1 |     2 |
| `MacroTracker`                 | type  | `macros`            |     1 |     4 |
| `PassedProp`                   | type  | `croquis::template` |     2 |     4 |
| `ScopeBinding`                 | type  | `scope`             |     1 |     1 |
| `ScopeData`                    | type  | `scope`             |     3 |     8 |
| `ScopeId`                      | type  | `scope`             |     1 |     1 |
| `ScopeKind`                    | type  | `scope`             |     9 |    65 |
| `SlotUsage`                    | type  | `croquis::template` |     2 |     3 |
| `Croquis.binding_spans`        | field | `croquis`           |     2 |     2 |
| `Croquis.bindings`             | field | `croquis`           |     5 |     8 |
| `Croquis.component_shape`      | field | `croquis`           |     1 |     1 |
| `Croquis.component_usages`     | field | `croquis`           |     6 |     7 |
| `Croquis.macros`               | field | `croquis`           |     7 |    14 |
| `Croquis.pattern_diagnostics`  | field | `croquis`           |     2 |     3 |
| `Croquis.reactivity`           | field | `croquis`           |     5 |     6 |
| `Croquis.scopes`               | field | `croquis`           |     8 |    15 |
| `Croquis.template_expressions` | field | `croquis`           |     1 |     2 |

## Non-product `vize_croquis` imports

| item                          | files | sites |
| ----------------------------- | ----: | ----: |
| `BlockLocation`               |     2 |     2 |
| `ReactiveKind`                |     5 |    41 |
| `SfcDescriptor`               |     6 |    24 |
| `SfcScriptBlock`              |     3 |     6 |
| `SfcStyleBlock`               |     2 |     8 |
| `extract_identifier_refs_oxc` |     1 |     1 |
| `extract_identifiers_oxc`     |     2 |     2 |
| `is_kebab_case`               |     1 |     1 |
| `parse_script_setup`          |     2 |     4 |
| `v_bind_expression_ranges`    |     1 |     1 |

## Naive grep disagreements (resolved/grep)

| product                       | resolved | grep |
| ----------------------------- | -------: | ---: |
| `Analyzer`                    |        2 |    3 |
| `AnalyzerOptions`             |        2 |    3 |
| `ComponentShape`              |        1 |    2 |
| `ComponentUsage`              |        5 |    8 |
| `Croquis`                     |        7 |   23 |
| `Drawer`                      |       23 |   40 |
| `DrawerOptions`               |       23 |   41 |
| `EventListener`               |        2 |    4 |
| `PassedProp`                  |        4 |    7 |
| `Scope`                       |        0 |    3 |
| `ScopeBinding`                |        1 |    2 |
| `ScopeData`                   |        8 |   11 |
| `ScopeId`                     |        1 |    2 |
| `ScopeKind`                   |       65 |   73 |
| `SlotUsage`                   |        3 |    6 |
| `Span`                        |        0 |   12 |
| `Symbol`                      |        0 |    1 |
| `SymbolId`                    |        0 |    5 |
| `TemplateExpression`          |        0 |   15 |
| `Croquis.binding_spans`       |        2 |    3 |
| `Croquis.bindings`            |        8 |   14 |
| `Croquis.import_statements`   |        0 |    1 |
| `Croquis.macros`              |       14 |   24 |
| `Croquis.pattern_diagnostics` |        3 |    4 |
| `Croquis.scopes`              |       15 |   16 |
| `Croquis.types`               |        0 |    1 |
