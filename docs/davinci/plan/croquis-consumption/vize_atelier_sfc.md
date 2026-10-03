<!-- GENERATED FILE — do not edit by hand.
     Regenerate: rust-script tools/commands/davinci/croquis-consumers.rs --write
     Verify:     rust-script tools/commands/davinci/croquis-consumers.rs --check -->

# Croquis consumption: `vize_atelier_sfc`

One shard of the [Croquis consumption matrix](../croquis-consumption.md): resolved consumption and non-product imports in `crates/vize_atelier_sfc/src`. The method and product set live on that page; fresh naive-grep diagnostics are printed by `--check` and `--summary`.

## Resolved product sites

| product                     | kind  | module    | files | sites |
| --------------------------- | ----- | --------- | ----: | ----: |
| `BindingMetadata`           | type  | `croquis` |    12 |    31 |
| `Croquis`                   | type  | `croquis` |    10 |    22 |
| `Drawer`                    | type  | `drawer`  |     2 |     9 |
| `DrawerOptions`             | type  | `drawer`  |     1 |     5 |
| `ScopeKind`                 | type  | `scope`   |     1 |     2 |
| `Croquis.binding_spans`     | field | `croquis` |     1 |     1 |
| `Croquis.bindings`          | field | `croquis` |     7 |    23 |
| `Croquis.import_statements` | field | `croquis` |     1 |     2 |
| `Croquis.macros`            | field | `croquis` |     6 |    17 |
| `Croquis.scopes`            | field | `croquis` |     1 |     2 |
| `Croquis.types`             | field | `croquis` |     4 |    16 |
| `Croquis.unused_bindings`   | field | `croquis` |     2 |     5 |

## Non-product `vize_croquis` imports

| item                                      | files | sites |
| ----------------------------------------- | ----: | ----: |
| `BindingType`                             |    17 |   131 |
| `BlockLocation`                           |     5 |     7 |
| `DEFINE_EMITS`                            |     1 |     2 |
| `DEFINE_EXPOSE`                           |     1 |     1 |
| `DEFINE_MODEL`                            |     1 |     1 |
| `DEFINE_OPTIONS`                          |     1 |     1 |
| `DEFINE_PROPS`                            |     1 |     1 |
| `DEFINE_SLOTS`                            |     1 |     1 |
| `EmitDefinition`                          |     1 |     1 |
| `ModelDefinition`                         |     1 |     1 |
| `PadOption`                               |     1 |     1 |
| `PropDefinition`                          |     3 |     5 |
| `ResolvedTypeWorld`                       |     2 |     5 |
| `ScriptParseResult`                       |     1 |     1 |
| `ScriptParserOptions`                     |     1 |     1 |
| `SfcCustomBlock`                          |     2 |     2 |
| `SfcDescriptor`                           |    18 |    45 |
| `SfcError`                                |    22 |    64 |
| `SfcParseOptions`                         |    19 |   120 |
| `SfcScriptBlock`                          |     2 |     2 |
| `SfcStyleBlock`                           |     5 |     8 |
| `SfcTemplateBlock`                        |     7 |     9 |
| `TypeDeclaration`                         |     1 |     3 |
| `TypeDeclarationKind`                     |     1 |     2 |
| `TypeExportBinding`                       |     2 |     8 |
| `TypeImport`                              |     1 |     1 |
| `TypeLookup`                              |     3 |    18 |
| `TypeModule`                              |     2 |     7 |
| `TypeModuleReference`                     |     2 |     3 |
| `UnknownTypeReason`                       |     1 |     1 |
| `WITH_DEFAULTS`                           |     1 |     1 |
| `analyze_script_setup_program`            |     1 |     1 |
| `artifact_macro_names`                    |     1 |     1 |
| `checked_v_bind_expression_ranges`        |     1 |     1 |
| `extract_and_transform_v_bind`            |     1 |     1 |
| `extract_and_transform_v_bind_with_scope` |     1 |     1 |
| `extract_identifiers_checked`             |     1 |     1 |
| `find_matching_paren`                     |     1 |     1 |
| `is_builtin_component`                    |     1 |     1 |
| `is_builtin_macro`                        |     2 |     2 |
| `is_global_allowed`                       |     1 |     1 |
| `is_runtime_erased_macro`                 |     1 |     1 |
| `macro_artifact_kind`                     |     2 |     3 |
| `parse_script_setup`                      |     1 |     2 |
| `parse_script_with_options_and_jsx`       |     1 |     1 |
| `parse_sfc`                               |     1 |     1 |
| `prod_scoped_v_bind_name`                 |     1 |     1 |
| `runtime_erased_macro_names`              |     3 |     5 |
| `scoped_v_bind_name`                      |     1 |     1 |
