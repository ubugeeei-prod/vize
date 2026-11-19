<!-- GENERATED FILE — do not edit by hand.
     Regenerate: node tools/support/compat/levels/croquis-consumers.ts --write
     Verify:     node tools/support/compat/levels/croquis-consumers.ts --check -->

# Croquis consumption: `vize_maestro`

One shard of the [Croquis consumption matrix](../croquis-consumption.md): resolved consumption and non-product imports in `crates/vize_maestro/src`. The method and product set live on that page; fresh naive-grep diagnostics are printed by `--check` and `--summary`.

## Resolved product sites

| product                           | kind  | module              | files | sites |
| --------------------------------- | ----- | ------------------- | ----: | ----: |
| `Analyzer`                        | type  | `analyzer`          |     3 |     3 |
| `AnalyzerOptions`                 | type  | `analyzer`          |     3 |     3 |
| `ComponentShape`                  | type  | `croquis`           |     1 |     1 |
| `Croquis`                         | type  | `croquis`           |    10 |    18 |
| `Drawer`                          | type  | `drawer`            |    22 |    25 |
| `DrawerOptions`                   | type  | `drawer`            |    22 |    25 |
| `MacroTracker`                    | type  | `macros`            |     1 |     4 |
| `ScopeBinding`                    | type  | `scope`             |     1 |     1 |
| `ScopeData`                       | type  | `scope`             |     5 |    10 |
| `ScopeKind`                       | type  | `scope`             |    11 |    74 |
| `SlotUsage`                       | type  | `croquis::template` |     1 |     1 |
| `Croquis.binding_spans`           | field | `croquis`           |     2 |     2 |
| `Croquis.bindings`                | field | `croquis`           |     1 |     2 |
| `Croquis.component_registrations` | field | `croquis`           |     1 |     1 |
| `Croquis.component_shape`         | field | `croquis`           |     1 |     1 |
| `Croquis.macros`                  | field | `croquis`           |     9 |    21 |
| `Croquis.pattern_diagnostics`     | field | `croquis`           |     2 |     3 |
| `Croquis.scopes`                  | field | `croquis`           |    11 |    18 |
| `Croquis.template_expressions`    | field | `croquis`           |     1 |     2 |
| `Croquis.types`                   | field | `croquis`           |     1 |     1 |

## Non-product `vize_croquis` imports

| item                            | files | sites |
| ------------------------------- | ----: | ----: |
| `AlphaSchema`                   |     1 |     1 |
| `BindingIdentity`               |     2 |     8 |
| `BindingOccurrences`            |     3 |     5 |
| `Bindings`                      |     6 |    13 |
| `BindingsTable`                 |     2 |     2 |
| `BlockLocation`                 |     2 |     2 |
| `CroquisFacts`                  |     6 |     6 |
| `Demand`                        |     6 |    14 |
| `EmitContract`                  |     1 |     1 |
| `FactConsumer`                  |     6 |     8 |
| `FactGroup`                     |     3 |     3 |
| `OccurrenceBlock`               |     2 |    11 |
| `PropContract`                  |     2 |     3 |
| `ReactiveKind`                  |     6 |    43 |
| `SfcDescriptor`                 |     8 |    30 |
| `SfcScriptBlock`                |     4 |     7 |
| `SfcStyleBlock`                 |     2 |     8 |
| `SfcTemplateBlock`              |     1 |     1 |
| `SignatureContract`             |     2 |     4 |
| `SlotContract`                  |     1 |     2 |
| `TemplateSemanticReuse`         |     3 |     5 |
| `TypeEnvironment`               |     1 |     1 |
| `collect_petite_semantic_reuse` |     1 |     1 |
| `component_usage_list`          |     5 |     6 |
| `declaration_key`               |     1 |     1 |
| `extract_identifier_refs_oxc`   |     1 |     1 |
| `extract_identifiers_oxc`       |     2 |     2 |
| `hyphenate`                     |     1 |     1 |
| `is_kebab_case`                 |     1 |     1 |
| `parse_program_for_analysis`    |     5 |     6 |
| `parse_script_setup`            |     2 |     4 |
| `reactivity_lookup`             |     3 |     3 |
| `reactivity_sources`            |     3 |     3 |
| `to_pascal_case`                |     1 |     2 |
