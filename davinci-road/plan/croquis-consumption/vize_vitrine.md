<!-- GENERATED FILE — do not edit by hand.
     Regenerate: rust-script tools/commands/davinci/croquis-consumers.rs --write
     Verify:     rust-script tools/commands/davinci/croquis-consumers.rs --check -->

# Croquis consumption: `vize_vitrine`

One shard of the [Croquis consumption matrix](../croquis-consumption.md): every fact the matrix records about `crates/vize_vitrine/src`. The method and the product set live on that page.

## Resolved product sites

| product                       | kind  | module       | files | sites |
| ----------------------------- | ----- | ------------ | ----: | ----: |
| `Analyzer`                    | type  | `analyzer`   |     1 |     1 |
| `AnalyzerOptions`             | type  | `analyzer`   |     1 |     1 |
| `Croquis`                     | type  | `croquis`    |     2 |     2 |
| `InvalidExportKind`           | type  | `croquis`    |     1 |     6 |
| `ReactivityTracker`           | type  | `reactivity` |     1 |     1 |
| `ScopeKind`                   | type  | `scope`      |     1 |     6 |
| `TypeExportKind`              | type  | `croquis`    |     1 |     2 |
| `Croquis.binding_spans`       | field | `croquis`    |     1 |     1 |
| `Croquis.bindings`            | field | `croquis`    |     1 |     2 |
| `Croquis.invalid_exports`     | field | `croquis`    |     1 |     2 |
| `Croquis.macros`              | field | `croquis`    |     1 |     3 |
| `Croquis.pattern_diagnostics` | field | `croquis`    |     2 |     3 |
| `Croquis.provide_inject`      | field | `croquis`    |     1 |     2 |
| `Croquis.reactivity`          | field | `croquis`    |     1 |     1 |
| `Croquis.scopes`              | field | `croquis`    |     2 |     3 |
| `Croquis.type_exports`        | field | `croquis`    |     1 |     2 |
| `Croquis.unused_bindings`     | field | `croquis`    |     1 |     1 |

## Non-product `vize_croquis` imports

| item                                         | files | sites |
| -------------------------------------------- | ----: | ----: |
| `InjectPattern`                              |     1 |     8 |
| `ProvideKey`                                 |     1 |     4 |
| `SfcDescriptor`                              |     3 |     3 |
| `SfcParseOptions`                            |    10 |    15 |
| `generate_declaration_ts`                    |     1 |     2 |
| `generate_declaration_ts_with_split_scripts` |     1 |     1 |
| `parse_sfc`                                  |     1 |     1 |

## Naive grep disagreements (resolved/grep)

| product             | resolved | grep |
| ------------------- | -------: | ---: |
| `Analyzer`          |        1 |    2 |
| `BindingMetadata`   |        0 |    6 |
| `Croquis`           |        2 |    5 |
| `ReactivityTracker` |        1 |    2 |
| `Scope`             |        0 |    2 |
| `ScopeId`           |        0 |    2 |
| `Span`              |        0 |    3 |
| `Symbol`            |        0 |    3 |
| `Croquis.bindings`  |        2 |   14 |
| `Croquis.scopes`    |        3 |    8 |
