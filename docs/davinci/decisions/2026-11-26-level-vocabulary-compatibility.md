# Level vocabulary and supported Rust facades

The level vocabulary change in [#13707](https://github.com/ubugeeei-prod/vize/pull/13707)
uses `L2Template`, `L2Script`, `*Analysis`, `*Info`, and `*Record` for canonical
experimental level APIs. Their Rust names and diagnostic `Debug` labels may
change; versioned dump wire names retain their existing schemas and headers.

The alpha-supported and compatibility-preview crates keep their previously public facade names
for at least one published minor release with deprecation attributes:

| Existing supported name             | Canonical replacement                 |
| ----------------------------------- | ------------------------------------- |
| `vize_carton::fact`                 | `vize_carton::analysis`               |
| `vize_croquis::facts::FactConsumer` | `AnalysisConsumer`                    |
| `vize_croquis::facts::FactError`    | `AnalysisError`                       |
| `vize_croquis::facts::FactGroup`    | `Analysis`                            |
| `vize_croquis::facts::FactManager`  | `AnalysisManager`                     |
| `vize_croquis::facts::FactRegistry` | `AnalysisRegistry`                    |
| `vize_croquis::facts::FactTable`    | `AnalysisTable`                       |
| `vize_croquis::facts::FactView`     | `AnalysisView`                        |
| `vize_patina::L2Template`           | `vize_patina::LintL2Template`         |
| `vize_patina::markup::L2Template`   | `vize_patina::markup::LintL2Template` |

Carton's old module includes all nine nested module routes and the fifteen renamed
analysis items at their original root and nested paths: `FactGroup`, `FactConsumer`,
`FactProducer`, `FactManager`, `FactRegistry`, `FactTable`, `FactTableBuilder`,
`FactView`, `FactError`, `FactVerify`, `FactVerifyObserver`, `NoFactVerify`,
`ExpressionFacts`, `ExpressionFact`, and `ExpressionFactsAlpha`. Its wildcard
re-export of the experimental foundation made these supported Carton APIs.

The introducing minor is the first tagged release whose source includes #13707.
Removal must occur in a later minor after these deprecated names have been
published for one minor; the removal change must identify both releases.
An experimental level breaking marker does not waive this supported-facade
window. This follows the Cargo `package.metadata.vize.stability` values and
the [Rust compatibility policy](../../content/stability.md#rust-crate-support-tiers).

The old trait, error-enum, and unit-struct names are re-exports of the canonical
items so existing external `impl` blocks, enum-variant imports, and unit
construction and patterns keep their exact identity. Rust 1.100 does not emit a
deprecation warning for direct imports of these final re-exported items;
their explicit deprecation attributes and this migration record still name
the replacement. Importing Carton's deprecated `fact` module itself and using
the concrete type aliases emit the usual deprecation warnings.
No wrapper traits, blanket implementations, or restored experimental
`vize_l0::fact` module are introduced.
