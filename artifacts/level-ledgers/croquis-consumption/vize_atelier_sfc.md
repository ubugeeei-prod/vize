<!-- GENERATED FILE — do not edit by hand.
     Regenerate: node tools/support/compat/levels/croquis-consumers.ts --write
     Verify:     node tools/support/compat/levels/croquis-consumers.ts --check -->

# Croquis consumption: `vize_atelier_sfc`

One shard of the [Croquis consumption matrix](../croquis-consumption.md): resolved consumption and non-product imports in `crates/vize_atelier_sfc/src`. The method and product set live on that page; fresh naive-grep diagnostics are printed by `--check` and `--summary`.

## Resolved product sites

| product                     | kind  | module    | files | sites |
| --------------------------- | ----- | --------- | ----: | ----: |
| `BindingMetadata`           | type  | `croquis` |    18 |    39 |
| `Croquis`                   | type  | `croquis` |     9 |    37 |
| `Drawer`                    | type  | `drawer`  |     3 |    24 |
| `DrawerOptions`             | type  | `drawer`  |     1 |     5 |
| `ScopeKind`                 | type  | `scope`   |     1 |     2 |
| `Croquis.binding_spans`     | field | `croquis` |     3 |     4 |
| `Croquis.bindings`          | field | `croquis` |     5 |    20 |
| `Croquis.import_statements` | field | `croquis` |     1 |     2 |
| `Croquis.macros`            | field | `croquis` |     7 |    18 |
| `Croquis.scopes`            | field | `croquis` |     1 |     2 |
| `Croquis.types`             | field | `croquis` |     5 |    17 |
| `Croquis.unused_bindings`   | field | `croquis` |     4 |    13 |

## Non-product `vize_croquis` imports

| item                                          | files | sites |
| --------------------------------------------- | ----: | ----: |
| `AuthoredModuleExport`                        |     4 |     4 |
| `BindingIdentity`                             |     1 |     1 |
| `BindingOccurrences`                          |     5 |    11 |
| `BindingType`                                 |    24 |   160 |
| `Bindings`                                    |     1 |     2 |
| `BlockLocation`                               |     9 |    17 |
| `CroquisFacts`                                |     1 |     1 |
| `CssVBindEdit`                                |     1 |     2 |
| `Data`                                        |     1 |     1 |
| `Demand`                                      |     1 |     2 |
| `EmitDefinition`                              |     1 |     1 |
| `FactConsumer`                                |     1 |     1 |
| `LintTemplate`                                |     3 |     6 |
| `ModelDefinition`                             |     1 |     1 |
| `OccurrenceBlock`                             |     1 |    15 |
| `PadOption`                                   |     3 |     4 |
| `PropDefinition`                              |     3 |     5 |
| `Props`                                       |     1 |     1 |
| `ResolvedTypeWorld`                           |     6 |    13 |
| `ScriptParseResult`                           |     1 |     1 |
| `SetupConst`                                  |     1 |     1 |
| `SetupMaybeRef`                               |     1 |     1 |
| `SetupReactiveConst`                          |     1 |     1 |
| `SetupRef`                                    |     1 |     1 |
| `SfcCustomBlock`                              |     3 |     3 |
| `SfcDescriptor`                               |    31 |    80 |
| `SfcError`                                    |    32 |   111 |
| `SfcParseOptions`                             |    45 |   188 |
| `SfcScriptBlock`                              |     4 |     4 |
| `SfcStyleBlock`                               |     7 |    10 |
| `SfcTemplateBlock`                            |    10 |    17 |
| `SharedProgram`                               |     1 |     1 |
| `TemplateSemanticReuse`                       |     3 |     4 |
| `TypeDeclaration`                             |     1 |     3 |
| `TypeDeclarationKind`                         |     1 |     2 |
| `TypeExportBinding`                           |     3 |     9 |
| `TypeImport`                                  |     1 |     1 |
| `TypeLookup`                                  |     6 |    27 |
| `TypeModule`                                  |     2 |    12 |
| `TypeModuleReference`                         |     2 |     3 |
| `UnknownTypeReason`                           |     1 |     1 |
| `UnusedBindings`                              |     1 |     2 |
| `VueGlobal`                                   |     1 |     1 |
| `checked_v_bind_expression_ranges`            |     1 |     1 |
| `evaluate`                                    |     1 |     1 |
| `extract_and_transform_v_bind`                |     1 |     1 |
| `extract_and_transform_v_bind_with_context`   |     1 |     1 |
| `extract_and_transform_v_bind_with_edits`     |     1 |     1 |
| `extract_and_transform_v_bind_with_scope`     |     1 |     1 |
| `extract_css_vars`                            |     1 |     1 |
| `extract_identifier_refs_with_witness`        |     1 |     1 |
| `extract_identifiers_checked`                 |     1 |     1 |
| `find_matching_paren`                         |     1 |     1 |
| `is_builtin_component`                        |     1 |     1 |
| `parse_original_for_analysis`                 |     1 |     1 |
| `parse_script_setup`                          |     1 |     1 |
| `parse_sfc`                                   |     2 |     2 |
| `prod_scoped_v_bind_name`                     |     1 |     1 |
| `retain_v_for_binding_parameters_with_source` |     1 |     1 |
| `scoped_v_bind_name`                          |     1 |     1 |
| `shared_program`                              |     1 |     1 |
| `with_shared_parse`                           |     2 |     2 |
