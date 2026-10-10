<!-- GENERATED FILE — do not edit by hand.
     Regenerate: rust-script tools/commands/davinci/croquis-consumers.rs --write
     Verify:     rust-script tools/commands/davinci/croquis-consumers.rs --check -->

# Croquis consumption: `vize_patina`

One shard of the [Croquis consumption matrix](../croquis-consumption.md): resolved consumption and non-product imports in `crates/vize_patina/src`. The method and product set live on that page; fresh naive-grep diagnostics are printed by `--check` and `--summary`.

## Resolved product sites

| product                                    | kind  | module                         | files | sites |
| ------------------------------------------ | ----- | ------------------------------ | ----: | ----: |
| `COMPILER_MACRO_NAMES`                     | type  | `croquis`                      |     1 |     1 |
| `Croquis`                                  | type  | `croquis`                      |    18 |    37 |
| `Drawer`                                   | type  | `drawer`                       |     2 |     2 |
| `DrawerOptions`                            | type  | `drawer`                       |     1 |     1 |
| `ElementIdKind`                            | type  | `croquis::template`            |     1 |     2 |
| `OptionMember`                             | type  | `croquis`                      |     2 |     3 |
| `Scope`                                    | type  | `scope`                        |     2 |     2 |
| `ScopeData`                                | type  | `scope`                        |     2 |     2 |
| `ScopeKind`                                | type  | `scope`                        |     3 |     4 |
| `TemplateComponentRegistrations`           | type  | `croquis::template_components` |     1 |     1 |
| `UnusedVarContext`                         | type  | `croquis`                      |     1 |     4 |
| `Croquis.binding_spans`                    | field | `croquis`                      |     2 |     3 |
| `Croquis.bindings`                         | field | `croquis`                      |     2 |     3 |
| `Croquis.component_registrations`          | field | `croquis`                      |     2 |     2 |
| `Croquis.element_ids`                      | field | `croquis`                      |     1 |     2 |
| `Croquis.import_statements`                | field | `croquis`                      |     1 |     1 |
| `Croquis.macros`                           | field | `croquis`                      |     9 |    23 |
| `Croquis.scopes`                           | field | `croquis`                      |     2 |     4 |
| `Croquis.template_component_registrations` | field | `croquis`                      |     1 |     1 |
| `Croquis.types`                            | field | `croquis`                      |     1 |     2 |
| `Croquis.unused_bindings`                  | field | `croquis`                      |     2 |     7 |

## Non-product `vize_croquis` imports

| item                                      | files | sites |
| ----------------------------------------- | ----: | ----: |
| `Bindings`                                |     1 |     2 |
| `BlockLocation`                           |     2 |     3 |
| `CroquisFacts`                            |     4 |     5 |
| `Demand`                                  |     3 |     6 |
| `FactConsumer`                            |     4 |     4 |
| `FactTable`                               |     1 |     1 |
| `FactView`                                |     1 |     1 |
| `MacroKind`                               |     3 |     3 |
| `ReactiveKind`                            |     1 |     1 |
| `ReactivityLoss`                          |     2 |     2 |
| `ReactivityLossKind`                      |     2 |    23 |
| `ScriptParseResult`                       |     3 |     3 |
| `ScriptParserOptions`                     |     2 |     2 |
| `SfcCustomBlock`                          |     1 |     2 |
| `SfcDescriptor`                           |    17 |    26 |
| `SfcError`                                |     1 |     2 |
| `SfcParseOptions`                         |    14 |    15 |
| `SfcScriptBlock`                          |     2 |     2 |
| `TypeDefinitions`                         |     1 |     1 |
| `UndefinedRefs`                           |     1 |     2 |
| `UnusedBindings`                          |     1 |     4 |
| `collect_options_descriptor`              |     4 |     4 |
| `collect_options_object`                  |     1 |     1 |
| `extract_identifier_refs_retained_only`   |     1 |     1 |
| `extract_identifiers_oxc`                 |     1 |     2 |
| `is_builtin_component`                    |     2 |     4 |
| `is_kebab_case_loose`                     |     1 |     1 |
| `is_pascal_case`                          |     2 |     2 |
| `names_match`                             |     3 |     3 |
| `parse_script_setup`                      |     4 |     4 |
| `parse_script_setup_with_generic_and_jsx` |     1 |     1 |
| `parse_script_with_options`               |     2 |     2 |
| `parse_script_with_options_and_jsx`       |     1 |     1 |
| `parse_v_for_expression`                  |     1 |     1 |
| `parse_v_for_scope_expression`            |     1 |     1 |
| `reactivity_lookup`                       |     1 |     1 |
| `to_pascal_case`                          |     5 |     7 |
| `used_component_name_list`                |     1 |     1 |
