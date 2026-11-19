<!-- GENERATED FILE — do not edit by hand.
     Regenerate: node tools/support/compat/levels/croquis-consumers.ts --write
     Verify:     node tools/support/compat/levels/croquis-consumers.ts --check -->

# Croquis consumption: `vize_vitrine`

One shard of the [Croquis consumption matrix](../croquis-consumption.md): resolved consumption and non-product imports in `crates/vize_vitrine/src`. The method and product set live on that page; fresh naive-grep diagnostics are printed by `--check` and `--summary`.

## Resolved product sites

| product                               | kind  | module         | files | sites |
| ------------------------------------- | ----- | -------------- | ----: | ----: |
| `Analyzer`                            | type  | `analyzer`     |     1 |     1 |
| `AnalyzerOptions`                     | type  | `analyzer`     |     1 |     1 |
| `Croquis`                             | type  | `croquis`      |     3 |     3 |
| `EffectGraphScript`                   | type  | `effect_graph` |     1 |     2 |
| `EffectGraphSummary`                  | type  | `effect_graph` |     2 |     8 |
| `InvalidExportKind`                   | type  | `croquis`      |     1 |     6 |
| `ReactivityTracker`                   | type  | `reactivity`   |     1 |     1 |
| `ScopeKind`                           | type  | `scope`        |     1 |     6 |
| `TypeExportKind`                      | type  | `croquis`      |     1 |     2 |
| `build_effect_graph_from_sfc_scripts` | type  | `effect_graph` |     1 |     1 |
| `Croquis.invalid_exports`             | field | `croquis`      |     1 |     2 |
| `Croquis.macros`                      | field | `croquis`      |     1 |     3 |
| `Croquis.pattern_diagnostics`         | field | `croquis`      |     2 |     3 |
| `Croquis.reactivity`                  | field | `croquis`      |     1 |     1 |
| `Croquis.scopes`                      | field | `croquis`      |     2 |     3 |
| `Croquis.type_exports`                | field | `croquis`      |     1 |     2 |
| `Croquis.unused_bindings`             | field | `croquis`      |     1 |     1 |

## Non-product `vize_croquis` imports

| item                                         | files | sites |
| -------------------------------------------- | ----: | ----: |
| `Bindings`                                   |     1 |     2 |
| `CroquisFacts`                               |     1 |     1 |
| `Demand`                                     |     1 |     2 |
| `FactConsumer`                               |     1 |     1 |
| `InjectPattern`                              |     1 |     8 |
| `ProvideKey`                                 |     1 |     4 |
| `SfcDescriptor`                              |     6 |     6 |
| `SfcError`                                   |     1 |     1 |
| `SfcParseOptions`                            |    10 |    15 |
| `generate_declaration_ts`                    |     1 |     2 |
| `generate_declaration_ts_with_split_scripts` |     1 |     1 |
| `inject_entries`                             |     1 |     1 |
| `provide_entries`                            |     1 |     1 |
