<!-- GENERATED FILE — do not edit by hand.
     Regenerate: rust-script tools/commands/davinci/croquis-consumers.rs --write
     Verify:     rust-script tools/commands/davinci/croquis-consumers.rs --check
     Totals:     rust-script tools/commands/davinci/croquis-consumers.rs --summary
     Generator:  tools/support/compat/davinci/croquis-consumers.mjs -->

# Croquis consumption matrix

Which workspace crates consume the public analysis products of `crates/vize_croquis`. Mechanizes the 2026-08-13 hand audit in [semantic-engine.md](../semantic-engine.md#the-problem-measured) (Davinci P0-7).

## Layout

This page holds only what depends on `vize_croquis` itself: the resolution method and the product set. Consumption facts are sharded one file per consuming crate, `davinci-road/plan/croquis-consumption/<crate>.md`: that crate's resolved product sites, its non-product `vize_croquis` imports, and its naive-grep disagreements. A crate with none of those has no shard.

Cross-crate aggregates — per-product totals, the products with no external consumer, and the resolved/grep totals — are deliberately **not committed**: they changed with every PR and made every open PR conflict. They are pure sums over the shards; print them with `rust-script tools/commands/davinci/croquis-consumers.rs --summary`. The staleness check (TS-12) byte-compares this page, every shard, and the shard set itself (a leftover shard is stale).

## Resolution method (and its limits)

**Product enumeration** — parsed from source, not hardcoded:

- `pub` fields of the `Croquis` struct in `crates/vize_croquis/src/croquis.rs` (rows named `Croquis.<field>`), plus the tracker/product types those fields reference, resolved through croquis.rs's own `use crate::…` declarations.
- Types re-exported by croquis.rs (`pub use bindings::…`, `snapshot::…`, …) and the crate-root `pub use` groups in `crates/vize_croquis/src/lib.rs` whose source is a local module (this is what brings in the `effect_graph`, `scope`, `symbol`, `analyzer`, `drawer`, and `reactivity_overlay` families).
- Crate-root passthrough re-exports of foreign items are **excluded** from the product set: `vize_carton::is_builtin_directive`, `vize_carton::is_builtin_tag`, `vize_carton::is_html_tag`, `vize_carton::is_math_ml_tag`, `vize_carton::is_native_tag`, `vize_carton::is_reserved_prop`, `vize_carton::is_svg_tag`, `vize_carton::is_void_tag`, `vize_relief::BindingType`.

**Consumer resolution** — symbol-aware, per `crates/*/src/**/*.rs`:

- Rust `use` declarations are parsed (brace groups, `as` aliases, `pub use`) into per-file alias tables mapping local names to `vize_croquis` items; `pub use` re-export chains across crates are followed to a fixpoint (name-level, see limits). Comments and string literals are stripped before any counting, and the `use` declarations themselves are not counted as reference sites (a `pub use` re-export counts as one site).
- **type rows** — sites are references to a resolved local alias, a module-qualified member (`reactivity::ReactiveKind`), or a fully qualified `vize_croquis::…` path.
- **`Croquis.<field>` rows** — sites are field accesses (`summary.bindings`) counted only on receivers resolved to `Croquis` values: idents with a `Croquis` type annotation (params, struct fields, `let`, `&`/`&'a`/`&mut`/`Option<&…>`/`Box`/`Rc`/`Arc` wrappers), calls to same-file functions returning `Croquis`, and `let`-bindings whose right-hand side calls a workspace `pub fn` returning `Croquis` (`drawer.finish()`, `ctx.analysis()`), reads a workspace `pub` field typed `Croquis` (`result.croquis`), or calls an associated function on the `Croquis` type itself (`Croquis::default()`) — producer tables parsed from `crates/*/src`, matched by name. Inline chains through those producers (`entry.analysis.race_conditions`, `ctx.croquis().bindings`) are counted too.
- **naive grep lane** (cross-check) — raw word-boundary text matches per product name (`\.field` matches for field rows) over the same files — comments, strings, doc text, and same-named unrelated symbols included, imports included. Disagreements are listed per crate, **not** reconciled: `grep > resolved` usually means comments/unrelated same-named symbols (for field rows: field accesses on non-`Croquis` receivers); `grep < resolved` would indicate a resolver bug and must be investigated.

**Known limits** (undercounts are possible; the naive grep lane bounds them):

- Re-export chains resolve by item **name**, not full module path; same-named items reached through different facade modules would be conflated.
- No type inference: field accesses through closure params, iterator chains, destructuring patterns, or re-borrowed locals (`let b = &a;`) are not counted; the producer tables match croquis-returning method **names** without owner types, so a same-named method on an unrelated type can mark a false receiver (only matters if that value also has a product-named field).
- Macro-generated code is invisible to source parsing.
- `#[cfg(test)]` code inside `src/` is included; `tests/`, `benches/`, `examples/` directories are not scanned. `vize_croquis` itself is excluded (internal use is not consumption). Note that `vize_croquis_cf` is a separate crate and therefore counted as an external consumer, even though it is part of the same semantic layer.
- No glob imports (`use vize_croquis::…::*`) exist in the workspace today.

## Product set

| product                                | kind  | module               |
| -------------------------------------- | ----- | -------------------- |
| `AnalysisStats`                        | type  | `croquis`            |
| `Analyzer`                             | type  | `analyzer`           |
| `AnalyzerOptions`                      | type  | `analyzer`           |
| `BindingFlags`                         | type  | `scope`              |
| `BindingMetadata`                      | type  | `croquis`            |
| `BlockKind`                            | type  | `scope`              |
| `BlockScopeData`                       | type  | `scope`              |
| `COMPILER_MACRO_NAMES`                 | type  | `croquis`            |
| `CallbackScopeData`                    | type  | `scope`              |
| `ClientOnlyScopeData`                  | type  | `scope`              |
| `ClosureScopeData`                     | type  | `scope`              |
| `ComponentRegistration`                | type  | `croquis::template`  |
| `ComponentShape`                       | type  | `croquis`            |
| `ComponentUsage`                       | type  | `croquis::template`  |
| `Croquis`                              | type  | `croquis`            |
| `CroquisSemanticSnapshot`              | type  | `croquis`            |
| `CroquisSemanticSummary`               | type  | `croquis`            |
| `CroquisStats`                         | type  | `croquis`            |
| `Drawer`                               | type  | `drawer`             |
| `DrawerOptions`                        | type  | `drawer`             |
| `EffectGraph`                          | type  | `effect_graph`       |
| `EffectGraphScript`                    | type  | `effect_graph`       |
| `EffectGraphSummary`                   | type  | `effect_graph`       |
| `ElementIdInfo`                        | type  | `croquis::template`  |
| `ElementIdKind`                        | type  | `croquis::template`  |
| `EventHandlerScopeData`                | type  | `scope`              |
| `EventListener`                        | type  | `croquis::template`  |
| `ExternalModuleScopeData`              | type  | `scope`              |
| `HoistTracker`                         | type  | `hoist`              |
| `ImportStatementInfo`                  | type  | `croquis`            |
| `ImportedExport`                       | type  | `scope`              |
| `InvalidExport`                        | type  | `croquis`            |
| `InvalidExportKind`                    | type  | `croquis`            |
| `JsGlobalScopeData`                    | type  | `scope`              |
| `JsRuntime`                            | type  | `scope`              |
| `MacroTracker`                         | type  | `macros`             |
| `NonScriptSetupScopeData`              | type  | `scope`              |
| `OptionGroup`                          | type  | `croquis`            |
| `OptionKey`                            | type  | `croquis`            |
| `OptionMember`                         | type  | `croquis`            |
| `OptionsDescriptor`                    | type  | `croquis`            |
| `PARAM_INLINE_CAP`                     | type  | `scope`              |
| `ParamNames`                           | type  | `scope`              |
| `ParentScopes`                         | type  | `scope`              |
| `PassedProp`                           | type  | `croquis::template`  |
| `ProvideInjectTracker`                 | type  | `provide`            |
| `RaceConditionTracker`                 | type  | `race`               |
| `ReExportForward`                      | type  | `croquis`            |
| `ReExportInfo`                         | type  | `croquis`            |
| `ReactivityEffectEdgeOverlay`          | type  | `reactivity_overlay` |
| `ReactivityEffectGraphOverlay`         | type  | `reactivity_overlay` |
| `ReactivityLossOverlay`                | type  | `reactivity_overlay` |
| `ReactivityOverlay`                    | type  | `reactivity_overlay` |
| `ReactivityOverlaySummary`             | type  | `reactivity_overlay` |
| `ReactivitySourceOverlay`              | type  | `reactivity_overlay` |
| `ReactivityTracker`                    | type  | `reactivity`         |
| `Scope`                                | type  | `scope`              |
| `ScopeBinding`                         | type  | `scope`              |
| `ScopeChain`                           | type  | `scope`              |
| `ScopeData`                            | type  | `scope`              |
| `ScopeId`                              | type  | `scope`              |
| `ScopeKind`                            | type  | `scope`              |
| `ScriptSetupScopeData`                 | type  | `scope`              |
| `SemanticBindingSnapshot`              | type  | `croquis`            |
| `SemanticComponentUsageSnapshot`       | type  | `croquis`            |
| `SemanticEventListenerSnapshot`        | type  | `croquis`            |
| `SemanticInjectSnapshot`               | type  | `croquis`            |
| `SemanticPassedPropSnapshot`           | type  | `croquis`            |
| `SemanticProvideSnapshot`              | type  | `croquis`            |
| `SemanticReactiveSourceSnapshot`       | type  | `croquis`            |
| `SemanticReactivityLossSnapshot`       | type  | `croquis`            |
| `SemanticScopeBindingSnapshot`         | type  | `croquis`            |
| `SemanticScopeSnapshot`                | type  | `croquis`            |
| `SemanticSlotUsageSnapshot`            | type  | `croquis`            |
| `SemanticSourceRange`                  | type  | `croquis`            |
| `SemanticTemplateExpressionSnapshot`   | type  | `croquis`            |
| `SetupContextTracker`                  | type  | `setup_context`      |
| `SlotUsage`                            | type  | `croquis::template`  |
| `Span`                                 | type  | `scope`              |
| `SpreadProp`                           | type  | `croquis::template`  |
| `Symbol`                               | type  | `symbol`             |
| `SymbolFlags`                          | type  | `symbol`             |
| `SymbolId`                             | type  | `symbol`             |
| `SymbolTable`                          | type  | `symbol`             |
| `TemplateExpression`                   | type  | `croquis`            |
| `TemplateExpressionKind`               | type  | `croquis`            |
| `TemplateInfo`                         | type  | `croquis::template`  |
| `TypeExport`                           | type  | `croquis`            |
| `TypeExportKind`                       | type  | `croquis`            |
| `TypeResolver`                         | type  | `types`              |
| `UndefinedRef`                         | type  | `croquis`            |
| `UniversalScopeData`                   | type  | `scope`              |
| `UnusedTemplateVar`                    | type  | `croquis`            |
| `UnusedVarContext`                     | type  | `croquis`            |
| `VForScopeData`                        | type  | `scope`              |
| `VSlotScopeData`                       | type  | `scope`              |
| `VueGlobalScopeData`                   | type  | `scope`              |
| `build_effect_graph_from_script`       | type  | `effect_graph`       |
| `build_effect_graph_from_script_setup` | type  | `effect_graph`       |
| `build_effect_graph_from_sfc_scripts`  | type  | `effect_graph`       |
| `Croquis.binding_spans`                | field | `croquis`            |
| `Croquis.bindings`                     | field | `croquis`            |
| `Croquis.component_registrations`      | field | `croquis`            |
| `Croquis.component_shape`              | field | `croquis`            |
| `Croquis.component_usages`             | field | `croquis`            |
| `Croquis.element_ids`                  | field | `croquis`            |
| `Croquis.hoists`                       | field | `croquis`            |
| `Croquis.import_statements`            | field | `croquis`            |
| `Croquis.invalid_exports`              | field | `croquis`            |
| `Croquis.macros`                       | field | `croquis`            |
| `Croquis.options_descriptor`           | field | `croquis`            |
| `Croquis.pattern_diagnostics`          | field | `croquis`            |
| `Croquis.provide_inject`               | field | `croquis`            |
| `Croquis.race_conditions`              | field | `croquis`            |
| `Croquis.re_export_forwards`           | field | `croquis`            |
| `Croquis.re_exports`                   | field | `croquis`            |
| `Croquis.reactivity`                   | field | `croquis`            |
| `Croquis.scopes`                       | field | `croquis`            |
| `Croquis.setup_context`                | field | `croquis`            |
| `Croquis.symbols`                      | field | `croquis`            |
| `Croquis.template_expressions`         | field | `croquis`            |
| `Croquis.template_info`                | field | `croquis`            |
| `Croquis.type_exports`                 | field | `croquis`            |
| `Croquis.types`                        | field | `croquis`            |
| `Croquis.undefined_refs`               | field | `croquis`            |
| `Croquis.unused_bindings`              | field | `croquis`            |
| `Croquis.used_components`              | field | `croquis`            |
| `Croquis.used_directives`              | field | `croquis`            |
