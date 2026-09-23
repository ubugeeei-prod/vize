<!-- GENERATED FILE — do not edit by hand.
     Regenerate: rust-script tools/commands/davinci/croquis-consumers.rs --write
     Verify:     rust-script tools/commands/davinci/croquis-consumers.rs --check -->

# Croquis consumption: `vize_patina`

One shard of the [Croquis consumption matrix](../croquis-consumption.md): every fact the matrix records about `crates/vize_patina/src`. The method and the product set live on that page.

## Resolved product sites

| product                           | kind  | module              | files | sites |
| --------------------------------- | ----- | ------------------- | ----: | ----: |
| `COMPILER_MACRO_NAMES`            | type  | `croquis`           |     1 |     1 |
| `Croquis`                         | type  | `croquis`           |    11 |    26 |
| `Drawer`                          | type  | `drawer`            |     1 |     1 |
| `ElementIdKind`                   | type  | `croquis::template` |     1 |     2 |
| `OptionMember`                    | type  | `croquis`           |     2 |     3 |
| `Scope`                           | type  | `scope`             |     2 |     2 |
| `ScopeData`                       | type  | `scope`             |     2 |     2 |
| `ScopeKind`                       | type  | `scope`             |     2 |     3 |
| `UnusedVarContext`                | type  | `croquis`           |     1 |     4 |
| `Croquis.component_registrations` | field | `croquis`           |     2 |     2 |
| `Croquis.element_ids`             | field | `croquis`           |     1 |     2 |
| `Croquis.import_statements`       | field | `croquis`           |     1 |     1 |
| `Croquis.macros`                  | field | `croquis`           |     8 |    22 |
| `Croquis.scopes`                  | field | `croquis`           |     2 |     4 |

## Non-product `vize_croquis` imports

| item                                      | files | sites |
| ----------------------------------------- | ----: | ----: |
| `Bindings`                                |     1 |     2 |
| `BlockLocation`                           |     1 |     2 |
| `CroquisFacts`                            |     2 |     2 |
| `Demand`                                  |     2 |     4 |
| `FactConsumer`                            |     2 |     2 |
| `MacroKind`                               |     3 |     3 |
| `ReactiveKind`                            |     1 |     1 |
| `ReactivityLoss`                          |     1 |     2 |
| `ReactivityLossKind`                      |     1 |    21 |
| `ScriptParseResult`                       |     2 |     2 |
| `ScriptParserOptions`                     |     1 |     1 |
| `SfcCustomBlock`                          |     1 |     2 |
| `SfcDescriptor`                           |    11 |    14 |
| `SfcError`                                |     1 |     2 |
| `SfcParseOptions`                         |    12 |    13 |
| `UndefinedRefs`                           |     1 |     2 |
| `collect_options_descriptor`              |     4 |     4 |
| `collect_options_object`                  |     1 |     1 |
| `extract_slot_props`                      |     1 |     1 |
| `is_builtin_component`                    |     3 |     7 |
| `is_kebab_case_loose`                     |     1 |     1 |
| `is_pascal_case`                          |     2 |     2 |
| `names_match`                             |     3 |     3 |
| `parse_script_setup`                      |     3 |     3 |
| `parse_script_setup_with_generic_and_jsx` |     1 |     1 |
| `parse_script_with_options`               |     1 |     1 |
| `parse_v_for_expression`                  |     1 |     1 |
| `reactivity_lookup`                       |     1 |     1 |
| `to_pascal_case`                          |     2 |     3 |
| `used_component_name_list`                |     1 |     1 |

## Naive grep disagreements (resolved/grep)

| product                 | resolved | grep |
| ----------------------- | -------: | ---: |
| `COMPILER_MACRO_NAMES`  |        1 |    2 |
| `Croquis`               |       26 |   50 |
| `Drawer`                |        1 |    2 |
| `ElementIdKind`         |        2 |    3 |
| `OptionMember`          |        3 |    5 |
| `Scope`                 |        2 |   19 |
| `ScopeData`             |        2 |    4 |
| `ScopeId`               |        0 |    1 |
| `ScopeKind`             |        3 |    5 |
| `Span`                  |        0 |  219 |
| `Symbol`                |        0 |   15 |
| `SymbolId`              |        0 |    3 |
| `UnusedVarContext`      |        4 |    5 |
| `Croquis.bindings`      |        0 |   10 |
| `Croquis.macros`        |       22 |   31 |
| `Croquis.reactivity`    |        0 |    3 |
| `Croquis.scopes`        |        4 |   27 |
| `Croquis.template_info` |        0 |    1 |
| `Croquis.types`         |        0 |    1 |
