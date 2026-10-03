<!-- GENERATED FILE — do not edit by hand.
     Regenerate: rust-script tools/commands/davinci/croquis-consumers.rs --write
     Verify:     rust-script tools/commands/davinci/croquis-consumers.rs --check -->

# Croquis consumption: `vize_vitrine`

One shard of the [Croquis consumption matrix](../croquis-consumption.md): resolved consumption and non-product imports in `crates/vize_vitrine/src`. The method and product set live on that page; fresh naive-grep diagnostics are printed by `--check` and `--summary`.

## Resolved product sites

| product                       | kind  | module       | files | sites |
| ----------------------------- | ----- | ------------ | ----: | ----: |
| `Analyzer`                    | type  | `analyzer`   |     1 |     1 |
| `AnalyzerOptions`             | type  | `analyzer`   |     1 |     1 |
| `Croquis`                     | type  | `croquis`    |     4 |     5 |
| `InvalidExportKind`           | type  | `croquis`    |     1 |     6 |
| `ReactivityTracker`           | type  | `reactivity` |     1 |     1 |
| `ScopeKind`                   | type  | `scope`      |     1 |     6 |
| `TypeExportKind`              | type  | `croquis`    |     1 |     2 |
| `Croquis.binding_spans`       | field | `croquis`    |     1 |     1 |
| `Croquis.bindings`            | field | `croquis`    |     2 |     2 |
| `Croquis.invalid_exports`     | field | `croquis`    |     1 |     2 |
| `Croquis.macros`              | field | `croquis`    |     1 |     3 |
| `Croquis.pattern_diagnostics` | field | `croquis`    |     2 |     3 |
| `Croquis.reactivity`          | field | `croquis`    |     1 |     1 |
| `Croquis.scopes`              | field | `croquis`    |     2 |     3 |
| `Croquis.type_exports`        | field | `croquis`    |     1 |     2 |
| `Croquis.unused_bindings`     | field | `croquis`    |     1 |     1 |

## Non-product `vize_croquis` imports

| item                                         | files | sites |
| -------------------------------------------- | ----: | ----: |
| `BindingKey`                                 |     1 |     2 |
| `BindingType`                                |     1 |     1 |
| `Bindings`                                   |     2 |     5 |
| `CROQUIS_FACTS`                              |     1 |     2 |
| `ComponentContract`                          |     1 |     1 |
| `ComponentIdentity`                          |     1 |     1 |
| `ComponentUsages`                            |     1 |     3 |
| `CroquisFacts`                               |     1 |     1 |
| `Demand`                                     |     1 |     2 |
| `EmitContract`                               |     1 |     1 |
| `FactConsumer`                               |     2 |     2 |
| `FactManager`                                |     1 |     2 |
| `GroupedComponentUse`                        |     1 |     1 |
| `InjectPattern`                              |     2 |    13 |
| `PropContract`                               |     1 |     1 |
| `ProvideInject`                              |     1 |     3 |
| `ProvideInjectFact`                          |     1 |     4 |
| `ProvideInjectKey`                           |     1 |     4 |
| `ProvideKey`                                 |     2 |     7 |
| `RaceConditionRisk`                          |     1 |     1 |
| `RaceConditionRiskKind`                      |     1 |     5 |
| `RaceConditions`                             |     1 |     3 |
| `ReactiveKind`                               |     1 |     9 |
| `Reactivity`                                 |     1 |     3 |
| `ReactivityContract`                         |     1 |     1 |
| `ReactivityFact`                             |     1 |     3 |
| `ReactivityKey`                              |     1 |     3 |
| `ReactivityLossKind`                         |     1 |    11 |
| `SfcDescriptor`                              |     5 |     5 |
| `SfcError`                                   |     1 |     1 |
| `SfcParseOptions`                            |    11 |    16 |
| `SignatureContract`                          |     1 |     1 |
| `SlotContract`                               |     1 |     1 |
| `UndefinedRefs`                              |     1 |     3 |
| `UnusedBindings`                             |     1 |     3 |
| `generate_declaration_ts`                    |     1 |     2 |
| `generate_declaration_ts_with_split_scripts` |     1 |     1 |
| `inject_entries`                             |     1 |     1 |
| `parse_sfc`                                  |     2 |     2 |
| `provide_entries`                            |     1 |     1 |
