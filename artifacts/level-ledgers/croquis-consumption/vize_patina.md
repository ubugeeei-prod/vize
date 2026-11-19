<!-- GENERATED FILE — do not edit by hand.
     Regenerate: node tools/support/compat/levels/croquis-consumers.ts --write
     Verify:     node tools/support/compat/levels/croquis-consumers.ts --check -->

# Croquis consumption: `vize_patina`

One shard of the [Croquis consumption matrix](../croquis-consumption.md): resolved consumption and non-product imports in `crates/vize_patina/src`. The method and product set live on that page; fresh naive-grep diagnostics are printed by `--check` and `--summary`.

## Resolved product sites

| product                                    | kind  | module                         | files | sites |
| ------------------------------------------ | ----- | ------------------------------ | ----: | ----: |
| `COMPILER_MACRO_NAMES`                     | type  | `croquis`                      |     1 |     1 |
| `Croquis`                                  | type  | `croquis`                      |    24 |    47 |
| `Drawer`                                   | type  | `drawer`                       |     2 |     2 |
| `DrawerOptions`                            | type  | `drawer`                       |     1 |     1 |
| `ElementIdKind`                            | type  | `croquis::template`            |     1 |     2 |
| `OptionGroup`                              | type  | `croquis`                      |     2 |     8 |
| `OptionMember`                             | type  | `croquis`                      |     2 |     4 |
| `Scope`                                    | type  | `scope`                        |     2 |     2 |
| `ScopeData`                                | type  | `scope`                        |     2 |     2 |
| `ScopeId`                                  | type  | `scope`                        |     1 |     2 |
| `ScopeKind`                                | type  | `scope`                        |     3 |     5 |
| `TemplateComponentRegistrations`           | type  | `croquis::template_components` |     1 |     1 |
| `UnusedVarContext`                         | type  | `croquis`                      |     1 |     4 |
| `Croquis.binding_spans`                    | field | `croquis`                      |     4 |     5 |
| `Croquis.bindings`                         | field | `croquis`                      |     6 |     8 |
| `Croquis.component_registrations`          | field | `croquis`                      |     2 |     2 |
| `Croquis.element_ids`                      | field | `croquis`                      |     1 |     2 |
| `Croquis.import_statements`                | field | `croquis`                      |     1 |     1 |
| `Croquis.macros`                           | field | `croquis`                      |    11 |    27 |
| `Croquis.options_descriptor`               | field | `croquis`                      |     1 |     1 |
| `Croquis.scopes`                           | field | `croquis`                      |     2 |     5 |
| `Croquis.template_component_registrations` | field | `croquis`                      |     1 |     1 |
| `Croquis.types`                            | field | `croquis`                      |     2 |     5 |
| `Croquis.unused_bindings`                  | field | `croquis`                      |     2 |     7 |

## Non-product `vize_croquis` imports

| item                                            | files | sites |
| ----------------------------------------------- | ----: | ----: |
| `BindingKey`                                    |     1 |     2 |
| `BindingType`                                   |     1 |     1 |
| `Bindings`                                      |     3 |     5 |
| `BlockLocation`                                 |     2 |     3 |
| `CROQUIS_FACTS`                                 |     1 |     2 |
| `ComponentContract`                             |     1 |     1 |
| `ComponentIdentity`                             |     1 |     1 |
| `ComponentUsages`                               |     1 |     3 |
| `CroquisFacts`                                  |     4 |     5 |
| `Demand`                                        |     3 |     6 |
| `EmitContract`                                  |     1 |     1 |
| `FactConsumer`                                  |     5 |     5 |
| `FactManager`                                   |     1 |     2 |
| `FactTable`                                     |     1 |     1 |
| `FactView`                                      |     1 |     1 |
| `GroupedComponentUse`                           |     1 |     1 |
| `InjectPattern`                                 |     1 |     5 |
| `LintTemplate`                                  |     2 |     5 |
| `MacroCall`                                     |     1 |     1 |
| `MacroKind`                                     |     3 |     3 |
| `PropContract`                                  |     1 |     1 |
| `ProvideInject`                                 |     1 |     3 |
| `ProvideInjectFact`                             |     1 |     4 |
| `ProvideInjectKey`                              |     1 |     4 |
| `ProvideKey`                                    |     1 |     3 |
| `RaceConditionRisk`                             |     1 |     1 |
| `RaceConditionRiskKind`                         |     1 |     5 |
| `RaceConditions`                                |     1 |     3 |
| `ReactiveKind`                                  |     2 |    10 |
| `Reactivity`                                    |     1 |     3 |
| `ReactivityContract`                            |     1 |     1 |
| `ReactivityFact`                                |     1 |     3 |
| `ReactivityKey`                                 |     1 |     3 |
| `ReactivityLoss`                                |     2 |     2 |
| `ReactivityLossKind`                            |     3 |    34 |
| `ResolvedTypeWorld`                             |    11 |    21 |
| `ScriptParseResult`                             |     3 |     3 |
| `ScriptParserOptions`                           |     2 |     2 |
| `SfcCustomBlock`                                |     1 |     2 |
| `SfcDescriptor`                                 |    24 |    34 |
| `SfcError`                                      |     1 |     2 |
| `SfcParseOptions`                               |    22 |    24 |
| `SfcScriptBlock`                                |     4 |     6 |
| `SfcStyleBlock`                                 |     1 |     2 |
| `SfcTemplateBlock`                              |     1 |     1 |
| `SignatureContract`                             |     1 |     1 |
| `SlotContract`                                  |     1 |     1 |
| `TemplateReferenceOwner`                        |     3 |     3 |
| `TemplateReferences`                            |     3 |     3 |
| `TypeDeclarationKind`                           |     1 |     1 |
| `TypeDefinitions`                               |     1 |     1 |
| `TypeLookup`                                    |     1 |     1 |
| `UndefinedRefs`                                 |     2 |     5 |
| `UnusedBindings`                                |     2 |     7 |
| `analyze_script_program`                        |     1 |     1 |
| `camelize`                                      |     1 |     2 |
| `capitalize`                                    |     1 |     2 |
| `collect_lint_template_references`              |     1 |     1 |
| `collect_options_descriptor`                    |     5 |     5 |
| `collect_options_object`                        |     1 |     1 |
| `extract_identifier_refs_retained_only`         |     2 |     3 |
| `extract_identifier_refs_retained_with_witness` |     1 |     1 |
| `extract_identifiers_oxc`                       |     1 |     2 |
| `extract_slot_props`                            |     2 |     2 |
| `is_builtin_component`                          |     3 |     5 |
| `is_kebab_case_loose`                           |     1 |     1 |
| `is_native_tag`                                 |     1 |     1 |
| `is_pascal_case`                                |     2 |     2 |
| `names_match`                                   |     4 |     4 |
| `parse_script_setup`                            |     4 |     4 |
| `parse_script_setup_with_generic_and_jsx`       |     1 |     1 |
| `parse_script_with_options`                     |     2 |     2 |
| `parse_script_with_options_and_jsx`             |     1 |     1 |
| `parse_sfc`                                     |     2 |     2 |
| `parse_v_for_expression`                        |     4 |     4 |
| `parse_v_for_scope_expression`                  |     7 |     8 |
| `reactivity_lookup`                             |     1 |     1 |
| `retain_v_for_binding_parameters`               |     2 |     2 |
| `retain_v_for_binding_parameters_with_source`   |     1 |     1 |
| `retained_binding_declarations`                 |     1 |     1 |
| `strip_js_comments`                             |     1 |     1 |
| `strip_js_comments_with_offsets`                |     2 |     2 |
| `to_pascal_case`                                |     6 |     9 |
| `used_component_name_list`                      |     1 |     1 |
